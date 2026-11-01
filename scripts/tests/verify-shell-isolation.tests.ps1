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
$unguardedUi.MainSource = [regex]::Replace($unguardedUi.MainSource, '(?s)#\[cfg\(feature = "design-preview"\)\]\s*mod ui;', 'mod ui;')
Assert-Rejected { Assert-ShellPreviewIsolation -Source $unguardedUi } 'UI module compiled without design-preview' 'feature-gated'

$fileIo = Copy-IsolationSnapshot $source
$fileIo.PreviewSources = @($fileIo.PreviewSources | ForEach-Object {
    if ($_.Path.EndsWith('model.rs', [StringComparison]::OrdinalIgnoreCase)) {
        [pscustomobject]@{ Path = $_.Path; Content = $_.Content + "`nstd::fs::write`n" }
    }
    else { $_ }
})
Assert-Rejected { Assert-ShellPreviewIsolation -Source $fileIo } 'preview file I/O' 'forbidden runtime side-effect'

$groupedFileAlias = Copy-IsolationSnapshot $source
$groupedFileAlias.PreviewSources = @($groupedFileAlias.PreviewSources | ForEach-Object {
    if ($_.Path.EndsWith('model.rs', [StringComparison]::OrdinalIgnoreCase)) {
        [pscustomobject]@{ Path = $_.Path; Content = $_.Content + "`nuse std::{fs as fs};`nfs::write`n" }
    }
    else { $_ }
})
Assert-Rejected { Assert-ShellPreviewIsolation -Source $groupedFileAlias } 'grouped aliased preview file I/O' 'forbidden runtime side-effect'

$aliasedUiFileIo = Copy-IsolationSnapshot $source
$aliasedUiFileIo.UiSources = @($aliasedUiFileIo.UiSources | ForEach-Object {
    if ($_.Path.EndsWith('app.rs', [StringComparison]::OrdinalIgnoreCase)) {
        [pscustomobject]@{ Path = $_.Path; Content = $_.Content + "`nuse std::fs as fs;`nfs::write`n" }
    }
    else { $_ }
})
Assert-Rejected { Assert-ShellPreviewIsolation -Source $aliasedUiFileIo } 'aliased preview UI file I/O' 'forbidden runtime side-effect'

$coreCall = Copy-IsolationSnapshot $source
$coreCall.UiSources = @($coreCall.UiSources | ForEach-Object {
    if ($_.Path.EndsWith('app.rs', [StringComparison]::OrdinalIgnoreCase)) {
        [pscustomobject]@{ Path = $_.Path; Content = $_.Content + "`nencryptdrive_core::readiness`n" }
    }
    else { $_ }
})
Assert-Rejected { Assert-ShellPreviewIsolation -Source $coreCall } 'preview core capability call' 'forbidden runtime side-effect'

$aliasedCoreCall = Copy-IsolationSnapshot $source
$aliasedCoreCall.UiSources = @($aliasedCoreCall.UiSources | ForEach-Object {
    if ($_.Path.EndsWith('app.rs', [StringComparison]::OrdinalIgnoreCase)) {
        [pscustomobject]@{ Path = $_.Path; Content = $_.Content + "`nuse encryptdrive_core as core;`ncore::readiness`n" }
    }
    else { $_ }
})
Assert-Rejected { Assert-ShellPreviewIsolation -Source $aliasedCoreCall } 'aliased preview core call' 'forbidden runtime side-effect'

$rootPreviewIo = Copy-IsolationSnapshot $source
$rootPreviewIo.PreviewCatalog = $rootPreviewIo.PreviewCatalog + "`nstd::fs::write`n"
Assert-Rejected { Assert-ShellPreviewIsolation -Source $rootPreviewIo } 'preview catalog file I/O' 'forbidden runtime side-effect'

$aliasedMainCore = Copy-IsolationSnapshot $source
$aliasedMainCore.MainSource = $aliasedMainCore.MainSource + "`nuse encryptdrive_core as core;`ncore::readiness`n"
Assert-Rejected { Assert-ShellPreviewIsolation -Source $aliasedMainCore } 'aliased core call in application entry source' 'forbidden runtime side-effect'

$shellFileIo = Copy-IsolationSnapshot $source
$shellFileIo.ShellSource = $shellFileIo.ShellSource + "`nstd::fs::read`n"
Assert-Rejected { Assert-ShellPreviewIsolation -Source $shellFileIo } 'production shell file I/O' 'forbidden runtime side-effect'

$unsafeFontPath = Copy-IsolationSnapshot $source
$unsafeFontPath.UiSources = @($unsafeFontPath.UiSources | ForEach-Object {
    if ($_.Path.EndsWith([IO.Path]::Combine('ui', 'icons.rs'), [StringComparison]::OrdinalIgnoreCase)) {
        [pscustomobject]@{ Path = $_.Path; Content = $_.Content.Replace('"segoeui.ttf"', '"../secret.txt"') }
    }
    else { $_ }
})
Assert-Rejected { Assert-ShellPreviewIsolation -Source $unsafeFontPath } 'unbounded system-font path' 'three approved system-font names'

$syntheticShell = Copy-IsolationSnapshot $source
$syntheticShell.ShellSource = $syntheticShell.ShellSource + "`nlet _fixture = PreviewModel::fixture(DatasetSize::OneHundred);`n"
Assert-Rejected { Assert-ShellPreviewIsolation -Source $syntheticShell } 'synthetic model in production shell' 'production shell must not expose synthetic fixtures'

$missingScenario = Copy-IsolationSnapshot $source
$missingScenario.PreviewCatalog = $missingScenario.PreviewCatalog.Replace('ALL: [Self; 22]', 'ALL: [Self; 21]')
Assert-Rejected { Assert-ShellPreviewIsolation -Source $missingScenario } 'incomplete preview scenario catalog' '22 scenarios'

Write-Output 'Shell isolation regression suite passed.'
