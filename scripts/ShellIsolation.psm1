#requires -Version 7.0

Set-StrictMode -Version Latest

function Get-ShellPreviewIsolationSource {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)] [string] $RepositoryRoot)

    $root = (Resolve-Path -LiteralPath $RepositoryRoot -ErrorAction Stop).Path
    $appRoot = Join-Path $root 'apps\encryptdrive-desktop\src'
    $previewRoot = Join-Path $appRoot 'preview'
    $uiRoot = Join-Path $appRoot 'ui'

    foreach ($required in @(
        (Join-Path $root 'apps\encryptdrive-desktop\Cargo.toml'),
        (Join-Path $appRoot 'lib.rs'),
        (Join-Path $appRoot 'main.rs'),
        (Join-Path $appRoot 'shell.rs'),
        $previewRoot,
        $uiRoot
    )) {
        if (-not (Test-Path -LiteralPath $required)) {
            throw "Required shell-isolation input is missing: $required"
        }
    }

    $previewSources = @(Get-ChildItem -LiteralPath $previewRoot -Filter '*.rs' -File -Recurse |
        Sort-Object -Property FullName |
        ForEach-Object { [pscustomobject]@{ Path = $_.FullName; Content = Get-Content -LiteralPath $_.FullName -Raw } })
    $uiSources = @(Get-ChildItem -LiteralPath $uiRoot -Filter '*.rs' -File -Recurse |
        Sort-Object -Property FullName |
        ForEach-Object { [pscustomobject]@{ Path = $_.FullName; Content = Get-Content -LiteralPath $_.FullName -Raw } })

    return [ordered]@{
        DesktopManifest = Get-Content -LiteralPath (Join-Path $root 'apps\encryptdrive-desktop\Cargo.toml') -Raw
        LibrarySource = Get-Content -LiteralPath (Join-Path $appRoot 'lib.rs') -Raw
        MainSource = Get-Content -LiteralPath (Join-Path $appRoot 'main.rs') -Raw
        ShellSource = Get-Content -LiteralPath (Join-Path $appRoot 'shell.rs') -Raw
        PreviewCatalog = Get-Content -LiteralPath (Join-Path $appRoot 'preview.rs') -Raw
        PreviewSources = $previewSources
        UiSources = $uiSources
    }
}

