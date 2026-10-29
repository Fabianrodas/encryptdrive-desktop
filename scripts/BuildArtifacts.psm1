#requires -Version 7.0

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

function Assert-AllowedManifestProperties {
    param(
        [Parameter(Mandatory = $true)] $Object,
        [Parameter(Mandatory = $true)] [string[]] $Allowed,
        [Parameter(Mandatory = $true)] [string] $Context
    )

    if ($null -eq $Object -or $Object -isnot [System.Management.Automation.PSCustomObject]) {
        throw "Manifest $Context must be a JSON object."
    }
    $allowedSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($name in $Allowed) { $null = $allowedSet.Add($name) }
    foreach ($property in $Object.PSObject.Properties) {
        if (-not $allowedSet.Contains($property.Name)) {
            throw "Unsupported property '$($property.Name)' in $Context."
        }
    }
}

function Assert-NoReparsePointPath {
    param(
        [Parameter(Mandatory = $true)] [string] $Path,
        [Parameter(Mandatory = $true)] [string] $Root,
        [Parameter(Mandatory = $true)] [string] $Context
    )

    $rootFull = [IO.Path]::GetFullPath($Root).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $rootItem = Get-Item -LiteralPath $rootFull -Force -ErrorAction Stop
    if (($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "$Context root must not be a reparse point."
    }

    $pathFull = [IO.Path]::GetFullPath($Path)
    if ($pathFull.Equals($rootFull, [StringComparison]::OrdinalIgnoreCase)) { return }
    $rootPrefix = $rootFull + [IO.Path]::DirectorySeparatorChar
    if (-not $pathFull.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Context path must remain beneath its permitted root."
    }

    $relative = [IO.Path]::GetRelativePath($rootFull, $pathFull)
    if ($relative -match '(^|[\\/])\.\.?([\\/]|$)') {
        throw "$Context path contains a traversal segment."
    }
    $current = $rootFull
    foreach ($segment in ($relative -split '[\\/]')) {
        if ([string]::IsNullOrWhiteSpace($segment)) { continue }
        $current = Join-Path $current $segment
        if (-not (Test-Path -LiteralPath $current)) { continue }
        $item = Get-Item -LiteralPath $current -Force -ErrorAction Stop
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "$Context path traverses a reparse point."
        }
    }
}

