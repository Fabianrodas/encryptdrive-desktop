#requires -Version 7.0

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$')]
    [string] $RunId,
    [ValidateSet('baseline', 'brepro', 'path-remap', 'brepro-and-path-remap')]
    [string] $BuildMode = 'baseline',
    [string] $RepositoryRoot = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'BuildArtifacts.psm1') -Force
$target = 'x86_64-pc-windows-msvc'
$repositoryRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
$runRoot = Join-Path $repositoryRoot "target\phase-1b\artifacts\$RunId"
$cargoTargetRoot = Join-Path $repositoryRoot 'target\phase-1b\cargo-target'
$manifestPath = Join-Path $runRoot 'manifest.json'
$repositoryPrefix = $repositoryRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar

if ($env:OS -ne 'Windows_NT') {
    throw 'This build workflow is restricted to native Windows hosts.'
}
if (-not $runRoot.StartsWith($repositoryPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Artifact output must remain under the repository target directory.'
}
if (Test-Path -LiteralPath $runRoot) { throw "Run output already exists; refusing to overwrite '$RunId'." }
Assert-NoReparsePointPath -Path $repositoryRoot -Root $repositoryRoot -Context 'Repository'
Assert-NoReparsePointPath -Path $runRoot -Root $repositoryRoot -Context 'Artifact output'
Assert-NoReparsePointPath -Path $cargoTargetRoot -Root $repositoryRoot -Context 'Cargo target root'

$repoSafe = 'safe.directory=' + $repositoryRoot
$revision = (& git -c $repoSafe -C $repositoryRoot rev-parse HEAD 2>&1 | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $revision -notmatch '^[0-9a-fA-F]{40}$') { throw 'Could not identify the source Git revision.' }
$statusText = & git -c $repoSafe -C $repositoryRoot status --porcelain=v1 --untracked-files=normal 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) { throw 'Could not determine whether the source worktree is dirty.' }
$dirty = -not [string]::IsNullOrWhiteSpace($statusText)

Push-Location -LiteralPath $repositoryRoot
try {
    $rustcLines = @(& rustc --version --verbose 2>&1)
    if ($LASTEXITCODE -ne 0) { throw 'rustc --version --verbose failed.' }
    $rustcVersion = [string]($rustcLines | Select-Object -First 1)
    $rustcCommit = (($rustcLines | Where-Object { $_ -match '^commit-hash:' } | Select-Object -First 1) -replace '^commit-hash:\s*', '')
    $rustcHost = (($rustcLines | Where-Object { $_ -match '^host:' } | Select-Object -First 1) -replace '^host:\s*', '')
    if ($rustcHost -ne $target) { throw "Active rustc host '$rustcHost' does not match required target '$target'." }
    $cargoVersion = [string]((& cargo --version 2>&1 | Out-String).Trim())
    if ($LASTEXITCODE -ne 0) { throw 'cargo --version failed.' }
}
finally {
    Pop-Location
}

$lockPath = Join-Path $repositoryRoot 'Cargo.lock'
if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) { throw 'Cargo.lock is missing.' }
$lockHash = (Get-FileHash -LiteralPath $lockPath -Algorithm SHA256).Hash.ToUpperInvariant()

$linkerInfo = [ordered]@{
    status = 'unavailable'
    name = $null
    version = $null
    detected_from = 'environment-probe'
    usage_proven = $false
}
$linkerCandidates = @()
$vsDevCmdPath = $null
if (-not [string]::IsNullOrWhiteSpace($env:VCToolsInstallDir)) {
    $linkerCandidates += Join-Path $env:VCToolsInstallDir 'bin\Hostx64\x64\link.exe'
    $linkerCandidates += Join-Path $env:VCToolsInstallDir 'bin\Hostx86\x64\link.exe'
}
$vswhereCandidates = @()
if (${env:ProgramFiles(x86)}) { $vswhereCandidates += Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe' }
if ($env:ProgramFiles) { $vswhereCandidates += Join-Path $env:ProgramFiles 'Microsoft Visual Studio\Installer\vswhere.exe' }
foreach ($vswhere in $vswhereCandidates) {
    if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) { continue }
    $visualStudioPath = (& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null | Select-Object -First 1)
    if ([string]::IsNullOrWhiteSpace([string]$visualStudioPath)) { continue }
    $candidateDevCmd = Join-Path $visualStudioPath 'Common7\Tools\VsDevCmd.bat'
    if (Test-Path -LiteralPath $candidateDevCmd -PathType Leaf) { $vsDevCmdPath = $candidateDevCmd }
    $msvcRoot = Join-Path $visualStudioPath 'VC\Tools\MSVC'
    if (Test-Path -LiteralPath $msvcRoot -PathType Container) {
        foreach ($toolset in @(Get-ChildItem -LiteralPath $msvcRoot -Directory | Sort-Object Name -Descending)) {
            $candidateLinker = Join-Path $toolset.FullName 'bin\Hostx64\x64\link.exe'
            if (Test-Path -LiteralPath $candidateLinker -PathType Leaf) {
                if ($linkerCandidates -notcontains $candidateLinker) { $linkerCandidates += $candidateLinker }
                break
            }
        }
    }
    break
}
foreach ($directory in ($env:Path -split [IO.Path]::PathSeparator)) {
    if ([string]::IsNullOrWhiteSpace($directory) -or $directory.StartsWith('\\')) { continue }
    $candidate = Join-Path $directory 'link.exe'
    if ($linkerCandidates -notcontains $candidate) { $linkerCandidates += $candidate }
}
foreach ($candidate in $linkerCandidates) {
    if (Test-Path -LiteralPath $candidate -PathType Leaf) {
        $versionInfo = (Get-Item -LiteralPath $candidate).VersionInfo
        if (-not [string]::IsNullOrWhiteSpace($versionInfo.FileVersion)) {
            $linkerInfo = [ordered]@{
                status = 'available'
                name = 'link.exe (MSVC linker)'
                version = $versionInfo.FileVersion
                detected_from = 'environment-probe'
                usage_proven = $false
            }
            break
        }
    }
}

