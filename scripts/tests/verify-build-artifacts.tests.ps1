#requires -Version 7.0

$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot '..\BuildArtifacts.psm1') -Force

function Assert-True {
    param([bool] $Condition, [string] $Message)
    if (-not $Condition) { throw "ASSERTION FAILED: $Message" }
}

function Assert-Rejected {
    param([scriptblock] $Action, [string] $Case, [string] $ExpectedMessage)
    try {
        & $Action | Out-Null
    }
    catch {
        $actualMessage = [string]$_.Exception.Message
        if (-not $actualMessage.Contains($ExpectedMessage, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Unexpected rejection for '$Case'. Expected message containing '$ExpectedMessage'; received '$actualMessage'."
        }
        return
    }
    throw "Expected rejection for '$Case', but the action succeeded."
}

function Copy-TestManifest {
    param([Parameter(Mandatory = $true)] $Value)
    return $Value | ConvertTo-Json -Depth 10 | ConvertFrom-Json
}

function Write-TestManifest {
    param(
        [Parameter(Mandatory = $true)] $Value,
        [Parameter(Mandatory = $true)] [string] $Name
    )
    $path = Join-Path $artifactRoot $Name
    $Value | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $path -Encoding utf8
    return $path
}

function Set-TestArtifactBytes {
    param(
        [Parameter(Mandatory = $true)] $Value,
        [Parameter(Mandatory = $true)] [string] $Variant,
        [Parameter(Mandatory = $true)] [byte[]] $Bytes
    )
    $entry = @($Value.artifacts | Where-Object { $_.variant -ceq $Variant })[0]
    $path = Join-Path $testRoot ($entry.relative_path -replace '/', [string][IO.Path]::DirectorySeparatorChar)
    [IO.File]::WriteAllBytes($path, $Bytes)
    $entry.size_bytes = [long]$Bytes.Length
    $entry.sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToUpperInvariant()
    return $entry
}

$realRepositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path
$testWorkspaceRoot = Join-Path $realRepositoryRoot 'target\phase-1b\test-workspaces'
$testRoot = Join-Path $testWorkspaceRoot ('manifest-verifier-' + [guid]::NewGuid().ToString('N'))
if (-not $testRoot.StartsWith($testWorkspaceRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Manifest test workspace must remain below ignored target/phase-1b/test-workspaces.'
}
try {
    $null = New-Item -ItemType Directory -Path $testRoot -Force
    $runId = 'manifest-tests'
    $revision = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
    $shortRevision = $revision.Substring(0, 12)
    $artifactRoot = Join-Path $testRoot "target\phase-1b\artifacts\$runId"
    $shellDirectory = Join-Path $artifactRoot 'shell'
    $previewDirectory = Join-Path $artifactRoot 'design-preview'
    $null = New-Item -ItemType Directory -Path $shellDirectory -Force
    $null = New-Item -ItemType Directory -Path $previewDirectory -Force
    $null = New-Item -ItemType Directory -Path (Join-Path $testRoot 'target\phase-1b\cargo-target\shell') -Force
    $null = New-Item -ItemType Directory -Path (Join-Path $testRoot 'target\phase-1b\cargo-target\design-preview') -Force
    $shellPath = Join-Path $shellDirectory "EncryptDrive-shell-$shortRevision-$runId.exe"
    $previewPath = Join-Path $previewDirectory "EncryptDrive-design-preview-$shortRevision-$runId.exe"
    $shellBytes = [byte[]](0x45, 0x44, 0x53, 0x48, 0x01)
    $previewBytes = [byte[]]([Text.Encoding]::ASCII.GetBytes('--design-preview') + [byte[]](0x02, 0x03))
    [IO.File]::WriteAllBytes($shellPath, $shellBytes)
    [IO.File]::WriteAllBytes($previewPath, $previewBytes)

    $shellHash = (Get-FileHash -LiteralPath $shellPath -Algorithm SHA256).Hash
    $previewHash = (Get-FileHash -LiteralPath $previewPath -Algorithm SHA256).Hash
    $manifest = [ordered]@{
        schema_version = 2
        run_id = $runId
        build_mode = 'baseline'
        source = [ordered]@{
            revision = $revision
            dirty = $false
        }
        environment = [ordered]@{
            target = 'x86_64-pc-windows-msvc'
            rustc_version = 'rustc 1.99.1 (test fixture)'
            rustc_commit_hash = ('c' * 40)
            rustc_host = 'x86_64-pc-windows-msvc'
            cargo_version = 'cargo 1.99.1 (test fixture)'
            profile = 'release'
            cargo_lock_sha256 = ('b' * 64)
            rustflags = @()
            linker = [ordered]@{
                status = 'unavailable'
                name = $null
                version = $null
                detected_from = 'environment-probe'
                usage_proven = $false
            }
        }
        artifacts = @(
            [ordered]@{
                variant = 'shell'
                features = @()
                relative_path = "target/phase-1b/artifacts/$runId/shell/EncryptDrive-shell-$shortRevision-$runId.exe"
                cargo_target_relative = 'target/phase-1b/cargo-target/shell'
                size_bytes = (Get-Item -LiteralPath $shellPath).Length
                sha256 = $shellHash
                profile = 'release'
                target = 'x86_64-pc-windows-msvc'
                pe_metadata = [ordered]@{ coff_timestamp_hex = '0x11111111'; codeview_pdb_file = $null; codeview_guid = $null; codeview_age = $null }
            },
            [ordered]@{
                variant = 'design-preview'
                features = @('design-preview')
                relative_path = "target/phase-1b/artifacts/$runId/design-preview/EncryptDrive-design-preview-$shortRevision-$runId.exe"
                cargo_target_relative = 'target/phase-1b/cargo-target/design-preview'
                size_bytes = (Get-Item -LiteralPath $previewPath).Length
                sha256 = $previewHash
                profile = 'release'
                target = 'x86_64-pc-windows-msvc'
                pe_metadata = [ordered]@{ coff_timestamp_hex = '0x22222222'; codeview_pdb_file = $null; codeview_guid = $null; codeview_age = $null }
            }
        )
    }

    $manifestPath = Join-Path $artifactRoot 'manifest.json'
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding utf8
    $valid = Test-BuildArtifactManifest -ManifestPath $manifestPath -RepositoryRoot $testRoot
    Assert-True ($valid.artifact_count -eq 2) 'A valid manifest reports both variants.'
    Write-Output 'PASS: accepts a complete manifest whose hashes and sizes match its files.'

    $missingVariant = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $missingVariant.artifacts = @($missingVariant.artifacts | Where-Object { $_.variant -ne 'design-preview' })
    $missingPath = Join-Path $artifactRoot 'missing-variant.json'
    $missingVariant | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $missingPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $missingPath -RepositoryRoot $testRoot } 'missing variant' 'exactly two build variants'
    Write-Output 'PASS: rejects a missing build variant.'

    $wrongFeatures = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongFeatures.artifacts[0].features = @('design-preview')
    $wrongFeaturesPath = Join-Path $artifactRoot 'wrong-features.json'
    $wrongFeatures | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongFeaturesPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongFeaturesPath -RepositoryRoot $testRoot } 'feature mismatch' 'wrong feature set'
    Write-Output 'PASS: rejects preview features on the default shell artifact.'

    $wrongTarget = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongTarget.artifacts[1].cargo_target_relative = $wrongTarget.artifacts[0].cargo_target_relative
    $wrongTargetPath = Join-Path $artifactRoot 'wrong-cargo-target.json'
    $wrongTarget | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongTargetPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongTargetPath -RepositoryRoot $testRoot } 'shared cargo target directory' 'isolated Cargo target directory'
    Write-Output 'PASS: rejects shell and preview sharing a Cargo target directory.'

    $wrongMode = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongMode.build_mode = 'brepro'
    $wrongModePath = Join-Path $artifactRoot 'wrong-build-mode.json'
    $wrongMode | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongModePath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongModePath -RepositoryRoot $testRoot } 'build mode/flags mismatch' 'do not match build_mode'
    Write-Output 'PASS: rejects build flags that do not match the declared experiment mode.'

    $wrongHash = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongHash.artifacts[0].sha256 = ('0' * 64)
    $wrongHashPath = Join-Path $artifactRoot 'wrong-hash.json'
    $wrongHash | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongHashPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongHashPath -RepositoryRoot $testRoot } 'hash mismatch' 'SHA-256 does not match'
    Write-Output 'PASS: rejects a hash that does not match the executable bytes.'

    $missingHash = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $missingHash.artifacts[1].sha256 = ''
    $missingHashPath = Join-Path $artifactRoot 'missing-hash.json'
    $missingHash | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $missingHashPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $missingHashPath -RepositoryRoot $testRoot } 'missing hash' 'missing a full SHA-256 value'
    Write-Output 'PASS: rejects an artifact without a full SHA-256 value.'

    $wrongSize = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongSize.artifacts[1].size_bytes = [long]$wrongSize.artifacts[1].size_bytes + 1
    $wrongSizePath = Join-Path $artifactRoot 'wrong-size.json'
    $wrongSize | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongSizePath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongSizePath -RepositoryRoot $testRoot } 'size mismatch' 'size_bytes does not match'
    Write-Output 'PASS: rejects a size that does not match the executable bytes.'

    $traversal = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $traversal.artifacts[0].relative_path = 'target/phase-1b/artifacts/manifest-tests/../../../../outside.exe'
    $traversalPath = Join-Path $artifactRoot 'path-traversal.json'
    $traversal | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $traversalPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $traversalPath -RepositoryRoot $testRoot } 'path traversal' 'no traversal segments'
    Write-Output 'PASS: rejects path traversal outside the run artifact directory.'

    $absolute = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $absolute.artifacts[0].relative_path = $shellPath
    $absolutePath = Join-Path $artifactRoot 'absolute-path.json'
    $absolute | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $absolutePath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $absolutePath -RepositoryRoot $testRoot } 'absolute path' 'no traversal segments'
    Write-Output 'PASS: rejects absolute artifact paths.'

    $duplicate = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $duplicate.artifacts[1].relative_path = $duplicate.artifacts[0].relative_path
    $duplicatePath = Join-Path $artifactRoot 'duplicate-path.json'
    $duplicate | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $duplicatePath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $duplicatePath -RepositoryRoot $testRoot } 'duplicate output path' 'share an output path'
    Write-Output 'PASS: rejects variants sharing an output path.'

    $wrongDirectory = Copy-TestManifest $manifest
    $wrongDirectoryArtifact = Join-Path $previewDirectory "EncryptDrive-shell-$shortRevision-$runId.exe"
    [IO.File]::WriteAllBytes($wrongDirectoryArtifact, $shellBytes)
    $wrongDirectory.artifacts[0].relative_path = "target/phase-1b/artifacts/$runId/design-preview/EncryptDrive-shell-$shortRevision-$runId.exe"
    $wrongDirectoryPath = Write-TestManifest $wrongDirectory 'wrong-variant-directory.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongDirectoryPath -RepositoryRoot $testRoot } 'incorrect variant directory' 'does not match its expected artifact path'
    Write-Output 'PASS: rejects a shell artifact stored in the preview directory.'

    $wrongFilename = Copy-TestManifest $manifest
    $wrongFilenameArtifact = Join-Path $shellDirectory "EncryptDrive-shell-$shortRevision-$runId-renamed.exe"
    [IO.File]::WriteAllBytes($wrongFilenameArtifact, $shellBytes)
    $wrongFilename.artifacts[0].relative_path = "target/phase-1b/artifacts/$runId/shell/EncryptDrive-shell-$shortRevision-$runId-renamed.exe"
    $wrongFilenamePath = Write-TestManifest $wrongFilename 'wrong-variant-filename.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongFilenamePath -RepositoryRoot $testRoot } 'incorrect variant filename' 'does not match its expected artifact path'
    Write-Output 'PASS: rejects a noncanonical artifact filename.'

    $swappedEntries = Copy-TestManifest $manifest
    foreach ($property in @('relative_path', 'size_bytes', 'sha256', 'pe_metadata')) {
        $temporary = $swappedEntries.artifacts[0].$property
        $swappedEntries.artifacts[0].$property = $swappedEntries.artifacts[1].$property
        $swappedEntries.artifacts[1].$property = $temporary
    }
    $swappedEntriesPath = Write-TestManifest $swappedEntries 'swapped-entry-fields.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $swappedEntriesPath -RepositoryRoot $testRoot } 'swapped manifest entries with updated paths, hashes, sizes, and PE metadata' 'does not match its expected artifact path'
    Write-Output 'PASS: rejects shell/preview entry substitution with updated paths, hashes, sizes, and PE metadata.'

    $missingManifestPath = Join-Path $artifactRoot 'absent-manifest.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $missingManifestPath -RepositoryRoot $testRoot } 'missing manifest' 'manifest does not exist'
    Write-Output 'PASS: rejects a missing manifest.'

    $corruptManifestPath = Join-Path $artifactRoot 'corrupt-manifest.json'
    Set-Content -LiteralPath $corruptManifestPath -Value '{ invalid json' -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $corruptManifestPath -RepositoryRoot $testRoot } 'corrupt JSON' 'not valid JSON'
    Write-Output 'PASS: rejects corrupt JSON.'

    Remove-Item -LiteralPath $previewPath -Force
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $manifestPath -RepositoryRoot $testRoot } 'missing executable' 'does not exist at its manifest path'
    [IO.File]::WriteAllBytes($previewPath, $previewBytes)
    Write-Output 'PASS: rejects a missing executable.'

    $changedBytes = [byte[]]$shellBytes.Clone()
    $changedBytes[0] = [byte]($changedBytes[0] -bxor 0x01)
    [IO.File]::WriteAllBytes($shellPath, $changedBytes)
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $manifestPath -RepositoryRoot $testRoot } 'modified executable bytes' 'SHA-256 does not match'
    [IO.File]::WriteAllBytes($shellPath, $shellBytes)
    Write-Output 'PASS: rejects modified executable bytes with unchanged manifest metadata.'

    $previewPresentedAsShell = Copy-TestManifest $manifest
    [IO.File]::WriteAllBytes($shellPath, $previewBytes)
    $previewEntry = $previewPresentedAsShell.artifacts[1]
    $previewPresentedAsShell.artifacts[0].pe_metadata = Copy-TestManifest ([pscustomobject]@{ wrapper = $previewEntry.pe_metadata }).wrapper
    $null = Set-TestArtifactBytes $previewPresentedAsShell 'shell' $previewBytes
    $previewPresentedAsShellPath = Write-TestManifest $previewPresentedAsShell 'preview-presented-as-shell.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $previewPresentedAsShellPath -RepositoryRoot $testRoot } 'preview binary presented as shell' "Artifact 'shell' contains marker '--design-preview'"
    [IO.File]::WriteAllBytes($shellPath, $shellBytes)
    Write-Output 'PASS: rejects the preview marker in the shell executable.'

    $shellPresentedAsPreview = Copy-TestManifest $manifest
    [IO.File]::WriteAllBytes($previewPath, $shellBytes)
    $shellPresentedAsPreview.artifacts[1].pe_metadata = Copy-TestManifest ([pscustomobject]@{ wrapper = $shellPresentedAsPreview.artifacts[0].pe_metadata }).wrapper
    $null = Set-TestArtifactBytes $shellPresentedAsPreview 'design-preview' $shellBytes
    $shellPresentedAsPreviewPath = Write-TestManifest $shellPresentedAsPreview 'shell-presented-as-preview.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $shellPresentedAsPreviewPath -RepositoryRoot $testRoot } 'shell binary presented as preview' "Artifact 'design-preview' is missing marker '--design-preview'."
    [IO.File]::WriteAllBytes($previewPath, $previewBytes)
    Write-Output 'PASS: rejects the absence of the preview marker in the design-preview executable.'

    $swappedBinaryPair = Copy-TestManifest $manifest
    [IO.File]::WriteAllBytes($shellPath, $previewBytes)
    [IO.File]::WriteAllBytes($previewPath, $shellBytes)
    $swappedBinaryPair.artifacts[0].pe_metadata = $manifest.artifacts[1].pe_metadata
    $swappedBinaryPair.artifacts[1].pe_metadata = $manifest.artifacts[0].pe_metadata
    $null = Set-TestArtifactBytes $swappedBinaryPair 'shell' $previewBytes
    $null = Set-TestArtifactBytes $swappedBinaryPair 'design-preview' $shellBytes
    $swappedBinaryPairPath = Write-TestManifest $swappedBinaryPair 'swapped-binary-pair.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $swappedBinaryPairPath -RepositoryRoot $testRoot } 'self-consistent swapped shell and preview binaries' "Artifact 'shell' contains marker '--design-preview'"
    [IO.File]::WriteAllBytes($shellPath, $shellBytes)
    [IO.File]::WriteAllBytes($previewPath, $previewBytes)
    Write-Output 'PASS: rejects a self-consistent pair of swapped shell/preview binaries.'

    $unknownTopLevel = Copy-TestManifest $manifest
    Add-Member -InputObject $unknownTopLevel -NotePropertyName unsupported_field -NotePropertyValue 'unexpected'
    $unknownTopLevelPath = Write-TestManifest $unknownTopLevel 'unknown-top-level.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $unknownTopLevelPath -RepositoryRoot $testRoot } 'unknown top-level field' "Unsupported property 'unsupported_field' in manifest"
    Write-Output 'PASS: rejects unsupported top-level manifest fields.'

    $unknownNested = Copy-TestManifest $manifest
    Add-Member -InputObject $unknownNested.environment -NotePropertyName unrecognized_field -NotePropertyValue 'unexpected'
    $unknownNestedPath = Write-TestManifest $unknownNested 'unknown-nested.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $unknownNestedPath -RepositoryRoot $testRoot } 'unknown nested field' "Unsupported property 'unrecognized_field' in environment"
    Write-Output 'PASS: rejects unsupported nested manifest fields.'

    $linkerClaim = Copy-TestManifest $manifest
    $linkerClaim.environment.linker.usage_proven = $true
    $linkerClaimPath = Write-TestManifest $linkerClaim 'linker-use-claim.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $linkerClaimPath -RepositoryRoot $testRoot } 'unproven linker use claim' 'usage_proven must be false'
    Write-Output 'PASS: rejects a linker-use claim that the build driver does not verify.'

    $missingLinkerDisclosure = Copy-TestManifest $manifest
    $null = $missingLinkerDisclosure.environment.linker.PSObject.Properties.Remove('detected_from')
    $missingLinkerDisclosurePath = Write-TestManifest $missingLinkerDisclosure 'missing-linker-disclosure.json'
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $missingLinkerDisclosurePath -RepositoryRoot $testRoot } 'missing linker detection disclosure' 'missing linker.detected_from'
    Write-Output 'PASS: requires explicit linker environment-detection disclosure.'

    $reparseRunId = 'reparse-point-tests'
    $reparseRunRoot = Join-Path $testRoot "target\phase-1b\artifacts\$reparseRunId"
    $reparseOutsideRoot = Join-Path $testRoot 'outside-artifact-run'
    $reparsePreviewDirectory = Join-Path $reparseRunRoot 'design-preview'
    $reparseShellDirectory = Join-Path $reparseRunRoot 'shell'
    $reparseJunction = $reparseShellDirectory
    $reparseShellPath = Join-Path $reparseOutsideRoot "EncryptDrive-shell-$shortRevision-$reparseRunId.exe"
    $reparsePreviewPath = Join-Path $reparsePreviewDirectory "EncryptDrive-design-preview-$shortRevision-$reparseRunId.exe"
    $reparseManifest = Copy-TestManifest $manifest
    $reparseManifest.run_id = $reparseRunId
    $reparseManifest.artifacts[0].relative_path = "target/phase-1b/artifacts/$reparseRunId/shell/EncryptDrive-shell-$shortRevision-$reparseRunId.exe"
    $reparseManifest.artifacts[1].relative_path = "target/phase-1b/artifacts/$reparseRunId/design-preview/EncryptDrive-design-preview-$shortRevision-$reparseRunId.exe"
    $null = New-Item -ItemType Directory -Path $reparseOutsideRoot -Force
    $null = New-Item -ItemType Directory -Path $reparsePreviewDirectory -Force
    [IO.File]::WriteAllBytes($reparseShellPath, $shellBytes)
    [IO.File]::WriteAllBytes($reparsePreviewPath, $previewBytes)
    $reparseManifestPath = Join-Path $reparseRunRoot 'manifest.json'
    $reparseManifest | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $reparseManifestPath -Encoding utf8
    $junctionCreated = $false
    try {
        $null = New-Item -ItemType Junction -Path $reparseJunction -Target $reparseOutsideRoot -ErrorAction Stop
        $junctionCreated = $true
    }
    catch {
        Write-Output "BLOCKED: could not create a Windows junction to exercise reparse rejection ($($_.FullyQualifiedErrorId))."
    }
    if ($junctionCreated) {
        Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $reparseManifestPath -RepositoryRoot $testRoot } 'artifact path through a junction' 'traverses a reparse point'
        Write-Output 'PASS: rejects an artifact path that traverses a junction.'
        Remove-Item -LiteralPath $reparseJunction -Force
    }

    $buildScript = Join-Path $PSScriptRoot '..\build-windows.ps1'
    $overwriteOutput = (& pwsh -NoProfile -File $buildScript -RunId 'manifest-tests' -RepositoryRoot $testRoot 2>&1 | Out-String)
    $overwriteExitCode = $LASTEXITCODE
    Assert-True ($overwriteExitCode -ne 0) 'The build script failed as expected when asked to reuse an existing run ID.'
    Assert-True $overwriteOutput.Contains("Run output already exists; refusing to overwrite 'manifest-tests'.") 'The failed child process reported the expected run ID collision.'
    Assert-True (Test-Path -LiteralPath $shellPath -PathType Leaf) 'An existing artifact remains after the overwrite attempt.'
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: refuses to overwrite a run ID that already has artifacts.'

    Write-Output 'All build-artifact manifest tests passed.'
}
finally {
    if ($junctionCreated -and (Test-Path -LiteralPath $reparseJunction)) {
        Remove-Item -LiteralPath $reparseJunction -Force
    }
    if (Test-Path -LiteralPath $testRoot) {
        $resolvedTestRoot = (Resolve-Path -LiteralPath $testRoot).Path
        if (-not $resolvedTestRoot.StartsWith($testWorkspaceRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Refusing to remove a manifest test workspace outside ignored target/phase-1b/test-workspaces.'
        }
        Remove-Item -LiteralPath $resolvedTestRoot -Recurse -Force
    }
}