function Test-FileContainsAsciiMarker {
    param(
        [Parameter(Mandatory = $true)] [string] $Path,
        [Parameter(Mandatory = $true)] [string] $Marker
    )

    $contents = [Text.Encoding]::Latin1.GetString([IO.File]::ReadAllBytes($Path))
    return $contents.IndexOf($Marker, [StringComparison]::Ordinal) -ge 0
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

    $resolvedRoot = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $RepositoryRoot).Path).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    Assert-NoReparsePointPath -Path $ManifestPath -Root $resolvedRoot -Context 'Manifest'

    try {
        $manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
    }
    catch {
        throw "Build artifact manifest is not valid JSON: $($_.Exception.Message)"
    }

    Assert-AllowedManifestProperties -Object $manifest -Allowed @('schema_version', 'run_id', 'build_mode', 'source', 'environment', 'artifacts') -Context 'manifest'

    if ((Assert-ManifestValue $manifest 'schema_version' 'manifest') -ne 2) {
        throw 'Unsupported build artifact manifest schema_version; expected 2.'
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
    Assert-AllowedManifestProperties -Object $source -Allowed @('revision', 'dirty') -Context 'source'
    $revision = [string](Assert-ManifestValue $source 'revision' 'source')
    if ($revision -notmatch '^[0-9a-fA-F]{40}$') {
        throw 'source.revision must be a full 40-character Git commit ID.'
    }
    $dirty = Assert-ManifestValue $source 'dirty' 'source'
    if ($dirty -isnot [bool]) {
        throw 'source.dirty must be a JSON boolean.'
    }

    $environment = Assert-ManifestValue $manifest 'environment' 'manifest'
    Assert-AllowedManifestProperties -Object $environment -Allowed @('target', 'rustc_version', 'rustc_commit_hash', 'rustc_host', 'cargo_version', 'profile', 'cargo_lock_sha256', 'rustflags', 'linker') -Context 'environment'
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
    Assert-AllowedManifestProperties -Object $linker -Allowed @('status', 'name', 'version', 'detected_from', 'usage_proven') -Context 'linker'
    $linkerStatus = [string](Assert-ManifestValue $linker 'status' 'linker')
    if ((Assert-ManifestValue $linker 'detected_from' 'linker') -cne 'environment-probe') {
        throw "linker.detected_from must be 'environment-probe'."
    }
    $linkerUsageProven = Assert-ManifestValue $linker 'usage_proven' 'linker'
    if ($linkerUsageProven -isnot [bool] -or $linkerUsageProven) {
        throw 'linker.usage_proven must be false; this workflow records environment detection only.'
    }
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
    $rootPrefix = $resolvedRoot + [IO.Path]::DirectorySeparatorChar
    $runArtifactRoot = Join-Path $resolvedRoot "target\phase-1b\artifacts\$runId"
    Assert-NoReparsePointPath -Path $runArtifactRoot -Root $resolvedRoot -Context 'Artifact run root'

    foreach ($artifact in $artifacts) {
        Assert-AllowedManifestProperties -Object $artifact -Allowed @('variant', 'features', 'relative_path', 'cargo_target_relative', 'size_bytes', 'sha256', 'profile', 'target', 'pe_metadata') -Context 'artifact'
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
        Assert-NoReparsePointPath -Path $cargoTargetPath -Root $resolvedRoot -Context "Artifact '$variant' Cargo target"
        $pathKey = $relativePath.ToLowerInvariant()
        if ($seenPaths.ContainsKey($pathKey)) { throw "Variants share an output path: $relativePath" }
        $seenPaths[$pathKey] = $true

        $shortRevision = $revision.Substring(0, 12).ToLowerInvariant()
        $expectedRelativePath = "target/phase-1b/artifacts/$runId/$variant/EncryptDrive-$variant-$shortRevision-$runId.exe"
        if (-not $relativePath.Equals($expectedRelativePath, [StringComparison]::Ordinal)) {
            throw "Artifact '$variant' path does not match its expected artifact path '$expectedRelativePath'."
        }

        $filePath = Join-Path $resolvedRoot ($relativePath -replace '/', [string][IO.Path]::DirectorySeparatorChar)
        if (-not (Test-Path -LiteralPath $filePath -PathType Leaf)) {
            throw "Artifact '$variant' does not exist at its manifest path."
        }
        Assert-NoReparsePointPath -Path $filePath -Root $runArtifactRoot -Context "Artifact '$variant'"
        $resolvedFile = (Resolve-Path -LiteralPath $filePath).Path
        $artifactRunPrefix = $runArtifactRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
        if (-not $resolvedFile.StartsWith($artifactRunPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Artifact '$variant' resolves outside its permitted artifact run root."
        }
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
        $hasPreviewMarker = Test-FileContainsAsciiMarker -Path $resolvedFile -Marker '--design-preview'
        if ($variant -eq 'design-preview' -and -not $hasPreviewMarker) {
            throw "Artifact 'design-preview' is missing marker '--design-preview'."
        }
        if ($variant -eq 'shell' -and $hasPreviewMarker) {
            throw "Artifact 'shell' contains marker '--design-preview'."
        }

        $pe = Assert-ManifestValue $artifact 'pe_metadata' "artifact '$variant'"
        Assert-AllowedManifestProperties -Object $pe -Allowed @('coff_timestamp_hex', 'codeview_pdb_file', 'codeview_guid', 'codeview_age') -Context 'pe_metadata'
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

Export-ModuleMember -Function Test-BuildArtifactManifest, Get-PeBuildMetadata, Assert-NoReparsePointPath