$previousEncodedFlags = [Environment]::GetEnvironmentVariable('CARGO_ENCODED_RUSTFLAGS', 'Process')
$previousRustFlags = [Environment]::GetEnvironmentVariable('RUSTFLAGS', 'Process')
$previousTargetDir = [Environment]::GetEnvironmentVariable('CARGO_TARGET_DIR', 'Process')
$previousTargetLinker = [Environment]::GetEnvironmentVariable('CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER', 'Process')
if (-not [string]::IsNullOrWhiteSpace($previousTargetLinker)) {
    throw 'CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER is set; refusing an unrecorded linker override.'
}

$normalizedRustflags = @()
$encodedRustflags = @()
if ($BuildMode -in @('brepro', 'brepro-and-path-remap')) {
    $normalizedRustflags += @('-C', 'link-arg=/Brepro')
    $encodedRustflags += @('-C', 'link-arg=/Brepro')
}
if ($BuildMode -in @('path-remap', 'brepro-and-path-remap')) {
    $normalizedRustflags += @('--remap-path-prefix=<repository-root>=/encryptdrive')
    $encodedRustflags += "--remap-path-prefix=$repositoryRoot=/encryptdrive"
    $cargoHome = if (-not [string]::IsNullOrWhiteSpace($env:CARGO_HOME)) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
    $normalizedRustflags += @('--remap-path-prefix=<cargo-home>=/.cargo')
    $encodedRustflags += "--remap-path-prefix=$cargoHome=/.cargo"
}

