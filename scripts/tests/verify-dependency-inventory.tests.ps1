#requires -Version 7.0

$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot '..\DependencyInventory.psm1') -Force

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
        if (-not $_.Exception.Message.Contains($ExpectedMessage, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Unexpected rejection for '$Case': $($_.Exception.Message)"
        }
        return
    }
    throw "Expected '$Case' to be rejected."
}

function New-TestInventory {
    $eguiId = 'registry:egui@0.37.0:' + ('A' * 64)
    $appId = 'workspace:encryptdrive-desktop@0.1.0'
    $coreId = 'workspace:encryptdrive-core@0.1.0'
    $locked = @(
        [pscustomobject]@{ id = $eguiId; name = 'egui'; version = '0.37.0'; source_kind = 'registry'; source_fingerprint = ('A' * 64); checksum = ('B' * 64); license_expression = 'MIT OR Apache-2.0'; license_file_declared = $false; license_file_value = $null; license_file_value_redacted = $false; license_metadata_status = 'license-expression-declared'; workspace_member = $false }
        [pscustomobject]@{ id = $appId; name = 'encryptdrive-desktop'; version = '0.1.0'; source_kind = 'workspace'; source_fingerprint = $null; checksum = $null; license_expression = $null; license_file_declared = $false; license_file_value = $null; license_file_value_redacted = $false; license_metadata_status = 'license-metadata-missing'; workspace_member = $true }
        [pscustomobject]@{ id = $coreId; name = 'encryptdrive-core'; version = '0.1.0'; source_kind = 'workspace'; source_fingerprint = $null; checksum = $null; license_expression = $null; license_file_declared = $false; license_file_value = $null; license_file_value_redacted = $false; license_metadata_status = 'license-metadata-missing'; workspace_member = $true }
    )
    $resolved = @(
        [pscustomobject]@{ package_id = $appId; dependency_depth = 0; relationship = 'root'; enabled_features = @(); dependencies = @([pscustomobject]@{ package_id = $eguiId; kinds = @('normal'); target_conditions = @($null) }) }
        [pscustomobject]@{ package_id = $eguiId; dependency_depth = 1; relationship = 'direct'; enabled_features = @('default_fonts'); dependencies = @() }
    )
    return [pscustomobject]@{
        schema_version = 1
        target = 'x86_64-pc-windows-msvc'
        lockfile = [pscustomobject]@{ sha256 = ('C' * 64); package_count = 3; packages = $locked }
        configurations = @(
            [pscustomobject]@{ name = 'shell'; requested_features = @(); root_package_id = $appId; resolved_package_count = 2; resolved_packages = $resolved }
            [pscustomobject]@{ name = 'design-preview'; requested_features = @('design-preview'); root_package_id = $appId; resolved_package_count = 2; resolved_packages = $resolved }
        )
        build_evidence = $null
    }
}

$valid = New-TestInventory
Assert-True (Test-DependencyInventoryDocument -Inventory $valid) 'a complete Windows inventory must pass validation'

$missingSchema = New-TestInventory
$missingSchema.PSObject.Properties.Remove('schema_version')
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $missingSchema } 'missing schema version' 'schema_version'

$missingBuildEvidence = New-TestInventory
$missingBuildEvidence.PSObject.Properties.Remove('build_evidence')
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $missingBuildEvidence } 'required build evidence field missing' 'build_evidence'

$wrongCount = New-TestInventory
$wrongCount.lockfile.package_count = 2
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $wrongCount } 'lockfile count mismatch' 'package_count'

$inconsistentLicense = New-TestInventory
$inconsistentLicense.lockfile.packages[0].license_metadata_status = 'license-metadata-missing'
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $inconsistentLicense } 'license metadata does not match recorded fields' 'license_metadata_status'

$missingRegistryChecksum = New-TestInventory
$missingRegistryChecksum.lockfile.packages[0].checksum = $null
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $missingRegistryChecksum } 'registry package missing locked checksum' 'requires a Cargo.lock checksum'

$misclassifiedWorkspace = New-TestInventory
$misclassifiedWorkspace.lockfile.packages[1].workspace_member = $false
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $misclassifiedWorkspace } 'workspace package classification mismatch' 'workspace_member'

$absoluteLicensePath = New-TestInventory
$absoluteLicensePath.lockfile.packages[0].license_file_declared = $true
$absoluteLicensePath.lockfile.packages[0].license_file_value = 'C:\inventory-fixture\LICENSE'
$absoluteLicensePath.lockfile.packages[0].license_metadata_status = 'license-expression-and-license-file-declared'
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $absoluteLicensePath } 'absolute license path must not leak into inventory' 'absolute path'

