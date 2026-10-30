#requires -Version 7.0

<#
.SYNOPSIS
Generates or structurally validates a Windows dependency inventory.

.DESCRIPTION
Generation reads the current locked Cargo inputs and, when supplied, records build
evidence from the given build manifest. In CI, the build manifest is verified by the
preceding build-artifact validation step.

ValidateOnly checks document structure and internal consistency only. It does not
independently rederive all facts from the current Cargo.lock, Cargo metadata, or
executables, and it does not prove inventory integrity, authenticity, or equivalence
to current source/build inputs.

.PARAMETER ValidateOnly
Validate the inventory document's structure and internal consistency without
regenerating or independently corroborating it against current source/build inputs.

.PARAMETER InventoryPath
Path to an existing inventory document; required with ValidateOnly.

.NOTES
Use the normal generation path, together with the verified build-manifest workflow,
when current Cargo and build evidence is required.
#>
[CmdletBinding()]
param(
    [switch] $ValidateOnly,
    [string] $InventoryPath,
    [string] $ManifestPath,
    [switch] $RequireNativeImports,
    [string] $RepositoryRoot = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'DependencyInventory.psm1') -Force

try {
    $root = (Resolve-Path -LiteralPath $RepositoryRoot).Path
    if ($ValidateOnly) {
        if ([string]::IsNullOrWhiteSpace($InventoryPath) -or -not (Test-Path -LiteralPath $InventoryPath -PathType Leaf)) {
            throw 'ValidateOnly requires an existing InventoryPath.'
        }
        try { $inventory = Get-Content -LiteralPath $InventoryPath -Raw | ConvertFrom-Json }
        catch { throw "Inventory is not valid JSON: $($_.Exception.Message)" }
        $null = Test-DependencyInventoryDocument -Inventory $inventory
        Write-Output "Structurally valid dependency inventory: $($inventory.lockfile.package_count) lockfile package records; shell=$(@($inventory.configurations | Where-Object name -eq 'shell').resolved_package_count), design-preview=$(@($inventory.configurations | Where-Object name -eq 'design-preview').resolved_package_count) Windows-resolved packages."
        Write-Output 'ValidateOnly checks document structure and internal consistency only; it does not prove integrity, authenticity, or equivalence to current Cargo/build inputs.'
        if ($null -ne $inventory.build_evidence) { Write-Output "Build evidence: $($inventory.build_evidence.source_revision), native imports=$($inventory.build_evidence.native_import_status)." }
        exit 0
    }

    if (-not [string]::IsNullOrWhiteSpace($InventoryPath)) { throw 'InventoryPath is used only with ValidateOnly; generated output is kept under ignored target/.' }
    if ($RequireNativeImports -and [string]::IsNullOrWhiteSpace($ManifestPath)) { throw 'RequireNativeImports requires a verified build ManifestPath.' }

    $manifestRunId = $null
    if (-not [string]::IsNullOrWhiteSpace($ManifestPath)) {
        $resolvedManifestPath = if ([IO.Path]::IsPathRooted($ManifestPath)) { $ManifestPath } else { Join-Path $root $ManifestPath }
        if (-not (Test-Path -LiteralPath $resolvedManifestPath -PathType Leaf)) { throw "Build ManifestPath does not exist: $ManifestPath" }
        try { $manifestPreview = Get-Content -LiteralPath $resolvedManifestPath -Raw | ConvertFrom-Json }
        catch { throw "Build manifest is not valid JSON: $($_.Exception.Message)" }
        $manifestRunId = [string]$manifestPreview.run_id
        if ($manifestRunId -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$' -or $manifestRunId -in @('.', '..')) { throw 'Build manifest has an invalid run_id.' }
    }
    else {
        $manifestRunId = 'metadata-only-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + "-$PID"
    }

    $outputDirectory = Join-Path $root 'target\phase-1b\dependency-inventory'
    $outputPath = Join-Path $outputDirectory "$manifestRunId.json"
    if (Test-Path -LiteralPath $outputPath) { throw "Refusing to overwrite existing dependency inventory '$manifestRunId.json'." }

    $inventory = New-DependencyInventory -RepositoryRoot $root -ManifestPath $resolvedManifestPath
    if ($RequireNativeImports -and [string]$inventory.build_evidence.native_import_status -cne 'collected') {
        throw "Native import collection is required but reported '$($inventory.build_evidence.native_import_status)'."
    }
    $null = Test-DependencyInventoryDocument -Inventory $inventory

    $null = New-Item -ItemType Directory -Path $outputDirectory -Force
    Import-Module (Join-Path $PSScriptRoot 'BuildArtifacts.psm1') -Force
    Assert-NoReparsePointPath -Path $outputDirectory -Root $root -Context 'Dependency inventory output'
    $repoSafe = 'safe.directory=' + $root
    & git -c $repoSafe -C $root check-ignore --quiet -- 'target/phase-1b/dependency-inventory'
    if ($LASTEXITCODE -ne 0) { throw 'Generated dependency inventory location is not ignored by Git.' }

    $json = $inventory | ConvertTo-Json -Depth 60
    [IO.File]::WriteAllText($outputPath, $json + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
    $reloaded = Get-Content -LiteralPath $outputPath -Raw | ConvertFrom-Json
    $null = Test-DependencyInventoryDocument -Inventory $reloaded
    Write-Output "Wrote verified dependency inventory under target/phase-1b/dependency-inventory/$manifestRunId.json."
    Write-Output "Cargo.lock entries: $($reloaded.lockfile.package_count); Windows-resolved packages: shell=$(@($reloaded.configurations | Where-Object name -eq 'shell').resolved_package_count), design-preview=$(@($reloaded.configurations | Where-Object name -eq 'design-preview').resolved_package_count)."
    if ($null -ne $reloaded.build_evidence) {
        Write-Output "Build source: $($reloaded.build_evidence.source_revision) (dirty=$($reloaded.build_evidence.source_dirty)); native imports=$($reloaded.build_evidence.native_import_status)."
        foreach ($variant in $reloaded.build_evidence.variants) {
            Write-Output "  $($variant.name): $($variant.executable_size_bytes) bytes; SHA-256 $($variant.executable_sha256); observed PE imports=$(@($variant.static_pe_imports).Count); compiled-package observations=$(@($variant.compiled_package_observations).Count)."
        }
    }
    else { Write-Output 'Native imports and compiled package observations: NOT COLLECTED (no verified build manifest supplied).' }
}
catch {
    [Console]::Error.WriteLine("Dependency inventory verification failed: $($_.Exception.Message)")
    exit 1
}