$null = New-Item -ItemType Directory -Path $runRoot -Force
$artifacts = [Collections.Generic.List[object]]::new()
$oldRustflagsPresent = Test-Path Env:RUSTFLAGS
$oldEncodedPresent = Test-Path Env:CARGO_ENCODED_RUSTFLAGS
Push-Location -LiteralPath $repositoryRoot
try {
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
    $env:CARGO_ENCODED_RUSTFLAGS = [string]::Join([char]31, $encodedRustflags)

    foreach ($variant in @('shell', 'design-preview')) {
        $variantTarget = Join-Path $cargoTargetRoot $variant
        $variantDir = Join-Path $runRoot $variant
        Assert-NoReparsePointPath -Path $variantTarget -Root $repositoryRoot -Context "Cargo target '$variant'"
        Assert-NoReparsePointPath -Path $variantDir -Root $runRoot -Context "Artifact variant '$variant'"
        $null = New-Item -ItemType Directory -Path $variantDir -Force
        $null = New-Item -ItemType Directory -Path $cargoTargetRoot -Force
        $env:CARGO_TARGET_DIR = $variantTarget

        if (Test-Path -LiteralPath $variantTarget -PathType Container) {
            $cleanArgs = @('clean', '--package', 'encryptdrive-desktop', '--release', '--target', $target, '--quiet')
            if ($vsDevCmdPath) {
                $cleanCommand = 'call "' + $vsDevCmdPath + '" -arch=x64 -host_arch=x64 >nul && cargo ' + ($cleanArgs -join ' ')
                & $env:ComSpec /d /s /c $cleanCommand 2>&1 | Out-Null
            }
            else {
                & cargo @cleanArgs 2>&1 | Out-Null
            }
            if ($LASTEXITCODE -ne 0) { throw "Could not isolate the '$variant' package outputs before building." }
        }

        $cargoArgs = @('build', '--package', 'encryptdrive-desktop', '--release', '--locked', '--target', $target, '--verbose')
        if ($variant -eq 'design-preview') {
            $cargoArgs += @('--no-default-features', '--features', 'design-preview')
        }
        else {
            $cargoArgs += '--no-default-features'
        }

        $logPath = Join-Path $variantDir 'cargo-build.log'
        Write-Output "Building $variant ($BuildMode) in its isolated Cargo target directory..."
        if ($vsDevCmdPath) {
            $cargoCommand = 'call "' + $vsDevCmdPath + '" -arch=x64 -host_arch=x64 >nul && cargo ' + ($cargoArgs -join ' ')
            & $env:ComSpec /d /s /c $cargoCommand 2>&1 | Tee-Object -FilePath $logPath | Out-Null
        }
        else {
            & cargo @cargoArgs 2>&1 | Tee-Object -FilePath $logPath | Out-Null
        }
        if ($LASTEXITCODE -ne 0) {
            $tail = (Get-Content -LiteralPath $logPath -Tail 60 | Out-String)
            throw "Cargo failed for variant '$variant'. Last output:`n$tail"
        }

        $fingerprintFiles = @(Get-ChildItem -LiteralPath (Join-Path $variantTarget "$target\release\.fingerprint") -Filter 'bin-encryptdrive-desktop.json' -File -Recurse -ErrorAction SilentlyContinue)
        if ($fingerprintFiles.Count -ne 1) { throw "Expected one Cargo binary feature fingerprint for '$variant'; found $($fingerprintFiles.Count)." }
        $fingerprint = Get-Content -LiteralPath $fingerprintFiles[0].FullName -Raw | ConvertFrom-Json
        $actualFeatures = @()
        if (-not [string]::IsNullOrWhiteSpace([string]$fingerprint.features)) {
            $actualFeatures = @(ConvertFrom-Json ([string]$fingerprint.features))
        }
        $expectedActualFeatures = if ($variant -eq 'design-preview') { @('design-preview') } else { @() }
        if (($actualFeatures -join '|') -cne ($expectedActualFeatures -join '|')) {
            throw "Cargo built '$variant' with features [$($actualFeatures -join ', ')], expected [$($expectedActualFeatures -join ', ')]."
        }

        $sourceBinary = Join-Path $variantTarget "$target\release\encryptdrive-desktop.exe"
        if (-not (Test-Path -LiteralPath $sourceBinary -PathType Leaf)) { throw "Cargo did not produce expected output: $sourceBinary" }
        $shortRevision = $revision.Substring(0, 12)
        $artifactName = "EncryptDrive-$variant-$shortRevision-$RunId.exe"
        $artifactPath = Join-Path $variantDir $artifactName
        if (Test-Path -LiteralPath $artifactPath) { throw "Refusing to overwrite existing artifact '$artifactName'." }
        Copy-Item -LiteralPath $sourceBinary -Destination $artifactPath

        $artifactHash = (Get-FileHash -LiteralPath $artifactPath -Algorithm SHA256).Hash.ToUpperInvariant()
        $artifactSize = (Get-Item -LiteralPath $artifactPath).Length
        $peMetadata = Get-PeBuildMetadata -Path $artifactPath
        $relativePath = $artifactPath.Substring($repositoryPrefix.Length).Replace('\', '/')
        $cargoTargetRelative = $variantTarget.Substring($repositoryPrefix.Length).Replace('\', '/')
        $artifacts.Add([ordered]@{
            variant = $variant
            features = $actualFeatures
            relative_path = $relativePath
            cargo_target_relative = $cargoTargetRelative
            size_bytes = [long]$artifactSize
            sha256 = $artifactHash
            profile = 'release'
            target = $target
            pe_metadata = $peMetadata
        })
    }

    $manifest = [ordered]@{
        schema_version = 2
        run_id = $RunId
        build_mode = $BuildMode
        source = [ordered]@{ revision = $revision.ToLowerInvariant(); dirty = [bool]$dirty }
        environment = [ordered]@{
            target = $target
            rustc_version = $rustcVersion
            rustc_commit_hash = $rustcCommit
            rustc_host = $rustcHost
            cargo_version = $cargoVersion
            profile = 'release'
            cargo_lock_sha256 = $lockHash
            rustflags = $normalizedRustflags
            linker = $linkerInfo
        }
        artifacts = @($artifacts.ToArray())
    }
    $temporaryManifest = $manifestPath + '.tmp'
    $manifest | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $temporaryManifest -Encoding utf8
    Move-Item -LiteralPath $temporaryManifest -Destination $manifestPath
}
finally {
    if ($oldRustflagsPresent) { [Environment]::SetEnvironmentVariable('RUSTFLAGS', $previousRustFlags, 'Process') }
    else { Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue }
    if ($oldEncodedPresent) { [Environment]::SetEnvironmentVariable('CARGO_ENCODED_RUSTFLAGS', $previousEncodedFlags, 'Process') }
    else { Remove-Item Env:CARGO_ENCODED_RUSTFLAGS -ErrorAction SilentlyContinue }
    if ($null -ne $previousTargetDir) { [Environment]::SetEnvironmentVariable('CARGO_TARGET_DIR', $previousTargetDir, 'Process') }
    else { Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue }
    Pop-Location
}

$verified = Test-BuildArtifactManifest -ManifestPath $manifestPath -RepositoryRoot $repositoryRoot
Write-Output "Verified build manifest: target/phase-1b/artifacts/$RunId/manifest.json"
Write-Output "Source $revision (dirty=$dirty); mode=$BuildMode; variants=$($verified.artifact_count)."
Write-Output "Raw artifact files and hashes are recorded in the generated manifest."