$redactedLicensePath = New-TestInventory
$redactedLicensePath.lockfile.packages[0].license_file_declared = $true
$redactedLicensePath.lockfile.packages[0].license_file_value_redacted = $true
$redactedLicensePath.lockfile.packages[0].license_metadata_status = 'license-expression-and-license-file-declared'
Assert-True (Test-DependencyInventoryDocument -Inventory $redactedLicensePath) 'absolute license_file metadata must retain declaration status while redacting its machine path'

$unlockedResolvedPackage = New-TestInventory
$unlockedResolvedPackage.configurations[0].resolved_packages[1].package_id = 'registry:unlocked@9.9.9:' + ('D' * 64)
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $unlockedResolvedPackage } 'resolved package missing from lockfile' 'not present in lockfile'

$inconsistentIdentity = New-TestInventory
$inconsistentIdentity.lockfile.packages[0].version = '9.9.9'
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $inconsistentIdentity } 'package id inconsistent with identity fields' 'stable id'

$missingVariant = New-TestInventory
$missingVariant.configurations = @($missingVariant.configurations[0])
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $missingVariant } 'required configuration missing' 'design-preview'

$inconsistentRoot = New-TestInventory
$inconsistentRoot.configurations[0].resolved_packages[0].dependency_depth = 1
$inconsistentRoot.configurations[0].resolved_packages[0].relationship = 'direct'
Assert-Rejected { Test-DependencyInventoryDocument -Inventory $inconsistentRoot } 'root depth inconsistent with root identity' 'root package'

$validatorPath = Join-Path $PSScriptRoot '..\verify-dependency-inventory.ps1'
$malformedPath = Join-Path $env:TEMP "encryptdrive-dependency-inventory-malformed-$PID.json"
try {
    Set-Content -LiteralPath $malformedPath -Value '{"schema_version":' -NoNewline
    $pwshPath = (Get-Command pwsh.exe).Source
    $malformedOutput = & $pwshPath -NoProfile -File $validatorPath -ValidateOnly -InventoryPath $malformedPath 2>&1 | Out-String
    $malformedExit = $LASTEXITCODE
    Assert-True ($malformedExit -ne 0) 'malformed inventory JSON must fail the command-line validator'
    Assert-True ($malformedOutput.Contains('valid JSON', [StringComparison]::OrdinalIgnoreCase)) 'malformed JSON rejection must explain the input problem'
    $global:LASTEXITCODE = 0
}
finally {
    Remove-Item -LiteralPath $malformedPath -Force -ErrorAction SilentlyContinue
}

$validInventoryPath = Join-Path $env:TEMP "encryptdrive-dependency-inventory-valid-$PID-$([guid]::NewGuid().ToString('N')).json"
try {
    [IO.File]::WriteAllText($validInventoryPath, (($valid | ConvertTo-Json -Depth 60) + [Environment]::NewLine), [Text.UTF8Encoding]::new($false))
    $validateOutput = & $pwshPath -NoProfile -File $validatorPath -ValidateOnly -InventoryPath $validInventoryPath 2>&1 | Out-String
    $validateExit = $LASTEXITCODE
    Assert-True ($validateExit -eq 0) 'a structurally consistent inventory must pass ValidateOnly'
    Assert-True ($validateOutput.Contains('Structurally valid dependency inventory', [StringComparison]::Ordinal)) 'ValidateOnly success must identify its limited structural validation scope'
    Assert-True (-not $validateOutput.Contains('Verified dependency inventory', [StringComparison]::Ordinal)) 'ValidateOnly must not claim independent verification'
}
finally {
    Remove-Item -LiteralPath $validInventoryPath -Force -ErrorAction SilentlyContinue
}

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Push-Location -LiteralPath $env:TEMP
try { $generated = New-DependencyInventory -RepositoryRoot $repositoryRoot }
finally { Pop-Location }
$lockEntries = ([regex]::Matches((Get-Content -LiteralPath (Join-Path $repositoryRoot 'Cargo.lock') -Raw), '\[\[package\]\]')).Count
Assert-True ($generated.lockfile.package_count -eq $lockEntries) 'generated locked package inventory must match Cargo.lock entry count'
Assert-True ((@($generated.configurations | ForEach-Object { $_.name } | Sort-Object) -join '|') -ceq 'design-preview|shell') 'generated inventory must contain both Windows application configurations'
Assert-True (@($generated.configurations | Where-Object { $_.resolved_package_count -gt 0 }).Count -eq 2) 'each Windows configuration must resolve the application dependency graph'
$serialized = $generated | ConvertTo-Json -Depth 60
Assert-True (-not $serialized.Contains($env:USERPROFILE, [StringComparison]::OrdinalIgnoreCase)) 'generated inventory must not expose a machine-specific Cargo or repository path'

Write-Output 'PASS: structural ValidateOnly wording and limitation, valid inventory, malformed JSON, missing fields, inconsistent counts/identity/license, missing lock reference/configuration, registry checksum, workspace classification, and absolute-path redaction cases.'
