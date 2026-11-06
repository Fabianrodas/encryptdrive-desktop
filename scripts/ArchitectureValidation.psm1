function Assert-WorkspaceArchitecture {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [psobject] $Metadata
    )

    $expectedMembers = @(
        "encryptdrive-core",
        "encryptdrive-desktop",
        "vault-crypto",
        "vault-format",
        "vault-storage"
    )
    $actualMembers = @($Metadata.packages | ForEach-Object { $_.name } | Sort-Object)
    if (($actualMembers -join "|") -ne (($expectedMembers | Sort-Object) -join "|")) {
        throw "Unexpected workspace package set: $($actualMembers -join ', ')"
    }

    $expectedDependencies = @{
        "encryptdrive-desktop" = @("eframe", "egui_extras", "encryptdrive-core")
        "encryptdrive-core" = @()
        "vault-format" = @("base64", "serde", "serde_json", "time")
        "vault-crypto" = @("vault-format")
        "vault-storage" = @()
    }

    foreach ($package in $Metadata.packages) {
        $actualDependencies = @($package.dependencies | ForEach-Object { $_.name } | Sort-Object)
        $expected = @($expectedDependencies[$package.name] | Sort-Object)
        if (($actualDependencies -join "|") -ne ($expected -join "|")) {
            throw "Unexpected direct dependencies for $($package.name): $($actualDependencies -join ', ')"
        }
    }

    $approvedVaultFormatDependencies = @{
        "base64" = @{
            req = "=0.23.1"
            features = @("std")
        }
        "serde" = @{
            req = "=1.0.229"
            features = @()
        }
        "serde_json" = @{
            req = "=1.0.154"
            features = @("arbitrary_precision", "std")
        }
        "time" = @{
            req = "=0.3.55"
            features = @("parsing", "std")
        }
    }

    $vaultFormat = $Metadata.packages | Where-Object { $_.name -ceq "vault-format" }
    foreach ($dependencyName in $approvedVaultFormatDependencies.Keys) {
        $dependencyMatches = @($vaultFormat.dependencies | Where-Object { $_.name -ceq $dependencyName })
        if ($dependencyMatches.Count -ne 1) {
            throw "vault-format must declare exactly one $dependencyName dependency."
        }

        $dependency = $dependencyMatches[0]
        $approved = $approvedVaultFormatDependencies[$dependencyName]
        $actualFeatures = @($dependency.features | Sort-Object)
        $expectedFeatures = @($approved.features | Sort-Object)
        if ($dependency.req -cne $approved.req) {
            throw "vault-format $dependencyName must use exact version requirement $($approved.req)."
        }
        if ($dependency.uses_default_features -ne $false) {
            throw "vault-format $dependencyName must disable default features."
        }
        if ($dependency.optional -ne $false) {
            throw "vault-format $dependencyName must be a required direct dependency."
        }
        if ($dependency.source -cne "registry+https://github.com/rust-lang/crates.io-index" -or
            $null -ne $dependency.target -or
            $null -ne $dependency.registry -or
            $null -ne $dependency.rename -or
            $null -ne $dependency.kind) {
            throw "vault-format $dependencyName must be an unrenamed crates.io normal dependency."
        }
        if (($actualFeatures -join "|") -cne ($expectedFeatures -join "|")) {
            throw "vault-format $dependencyName features must be exactly: $($expectedFeatures -join ', ')."
        }
    }

    $vaultCrypto = $Metadata.packages | Where-Object { $_.name -ceq "vault-crypto" }
    $vaultFormatPackage = $Metadata.packages | Where-Object { $_.name -ceq "vault-format" }
    $vaultCryptoFormat = @($vaultCrypto.dependencies | Where-Object { $_.name -ceq "vault-format" })
    if ($vaultCryptoFormat.Count -ne 1) {
        throw "vault-crypto must declare exactly one internal vault-format dependency."
    }
    $approvedVaultCryptoDependency = $vaultCryptoFormat[0]
    if ($approvedVaultCryptoDependency.req -cne "=0.1.0") {
        throw "vault-crypto vault-format must use the exact workspace version requirement =0.1.0."
    }
    if ($null -ne $approvedVaultCryptoDependency.source -or
        $approvedVaultCryptoDependency.uses_default_features -ne $false -or
        $approvedVaultCryptoDependency.optional -ne $false -or
        $null -ne $approvedVaultCryptoDependency.target -or
        $null -ne $approvedVaultCryptoDependency.registry -or
        $null -ne $approvedVaultCryptoDependency.rename -or
        $null -ne $approvedVaultCryptoDependency.kind -or
        @($approvedVaultCryptoDependency.features).Count -ne 0) {
        throw "vault-crypto vault-format must remain a required internal path dependency with default features disabled."
    }
    $expectedVaultFormatPath = [System.IO.Path]::GetFullPath($vaultFormatPackage.manifest_path)
    $expectedVaultFormatPath = [System.IO.Path]::GetDirectoryName($expectedVaultFormatPath)
    $actualVaultFormatPath = [System.IO.Path]::GetFullPath($approvedVaultCryptoDependency.path)
    if (-not [string]::Equals($actualVaultFormatPath, $expectedVaultFormatPath, [StringComparison]::OrdinalIgnoreCase)) {
        throw "vault-crypto must depend on the vault-format package in this workspace."
    }

    Write-Output "Workspace membership, crate boundaries, approved vault-format declarations, and the authorized vault-crypto -> vault-format edge verified."
}

Export-ModuleMember -Function Assert-WorkspaceArchitecture
