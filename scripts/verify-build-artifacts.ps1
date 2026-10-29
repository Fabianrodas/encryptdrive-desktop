#requires -Version 7.0

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)] [string] $ManifestPath,
    [string] $RepositoryRoot = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'BuildArtifacts.psm1') -Force

try {
    $result = Test-BuildArtifactManifest -ManifestPath $ManifestPath -RepositoryRoot $RepositoryRoot
    Write-Output "Verified $($result.artifact_count) release variants for run '$($result.run_id)' at $($result.source_revision)."
    foreach ($variant in $result.variants) { Write-Output "  PASS $variant" }
}
catch {
    [Console]::Error.WriteLine("Build artifact verification failed: $($_.Exception.Message)")
    exit 1
}
