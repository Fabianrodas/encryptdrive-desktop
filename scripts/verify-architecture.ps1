#requires -Version 7.0

$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repositoryRoot
try {
    $metadataJson = cargo metadata --no-deps --format-version 1
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }
    $metadata = $metadataJson | ConvertFrom-Json

    $expectedMembers = @(
        "encryptdrive-core",
        "encryptdrive-desktop",
        "vault-crypto",
        "vault-format",
        "vault-storage"
    )
    $actualMembers = @($metadata.packages | ForEach-Object { $_.name } | Sort-Object -Unique)
    if (($actualMembers -join "|") -ne (($expectedMembers | Sort-Object) -join "|")) {
        throw "Unexpected workspace package set: $($actualMembers -join ', ')"
    }

    $expectedDependencies = @{
        "encryptdrive-desktop" = @("eframe", "egui_extras", "encryptdrive-core")
        "encryptdrive-core" = @()
        "vault-format" = @()
        "vault-crypto" = @()
        "vault-storage" = @()
    }

    foreach ($package in $metadata.packages) {
        $actualDependencies = @($package.dependencies | ForEach-Object { $_.name } | Sort-Object -Unique)
        $expected = @($expectedDependencies[$package.name] | Sort-Object -Unique)
        if (($actualDependencies -join "|") -ne ($expected -join "|")) {
            throw "Unexpected direct dependencies for $($package.name): $($actualDependencies -join ', ')"
        }
    }

    Write-Output "Workspace membership and direct dependency boundaries verified."
}
finally {
    Pop-Location
}
