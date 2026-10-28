Set-StrictMode -Version Latest

function Assert-ManifestValue {
    param(
        [Parameter(Mandatory = $true)] $Object,
        [Parameter(Mandatory = $true)] [string] $Property,
        [Parameter(Mandatory = $true)] [string] $Context
    )

    if ($null -eq $Object -or $null -eq $Object.PSObject.Properties[$Property]) {
        throw "Manifest is missing $Context.$Property."
    }
    return $Object.$Property
}

function Test-BuildArtifactManifest {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)] [string] $ManifestPath,
        [Parameter(Mandatory = $true)] [string] $RepositoryRoot
    )

    if (-not (Test-Path -LiteralPath $ManifestPath -PathType Leaf)) {
        throw "Build artifact manifest does not exist: $ManifestPath"
    }
    if (-not (Test-Path -LiteralPath $RepositoryRoot -PathType Container)) {
        throw "Repository root does not exist: $RepositoryRoot"
    }

    try {
        $manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
    }
    catch {
        throw "Build artifact manifest is not valid JSON: $($_.Exception.Message)"
    }

    if ((Assert-ManifestValue $manifest 'schema_version' 'manifest') -ne 1) {
        throw 'Unsupported build artifact manifest schema_version; expected 1.'
    }
    $buildMode = [string](Assert-ManifestValue $manifest 'build_mode' 'manifest')
    if ($buildMode -notin @('baseline', 'brepro', 'path-remap', 'brepro-and-path-remap')) {
        throw "Unsupported build_mode '$buildMode'."
    }

    $runId = [string](Assert-ManifestValue $manifest 'run_id' 'manifest')
    if ($runId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$' -or $runId -in @('.', '..')) {
        throw 'Manifest run_id contains unsupported characters.'
    }

    $source = Assert-ManifestValue $manifest 'source' 'manifest'
    $revision = [string](Assert-ManifestValue $source 'revision' 'source')
    if ($revision -notmatch '^[0-9a-fA-F]{40}$') {
        throw 'source.revision must be a full 40-character Git commit ID.'
    }
    $dirty = Assert-ManifestValue $source 'dirty' 'source'
    if ($dirty -isnot [bool]) {
        throw 'source.dirty must be a JSON boolean.'
    }

    $environment = Assert-ManifestValue $manifest 'environment' 'manifest'
    $target = [string](Assert-ManifestValue $environment 'target' 'environment')
    if ($target -ne 'x86_64-pc-windows-msvc') {
        throw "Unsupported target '$target'; this workflow is Windows x64/MSVC only."
    }
    foreach ($field in @('rustc_version', 'cargo_version')) {
        $value = [string](Assert-ManifestValue $environment $field 'environment')
        if ([string]::IsNullOrWhiteSpace($value)) { throw "environment.$field cannot be empty." }
    }
    $rustcCommit = [string](Assert-ManifestValue $environment 'rustc_commit_hash' 'environment')
    if ($rustcCommit -notmatch '^[0-9a-fA-F]{40}$') { throw 'environment.rustc_commit_hash must be a full commit ID.' }
    if ((Assert-ManifestValue $environment 'rustc_host' 'environment') -ne $target) {
        throw 'environment.rustc_host does not match the requested target.'
    }
    if ((Assert-ManifestValue $environment 'profile' 'environment') -ne 'release') {
        throw 'Only release artifacts are accepted by this manifest verifier.'
    }
    $lockHash = [string](Assert-ManifestValue $environment 'cargo_lock_sha256' 'environment')
    if ($lockHash -notmatch '^[0-9a-fA-F]{64}$') {
        throw 'environment.cargo_lock_sha256 must be a full SHA-256 value.'
    }
    $rustflags = @(Assert-ManifestValue $environment 'rustflags' 'environment')
    foreach ($flag in $rustflags) {
        if ($flag -isnot [string] -or [string]::IsNullOrWhiteSpace($flag)) {
            throw 'environment.rustflags must contain only non-empty strings.'
        }
    }
    $expectedRustflags = switch ($buildMode) {
        'baseline' { @() }
        'brepro' { @('-C', 'link-arg=/Brepro') }
        'path-remap' { @('--remap-path-prefix=<repository-root>=/encryptdrive', '--remap-path-prefix=<cargo-home>=/.cargo') }
        'brepro-and-path-remap' { @('-C', 'link-arg=/Brepro', '--remap-path-prefix=<repository-root>=/encryptdrive', '--remap-path-prefix=<cargo-home>=/.cargo') }
    }
    if (($rustflags -join '|') -cne (@($expectedRustflags) -join '|')) {
        throw "environment.rustflags do not match build_mode '$buildMode'."
    }
    $linker = Assert-ManifestValue $environment 'linker' 'environment'
    $linkerStatus = [string](Assert-ManifestValue $linker 'status' 'linker')
    if ($linkerStatus -eq 'available') {
        if ([string]::IsNullOrWhiteSpace([string](Assert-ManifestValue $linker 'name' 'linker')) -or
            [string]::IsNullOrWhiteSpace([string](Assert-ManifestValue $linker 'version' 'linker'))) {
            throw 'An available linker must include its name and version.'
        }
    }
    elseif ($linkerStatus -eq 'unavailable') {
        if ($null -ne (Assert-ManifestValue $linker 'name' 'linker') -or
            $null -ne (Assert-ManifestValue $linker 'version' 'linker')) {
            throw 'An unavailable linker must use null for name and version.'
        }
    }
    else {
        throw "Unsupported linker.status '$linkerStatus'."
    }

    $artifacts = @(Assert-ManifestValue $manifest 'artifacts' 'manifest')
    if ($artifacts.Count -ne 2) {
        throw "Manifest must contain exactly two build variants; found $($artifacts.Count)."
    }

    $expectedFeatures = @{
        'shell' = @()
        'design-preview' = @('design-preview')
    }
    $seenVariants = @{}
    $seenPaths = @{}
    $resolvedRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $rootPrefix = $resolvedRoot + [IO.Path]::DirectorySeparatorChar

    foreach ($artifact in $artifacts) {
        $variant = [string](Assert-ManifestValue $artifact 'variant' 'artifact')
        if (-not $expectedFeatures.ContainsKey($variant)) { throw "Unexpected artifact variant '$variant'." }
        if ($seenVariants.ContainsKey($variant)) { throw "Duplicate artifact variant '$variant'." }
        $seenVariants[$variant] = $true

        $features = @(Assert-ManifestValue $artifact 'features' "artifact '$variant'")
        $expected = @($expectedFeatures[$variant])
        if (($features -join '|') -cne ($expected -join '|')) {
            throw "Artifact '$variant' has the wrong feature set; expected [$($expected -join ', ')]."
        }
        if ((Assert-ManifestValue $artifact 'profile' "artifact '$variant'") -ne 'release') {
            throw "Artifact '$variant' is not a release build."
        }
        if ((Assert-ManifestValue $artifact 'target' "artifact '$variant'") -ne $target) {
            throw "Artifact '$variant' target differs from the environment target."
        }

        $relativePath = [string](Assert-ManifestValue $artifact 'relative_path' "artifact '$variant'")
        if ([string]::IsNullOrWhiteSpace($relativePath) -or $relativePath.Contains('\') -or
            [IO.Path]::IsPathRooted($relativePath) -or $relativePath -match '^[A-Za-z]:' -or
            $relativePath -match '(^|/)\.\.?(/|$)' -or $relativePath.Contains('//')) {
            throw "Artifact '$variant' path must be normalized, relative, and contain no traversal segments."
        }
        $expectedPrefix = "target/phase-1b/artifacts/$runId/"
        if (-not $relativePath.StartsWith($expectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Artifact '$variant' must be stored below $expectedPrefix."
        }
        $cargoTargetRelative = [string](Assert-ManifestValue $artifact 'cargo_target_relative' "artifact '$variant'")
        $expectedCargoTarget = "target/phase-1b/cargo-target/$variant"
        if ($cargoTargetRelative -cne $expectedCargoTarget) {
            throw "Artifact '$variant' must use its isolated Cargo target directory '$expectedCargoTarget'."
        }
        $cargoTargetPath = Join-Path $resolvedRoot ($cargoTargetRelative -replace '/', [string][IO.Path]::DirectorySeparatorChar)
        if (-not (Test-Path -LiteralPath $cargoTargetPath -PathType Container)) {
            throw "Artifact '$variant' Cargo target directory is missing."
        }
        $pathKey = $relativePath.ToLowerInvariant()
        if ($seenPaths.ContainsKey($pathKey)) { throw "Variants share an output path: $relativePath" }
        $seenPaths[$pathKey] = $true

        $filePath = Join-Path $resolvedRoot ($relativePath -replace '/', [string][IO.Path]::DirectorySeparatorChar)
        if (-not (Test-Path -LiteralPath $filePath -PathType Leaf)) {
            throw "Artifact '$variant' does not exist at its manifest path."
        }
        $resolvedFile = (Resolve-Path -LiteralPath $filePath).Path
        if (-not $resolvedFile.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Artifact '$variant' resolves outside the repository root."
        }
        if ([IO.Path]::GetExtension($resolvedFile) -ine '.exe') {
            throw "Artifact '$variant' must reference a Windows .exe file."
        }

        $size = Assert-ManifestValue $artifact 'size_bytes' "artifact '$variant'"
        $actualSize = (Get-Item -LiteralPath $resolvedFile).Length
        if ($size -isnot [ValueType] -or [long]$size -le 0 -or [long]$size -ne $actualSize) {
            throw "Artifact '$variant' size_bytes does not match the file."
        }
        $expectedHash = [string](Assert-ManifestValue $artifact 'sha256' "artifact '$variant'")
        if ($expectedHash -notmatch '^[0-9a-fA-F]{64}$') {
            throw "Artifact '$variant' is missing a full SHA-256 value."
        }
        $actualHash = (Get-FileHash -LiteralPath $resolvedFile -Algorithm SHA256).Hash
        if (-not $actualHash.Equals($expectedHash, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Artifact '$variant' SHA-256 does not match the file bytes."
        }

        $pe = Assert-ManifestValue $artifact 'pe_metadata' "artifact '$variant'"
        $timestamp = [string](Assert-ManifestValue $pe 'coff_timestamp_hex' "artifact '$variant'.pe_metadata")
        if ($timestamp -notmatch '^0x[0-9A-Fa-f]{8}$') {
            throw "Artifact '$variant' has an invalid PE COFF timestamp value."
        }
        $pdbFile = Assert-ManifestValue $pe 'codeview_pdb_file' "artifact '$variant'.pe_metadata"
        $pdbGuid = Assert-ManifestValue $pe 'codeview_guid' "artifact '$variant'.pe_metadata"
        $pdbAge = Assert-ManifestValue $pe 'codeview_age' "artifact '$variant'.pe_metadata"
        if ($null -ne $pdbFile -and ([string]$pdbFile -match '[/\\:]')) { throw "Artifact '$variant' PDB metadata must contain only a basename." }
        if ($null -ne $pdbGuid -and [string]$pdbGuid -notmatch '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$') {
            throw "Artifact '$variant' has an invalid CodeView PDB GUID."
        }
        if ($null -ne $pdbAge -and ([long]$pdbAge -lt 1 -or $pdbAge -gt [uint32]::MaxValue)) { throw "Artifact '$variant' has an invalid CodeView PDB age." }
        if (($null -eq $pdbFile) -ne ($null -eq $pdbGuid) -or ($null -eq $pdbGuid) -ne ($null -eq $pdbAge)) {
            throw "Artifact '$variant' PDB filename, GUID, and age must be present or absent together."
        }
    }

    foreach ($variant in @('shell', 'design-preview')) {
        if (-not $seenVariants.ContainsKey($variant)) { throw "Manifest is missing the '$variant' build variant." }
    }

    return [pscustomobject]@{
        status = 'verified'
        run_id = $runId
        source_revision = $revision.ToLowerInvariant()
        source_dirty = $dirty
        artifact_count = $artifacts.Count
        variants = @($seenVariants.Keys | Sort-Object)
    }
}

function Get-PeBuildMetadata {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)] [string] $Path)

    $bytes = [IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 256 -or [BitConverter]::ToUInt16($bytes, 0) -ne 0x5A4D) {
        throw "File is not a valid PE image: $Path"
    }
    $peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
    if ($peOffset -lt 0 -or $peOffset + 24 -gt $bytes.Length -or
        $bytes[$peOffset] -ne 0x50 -or $bytes[$peOffset + 1] -ne 0x45 -or
        $bytes[$peOffset + 2] -ne 0 -or $bytes[$peOffset + 3] -ne 0) {
        throw "File has an invalid PE signature: $Path"
    }

    $coff = $peOffset + 4
    $sectionCount = [BitConverter]::ToUInt16($bytes, $coff + 2)
    $timestamp = [BitConverter]::ToUInt32($bytes, $coff + 4)
    $optionalSize = [BitConverter]::ToUInt16($bytes, $coff + 16)
    $optional = $coff + 20
    if ($optional + $optionalSize -gt $bytes.Length) { throw "PE optional header is truncated: $Path" }
    $magic = [BitConverter]::ToUInt16($bytes, $optional)
    if ($magic -eq 0x20B) { $directoryOffset = $optional + 112 }
    elseif ($magic -eq 0x10B) { $directoryOffset = $optional + 96 }
    else { throw "Unsupported PE optional header: $Path" }

    $result = [ordered]@{
        coff_timestamp_hex = ('0x{0:X8}' -f $timestamp)
        codeview_pdb_file = $null
        codeview_guid = $null
        codeview_age = $null
    }
    if ($directoryOffset + (8 * 7) -gt ($optional + $optionalSize)) {
        return [pscustomobject]$result
    }

    $debugRva = [BitConverter]::ToUInt32($bytes, $directoryOffset + (8 * 6))
    $debugSize = [BitConverter]::ToUInt32($bytes, $directoryOffset + (8 * 6) + 4)
    if ($debugRva -eq 0 -or $debugSize -lt 28) { return [pscustomobject]$result }
    $sectionTable = $optional + $optionalSize
    if ($sectionTable + ($sectionCount * 40) -gt $bytes.Length) { throw "PE section table is truncated: $Path" }

    $debugOffset = $null
    for ($index = 0; $index -lt $sectionCount; $index++) {
        $section = $sectionTable + ($index * 40)
        $virtualSize = [BitConverter]::ToUInt32($bytes, $section + 8)
        $virtualAddress = [BitConverter]::ToUInt32($bytes, $section + 12)
        $rawSize = [BitConverter]::ToUInt32($bytes, $section + 16)
        $rawPointer = [BitConverter]::ToUInt32($bytes, $section + 20)
        $span = [Math]::Max([double]$virtualSize, [double]$rawSize)
        if ($debugRva -ge $virtualAddress -and $debugRva -lt ($virtualAddress + $span)) {
            $debugOffset = [long]$rawPointer + ([long]$debugRva - [long]$virtualAddress)
            break
        }
    }
    if ($null -eq $debugOffset -or $debugOffset + $debugSize -gt $bytes.Length) {
        return [pscustomobject]$result
    }

    $entryCount = [Math]::Floor($debugSize / 28)
    for ($index = 0; $index -lt $entryCount; $index++) {
        $entry = [int]$debugOffset + ($index * 28)
        $entrySize = [BitConverter]::ToUInt32($bytes, $entry + 16)
        $entryType = [BitConverter]::ToUInt32($bytes, $entry + 12)
        $dataPointer = [BitConverter]::ToUInt32($bytes, $entry + 24)
        if ($entryType -ne 2 -or $entrySize -lt 25 -or $dataPointer + $entrySize -gt $bytes.Length) { continue }
        $data = [int]$dataPointer
        if ($bytes[$data] -ne 0x52 -or $bytes[$data + 1] -ne 0x53 -or
            $bytes[$data + 2] -ne 0x44 -or $bytes[$data + 3] -ne 0x53) { continue }
        $guidBytes = New-Object byte[] 16
        [Array]::Copy($bytes, $data + 4, $guidBytes, 0, 16)
        $guid = New-Object System.Guid (,$guidBytes)
        $age = [BitConverter]::ToUInt32($bytes, $data + 20)
        $nameStart = $data + 24
        $nameLength = [int]$entrySize - 24
        $nameBytes = New-Object byte[] $nameLength
        [Array]::Copy($bytes, $nameStart, $nameBytes, 0, $nameLength)
        $pdbText = [Text.Encoding]::ASCII.GetString($nameBytes).Trim([char]0)
        $result.codeview_pdb_file = [IO.Path]::GetFileName($pdbText)
        $result.codeview_guid = $guid.ToString('D')
        $result.codeview_age = $age
        break
    }

    return [pscustomobject]$result
}

Export-ModuleMember -Function Test-BuildArtifactManifest, Get-PeBuildMetadata
