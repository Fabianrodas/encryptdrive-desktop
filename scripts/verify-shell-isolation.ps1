#requires -Version 7.0

[CmdletBinding()]
param([string] $RepositoryRoot = (Split-Path -Parent $PSScriptRoot))

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'ShellIsolation.psm1') -Force

try {
    $source = Get-ShellPreviewIsolationSource -RepositoryRoot $RepositoryRoot
    $result = Assert-ShellPreviewIsolation -Source $source
    Write-Output "PASS: $result"
    Write-Output 'NOTE: cfg(test) may compile preview modules for unit tests; this does not enable them in the production release binary.'
}
catch {
    [Console]::Error.WriteLine("Shell isolation validation failed: $($_.Exception.Message)")
    exit 1
}
