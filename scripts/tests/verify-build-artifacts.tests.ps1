$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot '..\BuildArtifacts.psm1') -Force

function Assert-True {
    param([bool] $Condition, [string] $Message)
    if (-not $Condition) { throw "ASSERTION FAILED: $Message" }
}

function Assert-Rejected {
    param([scriptblock] $Action, [string] $Case)
    $rejected = $false
    try { & $Action | Out-Null }
    catch { $rejected = $true }
    Assert-True $rejected "Expected manifest to be rejected: $Case"
}

$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('encryptdrive-artifact-verifier-' + [guid]::NewGuid().ToString('N'))
try {
    $artifactRoot = Join-Path $testRoot 'target\phase-1b\artifacts\manifest-tests'
    $null = New-Item -ItemType Directory -Path $artifactRoot -Force
    $null = New-Item -ItemType Directory -Path (Join-Path $testRoot 'target\phase-1b\cargo-target\shell') -Force
    $null = New-Item -ItemType Directory -Path (Join-Path $testRoot 'target\phase-1b\cargo-target\design-preview') -Force
    $shellPath = Join-Path $artifactRoot 'EncryptDrive-shell-test.exe'
    $previewPath = Join-Path $artifactRoot 'EncryptDrive-design-preview-test.exe'
    [IO.File]::WriteAllBytes($shellPath, [byte[]](0x45, 0x44, 0x53, 0x48, 0x01))
    [IO.File]::WriteAllBytes($previewPath, [byte[]](0x45, 0x44, 0x50, 0x52, 0x02, 0x03))

    $shellHash = (Get-FileHash -LiteralPath $shellPath -Algorithm SHA256).Hash
    $previewHash = (Get-FileHash -LiteralPath $previewPath -Algorithm SHA256).Hash
    $manifest = [ordered]@{
        schema_version = 1
        run_id = 'manifest-tests'
        build_mode = 'baseline'
        source = [ordered]@{
            revision = ('a' * 40)
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
            }
        }
        artifacts = @(
            [ordered]@{
                variant = 'shell'
                features = @()
                relative_path = 'target/phase-1b/artifacts/manifest-tests/EncryptDrive-shell-test.exe'
                cargo_target_relative = 'target/phase-1b/cargo-target/shell'
                size_bytes = (Get-Item -LiteralPath $shellPath).Length
                sha256 = $shellHash
                profile = 'release'
                target = 'x86_64-pc-windows-msvc'
                pe_metadata = [ordered]@{ coff_timestamp_hex = '0x00000000'; codeview_pdb_file = $null; codeview_guid = $null; codeview_age = $null }
            },
            [ordered]@{
                variant = 'design-preview'
                features = @('design-preview')
                relative_path = 'target/phase-1b/artifacts/manifest-tests/EncryptDrive-design-preview-test.exe'
                cargo_target_relative = 'target/phase-1b/cargo-target/design-preview'
                size_bytes = (Get-Item -LiteralPath $previewPath).Length
                sha256 = $previewHash
                profile = 'release'
                target = 'x86_64-pc-windows-msvc'
                pe_metadata = [ordered]@{ coff_timestamp_hex = '0x00000000'; codeview_pdb_file = $null; codeview_guid = $null; codeview_age = $null }
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
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $missingPath -RepositoryRoot $testRoot } 'missing variant'
    Write-Output 'PASS: rejects a missing build variant.'

    $wrongFeatures = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongFeatures.artifacts[0].features = @('design-preview')
    $wrongFeaturesPath = Join-Path $artifactRoot 'wrong-features.json'
    $wrongFeatures | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongFeaturesPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongFeaturesPath -RepositoryRoot $testRoot } 'feature mismatch'
    Write-Output 'PASS: rejects preview features on the default shell artifact.'

    $wrongTarget = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongTarget.artifacts[1].cargo_target_relative = $wrongTarget.artifacts[0].cargo_target_relative
    $wrongTargetPath = Join-Path $artifactRoot 'wrong-cargo-target.json'
    $wrongTarget | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongTargetPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongTargetPath -RepositoryRoot $testRoot } 'shared cargo target directory'
    Write-Output 'PASS: rejects shell and preview sharing a Cargo target directory.'

    $wrongMode = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongMode.build_mode = 'brepro'
    $wrongModePath = Join-Path $artifactRoot 'wrong-build-mode.json'
    $wrongMode | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongModePath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongModePath -RepositoryRoot $testRoot } 'build mode/flags mismatch'
    Write-Output 'PASS: rejects build flags that do not match the declared experiment mode.'

    $wrongHash = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongHash.artifacts[0].sha256 = ('0' * 64)
    $wrongHashPath = Join-Path $artifactRoot 'wrong-hash.json'
    $wrongHash | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongHashPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongHashPath -RepositoryRoot $testRoot } 'hash mismatch'
    Write-Output 'PASS: rejects a hash that does not match the executable bytes.'

    $missingHash = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $missingHash.artifacts[1].sha256 = ''
    $missingHashPath = Join-Path $artifactRoot 'missing-hash.json'
    $missingHash | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $missingHashPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $missingHashPath -RepositoryRoot $testRoot } 'missing hash'
    Write-Output 'PASS: rejects an artifact without a full SHA-256 value.'

    $wrongSize = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $wrongSize.artifacts[1].size_bytes = [long]$wrongSize.artifacts[1].size_bytes + 1
    $wrongSizePath = Join-Path $artifactRoot 'wrong-size.json'
    $wrongSize | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongSizePath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $wrongSizePath -RepositoryRoot $testRoot } 'size mismatch'
    Write-Output 'PASS: rejects a size that does not match the executable bytes.'

    $traversal = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $traversal.artifacts[0].relative_path = 'target/phase-1b/artifacts/manifest-tests/../../../../outside.exe'
    $traversalPath = Join-Path $artifactRoot 'path-traversal.json'
    $traversal | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $traversalPath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $traversalPath -RepositoryRoot $testRoot } 'path traversal'
    Write-Output 'PASS: rejects path traversal outside the run artifact directory.'

    $absolute = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $absolute.artifacts[0].relative_path = $shellPath
    $absolutePath = Join-Path $artifactRoot 'absolute-path.json'
    $absolute | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $absolutePath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $absolutePath -RepositoryRoot $testRoot } 'absolute path'
    Write-Output 'PASS: rejects absolute artifact paths.'

    $duplicate = $manifest | ConvertTo-Json -Depth 8 | ConvertFrom-Json
    $duplicate.artifacts[1].relative_path = $duplicate.artifacts[0].relative_path
    $duplicatePath = Join-Path $artifactRoot 'duplicate-path.json'
    $duplicate | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $duplicatePath -Encoding utf8
    Assert-Rejected { Test-BuildArtifactManifest -ManifestPath $duplicatePath -RepositoryRoot $testRoot } 'duplicate output path'
    Write-Output 'PASS: rejects variants sharing an output path.'

    $buildScript = Join-Path $PSScriptRoot '..\build-windows.ps1'
    & pwsh -NoProfile -File $buildScript -RunId 'manifest-tests' -RepositoryRoot $testRoot *> $null
    $overwriteExitCode = $LASTEXITCODE
    Assert-True ($overwriteExitCode -ne 0) 'The build script refuses to reuse an existing run ID.'
    Assert-True (Test-Path -LiteralPath $shellPath -PathType Leaf) 'An existing artifact remains after the overwrite attempt.'
    Write-Output 'PASS: refuses to overwrite a run ID that already has artifacts.'

    Write-Output 'All build-artifact manifest tests passed.'
}
finally {
    if (Test-Path -LiteralPath $testRoot) { Remove-Item -LiteralPath $testRoot -Recurse -Force }
}
