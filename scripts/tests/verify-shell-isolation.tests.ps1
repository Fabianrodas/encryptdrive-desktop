#requires -Version 7.0

[CmdletBinding()]
param([string] $TemporaryRoot = [IO.Path]::GetTempPath())

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '..\ShellIsolation.psm1') -Force

$script:RejectedCases = 0

function Assert-Rejected {
    param([scriptblock] $Action, [string] $Case, [string] $ExpectedMessage)
    try {
        & $Action | Out-Null
    }
    catch {
        if (-not $_.Exception.Message.Contains($ExpectedMessage, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Unexpected rejection for '$Case': $($_.Exception.Message)"
        }
        $script:RejectedCases++
        Write-Output "PASS: rejects $Case."
        return
    }
    throw "Expected '$Case' to be rejected."
}

function Copy-IsolationSnapshot {
    param([System.Collections.IDictionary] $Snapshot)
    return [ordered]@{
        DesktopManifest = [string]$Snapshot.DesktopManifest
        Files = @($Snapshot.Files | ForEach-Object { [pscustomobject]@{ RelativePath = $_.RelativePath; Content = $_.Content } })
    }
}

# Every mutation must target a file the scanner inspects and must actually change it.
function Edit-SnapshotFile {
    param([System.Collections.IDictionary] $Snapshot, [string] $RelativePath, [scriptblock] $Transform)
    $copy = Copy-IsolationSnapshot $Snapshot
    $target = @($copy.Files | Where-Object { $_.RelativePath -ceq $RelativePath })
    if ($target.Count -ne 1) { throw "Mutation target '$RelativePath' is not in the scanned snapshot." }
    $updated = [string](& $Transform $target[0].Content)
    if ($updated -ceq $target[0].Content) { throw "Mutation of '$RelativePath' did not change its content." }
    $target[0].Content = $updated
    return $copy
}

function Add-SnapshotText {
    param([System.Collections.IDictionary] $Snapshot, [string] $RelativePath, [string] $Text)
    $append = { param($content) $content + "`n$Text`n" }.GetNewClosure()
    return Edit-SnapshotFile $Snapshot $RelativePath $append
}

function Add-SnapshotFile {
    param([System.Collections.IDictionary] $Snapshot, [string] $RelativePath, [string] $Content)
    $copy = Copy-IsolationSnapshot $Snapshot
    if (@($copy.Files | Where-Object { $_.RelativePath -ieq $RelativePath }).Count -ne 0) {
        throw "Added file '$RelativePath' already exists in the snapshot."
    }
    $copy.Files = @($copy.Files) + [pscustomobject]@{ RelativePath = $RelativePath; Content = $Content }
    return $copy
}

function Edit-SnapshotManifest {
    param([System.Collections.IDictionary] $Snapshot, [scriptblock] $Transform)
    $copy = Copy-IsolationSnapshot $Snapshot
    $updated = [string](& $Transform $copy.DesktopManifest)
    if ($updated -ceq $copy.DesktopManifest) { throw 'Manifest mutation did not change its content.' }
    $copy.DesktopManifest = $updated
    return $copy
}

function Assert-SnapshotRejected {
    param([System.Collections.IDictionary] $Snapshot, [string] $Case, [string] $ExpectedMessage)
    Assert-Rejected { Assert-ShellPreviewIsolation -Source $Snapshot } $Case $ExpectedMessage
}

$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path
$source = Get-ShellPreviewIsolationSource -RepositoryRoot $repositoryRoot
Assert-ShellPreviewIsolation -Source $source | Out-Null
Write-Output 'PASS: current shell and synthetic preview source boundaries.'

$sourceRoot = Join-Path $repositoryRoot 'apps\encryptdrive-desktop\src'
$onDisk = @(Get-ChildItem -LiteralPath $sourceRoot -Filter '*.rs' -File -Recurse |
    ForEach-Object { [IO.Path]::GetRelativePath($sourceRoot, $_.FullName).Replace('\', '/') } | Sort-Object)
$scanned = @($source.Files | ForEach-Object RelativePath | Sort-Object)
if (($onDisk -join '|') -cne ($scanned -join '|') -or $scanned -notcontains 'design.rs' -or $scanned -notcontains 'ui/mod.rs') {
    throw "The scanner snapshot does not cover every Rust file under the Desktop source tree."
}
Write-Output "PASS: snapshot covers all $($scanned.Count) Rust files under apps/encryptdrive-desktop/src."

$side = 'forbidden runtime side-effect'
$inclusion = 'forbidden module-inclusion construct'

# --- Original 13 negative cases (Phase 1B-4) ---
Assert-SnapshotRejected (Edit-SnapshotManifest $source { param($m) $m.Replace('default = []', 'default = ["design-preview"]') }) 'preview enabled by default' 'default features empty'
Assert-SnapshotRejected (Edit-SnapshotFile $source 'main.rs' { param($c) [regex]::Replace($c, '(?s)#\[cfg\(feature = "design-preview"\)\]\s*mod ui;', 'mod ui;') }) 'UI module compiled without design-preview' 'feature-gated'
Assert-SnapshotRejected (Add-SnapshotText $source 'preview/model.rs' 'std::fs::write') 'preview file I/O' $side
Assert-SnapshotRejected (Add-SnapshotText $source 'preview/model.rs' "use std::{fs as fs};`nfs::write") 'grouped aliased preview file I/O' $side
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/app.rs' "use std::fs as fs;`nfs::write") 'aliased preview UI file I/O' $side
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/app.rs' 'encryptdrive_core::readiness') 'preview core capability call' $side
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/app.rs' "use encryptdrive_core as core;`ncore::readiness") 'aliased preview core call' $side
Assert-SnapshotRejected (Add-SnapshotText $source 'preview.rs' 'std::fs::write') 'preview catalog file I/O' $side
Assert-SnapshotRejected (Add-SnapshotText $source 'main.rs' "use encryptdrive_core as core;`ncore::readiness") 'aliased core call in application entry source' $side
Assert-SnapshotRejected (Add-SnapshotText $source 'shell.rs' 'std::fs::read') 'production shell file I/O' $side
Assert-SnapshotRejected (Edit-SnapshotFile $source 'ui/icons.rs' { param($c) $c.Replace('"segoeui.ttf"', '"../secret.txt"') }) 'unbounded system-font path' 'three approved system-font names'
Assert-SnapshotRejected (Add-SnapshotText $source 'shell.rs' 'let _fixture = PreviewModel::fixture(DatasetSize::OneHundred);') 'synthetic model in production shell' 'production shell must not expose synthetic fixtures'
Assert-SnapshotRejected (Edit-SnapshotFile $source 'preview.rs' { param($c) $c.Replace('ALL: [Self; 22]', 'ALL: [Self; 21]') }) 'incomplete preview scenario catalog' '22 scenarios'

# --- Phase 1B-4 remediation: coverage of every source file ---
Assert-SnapshotRejected (Add-SnapshotText $source 'design.rs' 'pub(crate) fn probe_write() -> std::io::Result<()> { std::fs::write("probe.txt", b"x") }') 'B1: std::fs write in the shared design root (design.rs)' "$side or production-crate reference: 'apps/encryptdrive-desktop/src/design.rs'"
$newModule = Add-SnapshotText $source 'main.rs' 'mod persistence;'
$newModule = Add-SnapshotFile $newModule 'persistence.rs' 'pub(crate) fn save(v: &[u8]) -> std::io::Result<()> { std::fs::write("settings.bin", v) }'
Assert-SnapshotRejected $newModule 'B2: new root module with std::fs::write' "$side or production-crate reference: 'apps/encryptdrive-desktop/src/persistence.rs'"
Assert-SnapshotRejected (Add-SnapshotText $source 'lib.rs' 'pub fn probe() { let _ = std::fs::remove_file("x"); }') 'std::fs operation in lib.rs' "$side or production-crate reference: 'apps/encryptdrive-desktop/src/lib.rs'"
$inTreePath = Add-SnapshotText $source 'design.rs' "#[path = `"helpers/io.rs`"]`npub(crate) mod io_helper;"
$inTreePath = Add-SnapshotFile $inTreePath 'helpers/io.rs' 'pub(crate) fn wipe() { let _ = std::fs::remove_dir_all("x"); }'
Assert-SnapshotRejected $inTreePath 'B3: in-tree #[path] module outside ui/preview with file I/O' "$side or production-crate reference: 'apps/encryptdrive-desktop/src/helpers/io.rs'"

# --- #[path], include! and extern crate ---
Assert-SnapshotRejected (Add-SnapshotText $source 'design.rs' "#[path = `"../../outside.rs`"]`npub(crate) mod outside;") '#[path] escaping the source root' 'escapes apps/encryptdrive-desktop/src'
Assert-SnapshotRejected (Add-SnapshotText $source 'design.rs' "#[path = `"C:/outside.rs`"]`npub(crate) mod outside;") 'absolute #[path]' 'absolute #[path]'
Assert-SnapshotRejected (Add-SnapshotText $source 'design.rs' "#[path = `"ui/notes.txt`"]`npub(crate) mod notes;") '#[path] to a non-scanned file' 'not a scanned Rust source file'
Assert-SnapshotRejected (Add-SnapshotText $source 'shell.rs' 'include!("generated.rs");') 'include! code inclusion' $inclusion
Assert-SnapshotRejected (Add-SnapshotText $source 'main.rs' 'extern crate std as s;') 'extern crate alias' $inclusion

# --- Statement-level use-tree expansion ---
Assert-SnapshotRejected (Add-SnapshotText $source 'shell.rs' 'use std::{io::{self, Write}, net::TcpStream};') 'B4: nested grouped std::net import in the shell' "import of 'std::net::TcpStream'"
Assert-SnapshotRejected (Add-SnapshotText $source 'preview/model.rs' "use std::{io::{self, Read}, path::Path};`npub fn probe() -> bool { Path::new(`"C:\\`").exists() }") 'B5: nested grouped std::path import in the preview model' "import of 'std::path::Path'"
Assert-SnapshotRejected (Add-SnapshotText $source 'preview/state.rs' 'use std::{io, net::TcpStream as Tcp};') 'grouped alias import' "import of 'std::net::TcpStream'"
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/components.rs' "pub(crate) use std::{`n    collections::BTreeMap,`n    // ; a comment must not end the statement`n    process::{`n        Command as Run,`n    },`n};") 'multiline nested import with comment' "import of 'std::process::Command'"
Assert-SnapshotRejected (Add-SnapshotText $source 'preview/model.rs' 'use ::std::fs::write;') 'leading :: import' "import of 'std::fs::write'"
Assert-SnapshotRejected (Add-SnapshotText $source 'shell.rs' 'use std as s;') 'aliased std root' "import of 'std'"
Assert-SnapshotRejected (Add-SnapshotText $source 'design.rs' 'use std::{self as s};') 'grouped self-aliased std root' "import of 'std'"
Assert-SnapshotRejected (Add-SnapshotText $source 'preview.rs' 'use std::*;') 'std root glob import' "import of 'std::*'"
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/theme.rs' 'use std::os::windows::fs::symlink_file;') 'std::os extension import in UI' "import of 'std::os::windows::fs::symlink_file'"

# --- Fully qualified backstops ---
Assert-SnapshotRejected (Add-SnapshotText $source 'preview/model.rs' 'pub fn probe() { let _ = TcpStream::connect("127.0.0.1:9"); }') 'unqualified TcpStream::connect backstop' "'TcpStream::connect('"
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/explorer.rs' 'fn probe() { let _ = TcpListener::bind("127.0.0.1:0"); }') 'unqualified TcpListener::bind in UI' "'TcpListener::bind('"
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/theme.rs' 'fn probe() { ::std::process::exit(0) }') 'leading :: std::process call in UI' "'std::process"
Assert-SnapshotRejected (Add-SnapshotText $source 'preview.rs' 'pub fn probe() { let _ = std::/* split */fs::read("x"); }') 'comment-split std::fs path' "'std::"
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/screens.rs' 'fn probe() -> bool { std::path::Path::new("C:\\").exists() }') 'std::path outside the UI font loader' "'std::path'"
Assert-SnapshotRejected (Add-SnapshotText $source 'ui/icons.rs' 'fn probe() { let _ = std::fs::read_dir("x"); }') 'second filesystem call in the UI font loader' 'only allowed UI file read'

# --- Snapshot and enumeration fail-closed behavior ---
$missingDesign = Copy-IsolationSnapshot $source
$missingDesign.Files = @($missingDesign.Files | Where-Object { $_.RelativePath -cne 'design.rs' })
Assert-SnapshotRejected $missingDesign 'snapshot without design.rs' 'Required shell-isolation input is missing'

$workRoot = Join-Path $TemporaryRoot ('shell-isolation-tests-' + [guid]::NewGuid().ToString('N'))
$junctionPath = $null
try {
    $missingRoot = Join-Path $workRoot 'missing-src'
    $null = New-Item -ItemType Directory -Path (Join-Path $missingRoot 'apps\encryptdrive-desktop') -Force
    Set-Content -LiteralPath (Join-Path $missingRoot 'apps\encryptdrive-desktop\Cargo.toml') -Value '[package]'
    Assert-Rejected { Get-ShellPreviewIsolationSource -RepositoryRoot $missingRoot } 'missing source root' 'Required shell-isolation input is missing'

    $emptyRoot = Join-Path $workRoot 'empty-src'
    $null = New-Item -ItemType Directory -Path (Join-Path $emptyRoot 'apps\encryptdrive-desktop\src\ui') -Force
    Set-Content -LiteralPath (Join-Path $emptyRoot 'apps\encryptdrive-desktop\Cargo.toml') -Value '[package]'
    Assert-Rejected { Get-ShellPreviewIsolationSource -RepositoryRoot $emptyRoot } 'empty source tree' 'contains no Rust source files'

    $reparseRoot = Join-Path $workRoot 'reparse-src'
    $reparseSource = Join-Path $reparseRoot 'apps\encryptdrive-desktop\src'
    $null = New-Item -ItemType Directory -Path $reparseSource -Force
    Set-Content -LiteralPath (Join-Path $reparseRoot 'apps\encryptdrive-desktop\Cargo.toml') -Value '[package]'
    foreach ($name in @('lib.rs', 'main.rs', 'shell.rs', 'design.rs', 'preview.rs')) {
        Set-Content -LiteralPath (Join-Path $reparseSource $name) -Value '// placeholder'
    }
    $outside = Join-Path $workRoot 'outside'
    $null = New-Item -ItemType Directory -Path $outside -Force
    Set-Content -LiteralPath (Join-Path $outside 'io.rs') -Value 'pub fn f() { let _ = std::fs::write("x", b"x"); }'
    try {
        $junctionPath = (New-Item -ItemType Junction -Path (Join-Path $reparseSource 'linked') -Target $outside -ErrorAction Stop).FullName
    }
    catch {
        Write-Output "BLOCKED: could not create a Windows junction to exercise reparse rejection ($($_.FullyQualifiedErrorId))."
    }
    if ($junctionPath) {
        Assert-Rejected { Get-ShellPreviewIsolationSource -RepositoryRoot $reparseRoot } 'junction inside the source tree' 'reparse point'
    }
}
finally {
    if ($junctionPath -and (Test-Path -LiteralPath $junctionPath)) {
        # Remove the link itself first so the recursive cleanup never follows it.
        [IO.Directory]::Delete($junctionPath)
    }
    if (Test-Path -LiteralPath $workRoot) {
        Remove-Item -LiteralPath $workRoot -Recurse -Force
    }
}

Write-Output "Shell isolation regression suite passed ($script:RejectedCases negative cases rejected)."
