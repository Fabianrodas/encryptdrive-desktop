#requires -Version 7.0

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '..\ShellIsolation.psm1') -Force

function Assert-Rejected {
    param([scriptblock] $Action, [string] $Case, [string] $ExpectedMessage)
    try {
        & $Action | Out-Null
    }
    catch {
        if (-not $_.Exception.Message.Contains($ExpectedMessage, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Unexpected rejection for '$Case': $($_.Exception.Message)"
        }
        Write-Output "PASS: rejects $Case."
        return
    }
    throw "Expected '$Case' to be rejected."
}

function Copy-IsolationSnapshot {
    param([System.Collections.IDictionary] $Snapshot)
    $copy = [ordered]@{}
    foreach ($key in $Snapshot.Keys) { $copy[$key] = $Snapshot[$key] }
    return $copy
}

$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path
$source = Get-ShellPreviewIsolationSource -RepositoryRoot $repositoryRoot
Assert-ShellPreviewIsolation -Source $source
Write-Output 'PASS: current shell and synthetic preview source boundaries.'

$defaultPreview = Copy-IsolationSnapshot $source
$defaultPreview.DesktopManifest = $defaultPreview.DesktopManifest.Replace('default = []', 'default = ["design-preview"]')
Assert-Rejected { Assert-ShellPreviewIsolation -Source $defaultPreview } 'preview enabled by default' 'default features empty'

$unguardedUi = Copy-IsolationSnapshot $source
$unguardedUi.MainSource = $unguardedUi.MainSource.Replace('#[cfg(feature = "design-preview")]' + [Environment]::NewLine + 'mod ui;', 'mod ui;')
Assert-Rejected { Assert-ShellPreviewIsolation -Source $unguardedUi } 'UI module compiled without design-preview' 'feature-gated'

$fileIo = Copy-IsolationSnapshot $source
$fileIo.PreviewSources = @($fileIo.PreviewSources | ForEach-Object {
    if ($_.Path.EndsWith('model.rs', [StringComparison]::OrdinalIgnoreCase)) {
        [pscustomobject]@{ Path = $_.Path; Content = $_.Content + "`nstd::fs::write`n" }
    }
    else { $_ }
})
Assert-Rejected { Assert-ShellPreviewIsolation -Source $fileIo } 'preview file I/O' 'forbidden runtime side-effect'

$coreCall = Copy-IsolationSnapshot $source
$coreCall.UiSources = @($coreCall.UiSources | ForEach-Object {
    if ($_.Path.EndsWith('app.rs', [StringComparison]::OrdinalIgnoreCase)) {
        [pscustomobject]@{ Path = $_.Path; Content = $_.Content + "`nencryptdrive_core::readiness`n" }
    }
    else { $_ }
})
Assert-Rejected { Assert-ShellPreviewIsolation -Source $coreCall } 'preview core capability call' 'forbidden runtime side-effect'

$syntheticShell = Copy-IsolationSnapshot $source
$syntheticShell.ShellSource = $syntheticShell.ShellSource + "`nlet _fixture = PreviewModel::fixture(DatasetSize::OneHundred);`n"
Assert-Rejected { Assert-ShellPreviewIsolation -Source $syntheticShell } 'synthetic model in production shell' 'production shell must not expose synthetic fixtures'

$missingScenario = Copy-IsolationSnapshot $source
$missingScenario.PreviewCatalog = $missingScenario.PreviewCatalog.Replace('ALL: [Self; 22]', 'ALL: [Self; 21]')
Assert-Rejected { Assert-ShellPreviewIsolation -Source $missingScenario } 'incomplete preview scenario catalog' '22 scenarios'

Write-Output 'Shell isolation regression suite passed.'