function Assert-ShellPreviewIsolation {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)] [System.Collections.IDictionary] $Source)

    foreach ($name in @('DesktopManifest', 'LibrarySource', 'MainSource', 'ShellSource', 'PreviewCatalog', 'PreviewSources', 'UiSources')) {
        if (-not $Source.Contains($name) -or $null -eq $Source[$name]) {
            throw "Shell-isolation snapshot is missing '$name'."
        }
    }

    $features = [regex]::Match([string]$Source.DesktopManifest, '(?ms)^\[features\]\s*(.*?)(?=^\[|\z)')
    if (-not $features.Success -or
        $features.Groups[1].Value -notmatch '(?m)^\s*default\s*=\s*\[\s*\]\s*$') {
        throw 'The Desktop Cargo manifest must keep default features empty.'
    }
    if ($features.Groups[1].Value -notmatch '(?m)^\s*design-preview\s*=\s*\[\s*\]\s*$') {
        throw 'The design-preview feature must remain explicit and dependency-free.'
    }

    if ([string]$Source.LibrarySource -notmatch '(?s)#\[cfg\(any\(feature\s*=\s*"design-preview"\s*,\s*test\)\)\]\s*pub\s+mod\s+preview\s*;') {
        throw 'The preview model must be gated to design-preview or unit-test compilation.'
    }
    if ([string]$Source.MainSource -notmatch '(?s)#\[cfg\(feature\s*=\s*"design-preview"\)\]\s*mod\s+ui\s*;') {
        throw 'The synthetic UI module must be feature-gated.'
    }
    if ([string]$Source.MainSource -notmatch '(?s)#\[cfg\(feature\s*=\s*"design-preview"\)\]\s*fn\s+run_preview\s*\(') {
        throw 'The preview entry point must be feature-gated.'
    }
    if ([string]$Source.MainSource -notmatch '(?s)#\[cfg\(not\(feature\s*=\s*"design-preview"\)\)\]\s*fn\s+main\s*\(\)\s*->\s*eframe::Result\s*\{\s*run_shell\(\)') {
        throw 'The default binary entry point must start the production shell.'
    }
    if ([string]$Source.ShellSource -match '(?i)\bpreview\b|design-preview|encryptdrive_core|readiness\s*\(') {
        throw 'The production shell must not depend on preview or core readiness behavior.'
    }
    if ([string]$Source.ShellSource -notmatch 'ShellState::new\(\)' -or
        [string]$Source.ShellSource -notmatch 'No vault open' -or
        [string]$Source.ShellSource -notmatch 'Trash is unavailable') {
        throw 'The production shell must start from its empty no-vault state and explain unavailable Trash.'
    }
    if ([string]$Source.ShellSource -match '(?i)PreviewModel|PreviewScenario|OperationState|create_vault|authenticate|import_file|export_file|move_to_trash|restore_file|delete_file') {
        throw 'The production shell must not expose synthetic fixtures or real operation handlers.'
    }

    $scenarioCatalog = [regex]::Match([string]$Source.PreviewCatalog, '(?s)pub\s+const\s+ALL\s*:\s*\[\s*Self\s*;\s*(\d+)\s*\]\s*=\s*\[(.*?)\];')
    if (-not $scenarioCatalog.Success) {
        throw 'The preview scenario catalog must declare its fixed ALL array.'
    }
    $declaredScenarioCount = [int]$scenarioCatalog.Groups[1].Value
    $listedScenarioCount = [regex]::Matches($scenarioCatalog.Groups[2].Value, '\bSelf::[A-Za-z_]\w*').Count
    if ($declaredScenarioCount -ne 22 -or $listedScenarioCount -ne 22) {
        throw "The synthetic preview must keep all 22 scenarios in its catalog; declared=$declaredScenarioCount listed=$listedScenarioCount."
    }

    if (@($Source.PreviewSources).Count -eq 0 -or @($Source.UiSources).Count -eq 0) {
        throw 'Preview and UI source trees cannot be empty.'
    }
    $previewIoPattern = '(?i)\buse\s+(?:std::)?(?:fs|net|process|path)\b|\bstd::(?:fs|net|process|path)\s*::|\b(?:File|OpenOptions)\s*::\s*(?:open|create|new)\s*\(|\b(?:TcpStream|UdpSocket|Command)\s*::\s*new\s*\(|\b(?:encryptdrive_core|vault_format|vault_crypto|vault_storage)\s*::'
    foreach ($file in @($Source.PreviewSources)) {
        if ([string]$file.Content -match $previewIoPattern) {
            throw "Synthetic model source contains a forbidden runtime side-effect or production-crate reference: '$($file.Path)'."
        }
    }

    $uiForbiddenPattern = '(?i)\buse\s+(?:std::)?(?:net|process)\b|\bstd::(?:net|process)\s*::|\b(?:TcpStream|UdpSocket|Command)\s*::\s*new\s*\(|\b(?:encryptdrive_core|vault_format|vault_crypto|vault_storage)\s*::|\b(?:File|OpenOptions)\s*::\s*(?:open|create|new)\s*\(|\bstd::fs\s*::\s*(?:write|remove|rename|create_dir|copy|OpenOptions)\b'
    foreach ($file in @($Source.UiSources)) {
        if ([string]$file.Content -match $uiForbiddenPattern) {
            throw "Preview-only UI contains a forbidden runtime side-effect or production-crate reference: '$($file.Path)'."
        }
        if ($file.Path.EndsWith([IO.Path]::Combine('ui', 'icons.rs'), [StringComparison]::OrdinalIgnoreCase)) {
            $fontReads = [regex]::Matches([string]$file.Content, '(?i)\bstd::fs\s*::\s*([A-Za-z_]\w*)')
            if ($fontReads.Count -ne 1 -or $fontReads[0].Groups[1].Value -cne 'read' -or
                [string]$file.Content -notmatch '(?s)std::env::var_os\("WINDIR"\).*?PathBuf::from\(windows_directory\)\.join\("Fonts"\).*?std::fs::read\(fonts_directory\.join\(filename\)\)') {
                throw "The only allowed UI file read is the bounded Windows system-font load in '$($file.Path)'."
            }
        }
        elseif ([string]$file.Content -match '(?i)\bstd::(?:fs|path)\s*::') {
            throw "Preview-only UI file access is not allowed outside the Windows font loader: '$($file.Path)'."
        }
    }

    $allUiText = (@($Source.UiSources | ForEach-Object { [string]$_.Content }) -join "`n")
    if ([string]$Source.MainSource -notmatch 'EncryptDrive\s+—\s+Design preview' -or
        $allUiText -notmatch 'Sample data\s+—\s+no real file or vault is affected\.') {
        throw 'Preview UI must clearly identify its design-preview and synthetic-data status.'
    }

    return 'Source and feature boundaries verified; release feature identity is verified separately by the build manifest.'
}

Export-ModuleMember -Function Get-ShellPreviewIsolationSource, Assert-ShellPreviewIsolation
