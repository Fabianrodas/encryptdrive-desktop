#requires -Version 7.0

$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path -Parent $PSScriptRoot
Import-Module (Join-Path $PSScriptRoot 'ArchitectureValidation.psm1') -Force
Push-Location $repositoryRoot
try {
    $metadataJson = cargo metadata --no-deps --format-version 1
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }
    $metadata = $metadataJson | ConvertFrom-Json
    Assert-WorkspaceArchitecture -Metadata $metadata
}
finally {
    Pop-Location
}
