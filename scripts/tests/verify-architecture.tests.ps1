#requires -Version 7.0

$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot '..\ArchitectureValidation.psm1') -Force

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

$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Push-Location $repositoryRoot
try {
    $metadataJson = cargo metadata --no-deps --format-version 1 | Out-String
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

$valid = $metadataJson | ConvertFrom-Json
Assert-True ($null -ne (Assert-WorkspaceArchitecture -Metadata $valid)) 'the approved live workspace must pass'

$unexpected = $metadataJson | ConvertFrom-Json
$vaultFormat = $unexpected.packages | Where-Object { $_.name -ceq 'vault-format' }
$vaultFormat.dependencies = @($vaultFormat.dependencies) + [pscustomobject]@{ name = 'uuid' }
Assert-Rejected { Assert-WorkspaceArchitecture -Metadata $unexpected } 'unexpected vault-format dependency' 'unexpected direct dependencies'

$unpinned = $metadataJson | ConvertFrom-Json
$vaultFormat = $unpinned.packages | Where-Object { $_.name -ceq 'vault-format' }
($vaultFormat.dependencies | Where-Object { $_.name -ceq 'serde_json' }).req = '^1.0.154'
Assert-Rejected { Assert-WorkspaceArchitecture -Metadata $unpinned } 'unpinned serde_json' 'exact version requirement'

$defaultFeatures = $metadataJson | ConvertFrom-Json
$vaultFormat = $defaultFeatures.packages | Where-Object { $_.name -ceq 'vault-format' }
($vaultFormat.dependencies | Where-Object { $_.name -ceq 'base64' }).uses_default_features = $true
Assert-Rejected { Assert-WorkspaceArchitecture -Metadata $defaultFeatures } 'base64 default features enabled' 'disable default features'

$unsafeFeatures = $metadataJson | ConvertFrom-Json
$vaultFormat = $unsafeFeatures.packages | Where-Object { $_.name -ceq 'vault-format' }
($vaultFormat.dependencies | Where-Object { $_.name -ceq 'serde_json' }).features += 'unbounded_depth'
Assert-Rejected { Assert-WorkspaceArchitecture -Metadata $unsafeFeatures } 'unbounded serde_json depth' 'features must be exactly'

$extraLowerDependency = $metadataJson | ConvertFrom-Json
$vaultStorage = $extraLowerDependency.packages | Where-Object { $_.name -ceq 'vault-storage' }
$vaultStorage.dependencies = @($vaultStorage.dependencies) + [pscustomobject]@{ name = 'vault-format' }
Assert-Rejected { Assert-WorkspaceArchitecture -Metadata $extraLowerDependency } 'unauthorized lower-crate edge' 'unexpected direct dependencies'

$missingAuthorizedEdge = $metadataJson | ConvertFrom-Json
$vaultCrypto = $missingAuthorizedEdge.packages | Where-Object { $_.name -ceq 'vault-crypto' }
$vaultCrypto.dependencies = @()
Assert-Rejected { Assert-WorkspaceArchitecture -Metadata $missingAuthorizedEdge } 'missing authorized AAD edge' 'unexpected direct dependencies'

$unpinnedInternalEdge = $metadataJson | ConvertFrom-Json
$vaultCrypto = $unpinnedInternalEdge.packages | Where-Object { $_.name -ceq 'vault-crypto' }
($vaultCrypto.dependencies | Where-Object { $_.name -ceq 'vault-format' }).req = '^0.1.0'
Assert-Rejected { Assert-WorkspaceArchitecture -Metadata $unpinnedInternalEdge } 'unpinned internal vault-format edge' 'exact workspace version requirement'

$externalInternalPath = $metadataJson | ConvertFrom-Json
$vaultCrypto = $externalInternalPath.packages | Where-Object { $_.name -ceq 'vault-crypto' }
($vaultCrypto.dependencies | Where-Object { $_.name -ceq 'vault-format' }).path = (Join-Path $env:TEMP 'unexpected-vault-format')
Assert-Rejected { Assert-WorkspaceArchitecture -Metadata $externalInternalPath } 'unexpected internal package path' 'must depend on the vault-format package in this workspace'

Write-Output 'Architecture validator regression checks passed: approved declarations accepted; unexpected dependency, version, feature, and lower-crate boundary changes rejected.'
