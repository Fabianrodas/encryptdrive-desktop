#requires -Version 7.0

Set-StrictMode -Version Latest

# Text-based regression guard for the production-shell / synthetic-preview boundary.
# It is not a Rust parser and not a security boundary; see
# docs/roadmap/PHASE_1B_SHELL_ISOLATION_VALIDATION.md for its limitations.

$script:SourceRootDisplay = 'apps/encryptdrive-desktop/src'
$script:RequiredSourceFiles = @('lib.rs', 'main.rs', 'shell.rs', 'design.rs', 'preview.rs')
$script:ProductionCrates = 'encryptdrive_core|vault_format|vault_crypto|vault_storage'

# Comments, string literals (normal, byte, C, raw) and char literals, in that order of precedence.
# Block comments nest in Rust, hence the balancing group.
$script:NonCodePattern = [regex]::new(
    '//[^\n]*' +
    '|/\*(?>/\*(?<depth>)|\*/(?<-depth>)|(?!/\*|\*/)[\s\S])*(?(depth)(?!))\*/' +
    '|(?<![\w])(?:br|cr|r)(?<hashes>#*)"[\s\S]*?"\k<hashes>' +
    '|"(?:\\[\s\S]|[^"\\])*"' +
    "|'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f]{1,6}\}|[\s\S])|[\uD800-\uDBFF][\uDC00-\uDFFF]|[^\\'\r\n])'")

# Import paths (after use-tree expansion, leading '::' removed) that are forbidden under every policy:
# the std root itself (`use std as s;`, `use std::{self as s};`, `use std::*;`), the file, network,
# process, path and OS-extension modules, and the production crates.
$script:ForbiddenImportPath = [regex]::new(
    "^(?:std(?:::\*)?|std::(?:fs|net|process|path|os)(?:::.*)?|(?:$($script:ProductionCrates))(?:::.*)?)$")

# Backstops for fully qualified use, applied only to comment/literal-free Rust code.
$script:StrictStdPattern = [regex]::new('\bstd\s*::\s*(?:r#)?(?:fs|net|process|path|os)\b')
$script:UiAlwaysForbiddenStdPattern = [regex]::new('\bstd\s*::\s*(?:r#)?(?:net|process|os)\b')
$script:UiFileStdPattern = [regex]::new('\bstd\s*::\s*(?:r#)?(?:fs|path)\b')
$script:RuntimeCallPattern = [regex]::new(
    '\b(?:TcpStream\s*::\s*connect(?:_timeout)?|TcpListener\s*::\s*bind|UdpSocket\s*::\s*bind|' +
    'Command\s*::\s*new|File\s*::\s*(?:open|create|create_new|options)|OpenOptions\s*::\s*new)\s*\(')
$script:ProductionCratePattern = [regex]::new("\b(?:$($script:ProductionCrates))\b")
$script:CodeInclusionPattern = [regex]::new('\binclude\s*!')
$script:ExternCratePattern = [regex]::new('\bextern\s+crate\b')

function Assert-SourceFileSet {
    param([object[]] $Files)

    if ($Files.Count -eq 0) {
        throw "Shell-isolation source tree contains no Rust source files: $script:SourceRootDisplay"
    }
    $paths = @{}
    foreach ($file in $Files) {
        $relativePath = [string]$file.RelativePath
        if ($relativePath -match '\\|^/|^[A-Za-z]:|(?:^|/)\.\.?(?:/|$)' -or -not $relativePath.EndsWith('.rs', [StringComparison]::OrdinalIgnoreCase)) {
            throw "Shell-isolation snapshot contains an invalid source path: '$relativePath'."
        }
        if ($paths.ContainsKey($relativePath)) {
            throw "Shell-isolation snapshot lists '$relativePath' more than once."
        }
        $paths[$relativePath] = $true
    }
    foreach ($required in $script:RequiredSourceFiles) {
        if (-not $paths.ContainsKey($required)) {
            throw "Required shell-isolation input is missing: $script:SourceRootDisplay/$required"
        }
    }
}

function Get-ShellPreviewIsolationSource {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)] [string] $RepositoryRoot)

    $root = (Resolve-Path -LiteralPath $RepositoryRoot -ErrorAction Stop).Path
    $manifestPath = Join-Path $root 'apps\encryptdrive-desktop\Cargo.toml'
    $sourceRoot = Join-Path $root 'apps\encryptdrive-desktop\src'
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Required shell-isolation input is missing: $manifestPath"
    }
    if (-not (Test-Path -LiteralPath $sourceRoot -PathType Container)) {
        throw "Required shell-isolation input is missing: $sourceRoot"
    }
    if ((Get-Item -LiteralPath $sourceRoot -Force -ErrorAction Stop).Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw "Shell-isolation source tree must not be a reparse point: $sourceRoot"
    }

    # Every item is inspected before any file is read, so a symlink or junction anywhere in the
    # tree fails closed instead of being followed or silently skipped.
    $items = @(Get-ChildItem -LiteralPath $sourceRoot -Recurse -Force -ErrorAction Stop | Sort-Object -Property FullName)
    foreach ($item in $items) {
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw "Shell-isolation source tree contains a reparse point (symlink or junction): $($item.FullName)"
        }
    }
    $files = @(foreach ($item in $items) {
        if ($item.PSIsContainer -or -not $item.Name.EndsWith('.rs', [StringComparison]::OrdinalIgnoreCase)) { continue }
        [pscustomobject]@{
            RelativePath = [IO.Path]::GetRelativePath($sourceRoot, $item.FullName).Replace('\', '/')
            Content = [IO.File]::ReadAllText($item.FullName)
        }
    })
    Assert-SourceFileSet $files

    return [ordered]@{
        DesktopManifest = [IO.File]::ReadAllText($manifestPath)
        Files = $files
    }
}

function ConvertTo-CodeOnlyText {
    param([string] $Text, [string] $DisplayPath)

    # Same length as the input, with comments and literals blanked, so offsets stay comparable.
    $code = $script:NonCodePattern.Replace($Text, { param($match) [regex]::Replace($match.Value, '[^\r\n]', ' ') })
    if ($code.Contains('"') -or $code.Contains('/*')) {
        throw "Shell-isolation scan could not tokenize literals or comments in '$DisplayPath'; failing closed."
    }
    return $code
}

function Expand-UseTree {
    param([string[]] $Tokens, [ref] $Index, [string] $Prefix)

    $path = $Prefix
    if ($Index.Value -lt $Tokens.Count -and $Tokens[$Index.Value] -eq '::') { $Index.Value++ }
    while ($Index.Value -lt $Tokens.Count) {
        $token = $Tokens[$Index.Value]
        $Index.Value++
        if ($token -eq '*') {
            if ($path) { "$path::*" } else { '*' }
            return
        }
        if ($token -eq '{') {
            while ($Index.Value -lt $Tokens.Count -and $Tokens[$Index.Value] -ne '}') {
                Expand-UseTree -Tokens $Tokens -Index $Index -Prefix $path
                if ($Index.Value -lt $Tokens.Count -and $Tokens[$Index.Value] -eq ',') { $Index.Value++ }
                elseif ($Index.Value -ge $Tokens.Count -or $Tokens[$Index.Value] -ne '}') { throw 'unbalanced use group' }
            }
            if ($Index.Value -ge $Tokens.Count) { throw 'unterminated use group' }
            $Index.Value++
            return
        }
        if ($token -cmatch '^(?:r#)?[A-Za-z_]\w*$' -and $token -cne 'as') {
            $segment = $token -creplace '^r#', ''
            $path = if ($segment -ceq 'self' -and $path) { $path } elseif ($path) { "$path::$segment" } else { $segment }
            if ($Index.Value -lt $Tokens.Count -and $Tokens[$Index.Value] -eq '::') {
                $Index.Value++
                continue
            }
            if ($Index.Value -lt $Tokens.Count -and $Tokens[$Index.Value] -ceq 'as') {
                $Index.Value++
                if ($Index.Value -ge $Tokens.Count -or $Tokens[$Index.Value] -cnotmatch '^(?:r#)?[A-Za-z_]\w*$') { throw 'invalid use alias' }
                $Index.Value++
            }
            $path
            return
        }
        throw "unexpected token '$token'"
    }
    throw 'incomplete use path'
}

function Get-UseStatementPath {
    param([string] $Code, [string] $DisplayPath)

    # `use<'a>` is precise-capture syntax in impl-trait bounds, not an import.
    foreach ($match in [regex]::Matches($Code, '(?<![\w#$])use\b(?!\s*<)')) {
        $end = $Code.IndexOf(';', $match.Index)
        if ($end -lt 0) { throw "Unterminated use statement in '$DisplayPath'; failing closed." }
        $statement = $Code.Substring($match.Index + 3, $end - $match.Index - 3)
        $tokens = @([regex]::Matches($statement, '::|[{},*]|(?:r#)?[A-Za-z_]\w*|\S') | ForEach-Object Value)
        $index = 0
        try {
            $paths = @(Expand-UseTree -Tokens $tokens -Index ([ref]$index) -Prefix '')
            if ($index -ne $tokens.Count) { throw 'trailing tokens' }
        }
        catch {
            throw "Unrecognized use statement in '$DisplayPath' ($($_.Exception.Message)): 'use$($statement.Trim())'; failing closed."
        }
        $paths
    }
}

function Assert-ModuleInclusion {
    param([pscustomobject] $File, [string] $Code, [hashtable] $KnownPaths, [string] $DisplayPath)

    $raw = [string]$File.Content
    foreach ($pattern in @($script:CodeInclusionPattern, $script:ExternCratePattern)) {
        $found = $pattern.Match($Code)
        if (-not $found.Success) { $found = $pattern.Match($raw) }
        if ($found.Success) {
            throw "Source contains a forbidden module-inclusion construct: '$DisplayPath' — '$($found.Value)'."
        }
    }

    foreach ($start in [regex]::Matches($Code, '#\s*!?\s*\[')) {
        $depth = 0
        $end = -1
        for ($i = $start.Index + $start.Length - 1; $i -lt $Code.Length; $i++) {
            if ($Code[$i] -eq '[') { $depth++ }
            elseif ($Code[$i] -eq ']') {
                $depth--
                if ($depth -eq 0) { $end = $i; break }
            }
        }
        if ($end -lt 0) { throw "Unterminated attribute in '$DisplayPath'; failing closed." }
        $codeSpan = $Code.Substring($start.Index, $end - $start.Index + 1)
        $pathKeys = [regex]::Matches($codeSpan, '\bpath\s*=(?!=)').Count
        if ($pathKeys -eq 0) { continue }
        $literals = [regex]::Matches($raw.Substring($start.Index, $end - $start.Index + 1), '\bpath\s*=\s*"([^"\\]*)"')
        if ($literals.Count -ne $pathKeys) {
            throw "Source contains a forbidden module-inclusion construct: '$DisplayPath' — non-literal #[path] attribute."
        }
        foreach ($literal in $literals) {
            $target = $literal.Groups[1].Value
            if ($target -match '^(?:[A-Za-z]:|/)') {
                throw "Source contains a forbidden module-inclusion construct: '$DisplayPath' — absolute #[path] '$target'."
            }
            $segments = [System.Collections.Generic.List[string]]::new()
            $parts = @(([string]$File.RelativePath).Split('/') | Select-Object -SkipLast 1) + $target.Split('/')
            foreach ($part in $parts) {
                if ($part -eq '' -or $part -eq '.') { continue }
                if ($part -eq '..') {
                    if ($segments.Count -eq 0) {
                        throw "Source contains a forbidden module-inclusion construct: '$DisplayPath' — #[path] '$target' escapes $script:SourceRootDisplay."
                    }
                    $segments.RemoveAt($segments.Count - 1)
                    continue
                }
                $segments.Add($part)
            }
            $resolved = $segments -join '/'
            if (-not $resolved.EndsWith('.rs', [StringComparison]::OrdinalIgnoreCase) -or -not $KnownPaths.ContainsKey($resolved)) {
                throw "Source contains a forbidden module-inclusion construct: '$DisplayPath' — #[path] '$target' is not a scanned Rust source file."
            }
        }
    }
}

function Assert-FontLoader {
    param([string] $Content, [string] $Code, [string] $DisplayPath)

    $fontReads = [regex]::Matches($Content, '(?i)\bstd::fs\s*::\s*([A-Za-z_]\w*)')
    if ($fontReads.Count -ne 1 -or $fontReads[0].Groups[1].Value -cne 'read' -or
        [regex]::Matches($Code, '\bstd\s*::\s*fs\b').Count -ne 1 -or
        $Content -notmatch '(?s)std::env::var_os\("WINDIR"\).*?PathBuf::from\(windows_directory\)\.join\("Fonts"\).*?std::fs::read\(fonts_directory\.join\(filename\)\)') {
        throw "The only allowed UI file read is the bounded Windows system-font load in '$DisplayPath'."
    }
    $fontCalls = [regex]::Matches($Content, '(?s)load_system_font\s*\(\s*definitions\s*,\s*&fonts_directory\s*,\s*"([^"]+)"')
    $fontNames = @($fontCalls | ForEach-Object { $_.Groups[1].Value } | Sort-Object)
    $expectedFontNames = @('segoeui.ttf', 'seguisb.ttf', 'segoeuib.ttf') | Sort-Object
    $fontFunctionCount = [regex]::Matches($Content, '\bload_system_font\s*\(').Count
    if ($fontCalls.Count -ne 3 -or ($fontNames -join '|') -cne ($expectedFontNames -join '|') -or $fontFunctionCount -ne 4) {
        throw "The Windows font loader must use only the three approved system-font names: '$DisplayPath'."
    }
}

function Assert-SourceFilePolicy {
    param([pscustomobject] $File, [hashtable] $KnownPaths)

    $relativePath = [string]$File.RelativePath
    $displayPath = "$script:SourceRootDisplay/$relativePath"
    $raw = [string]$File.Content
    $code = ConvertTo-CodeOnlyText -Text $raw -DisplayPath $displayPath
    $isUi = $relativePath.StartsWith('ui/', [StringComparison]::OrdinalIgnoreCase)
    $isFontLoader = $relativePath -ieq 'ui/icons.rs'
    $policy = if ($isUi) {
        'UI-policy source (ui/: shared production design files compiled through design.rs plus preview-only modules)'
    }
    else {
        'Strict-policy source (application entry, shell, shared design root, preview model, or any other non-ui file)'
    }
    $reject = { param($detail) throw "$policy contains a forbidden runtime side-effect or production-crate reference: '$displayPath' — $detail." }

    Assert-ModuleInclusion -File $File -Code $code -KnownPaths $KnownPaths -DisplayPath $displayPath

    foreach ($importPath in @(Get-UseStatementPath -Code $code -DisplayPath $displayPath)) {
        if ($script:ForbiddenImportPath.IsMatch($importPath)) { & $reject "import of '$importPath'" }
    }

    $patterns = @($script:RuntimeCallPattern, $script:ProductionCratePattern)
    $patterns += if ($isUi) { $script:UiAlwaysForbiddenStdPattern } else { $script:StrictStdPattern }
    if ($isUi -and -not $isFontLoader) { $patterns += $script:UiFileStdPattern }
    foreach ($pattern in $patterns) {
        $found = $pattern.Match($code)
        if ($found.Success) { & $reject "'$($found.Value)'" }
    }

    if ($isFontLoader) {
        Assert-FontLoader -Content $raw -Code $code -DisplayPath $displayPath
    }
}

function Assert-ShellPreviewIsolation {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)] [System.Collections.IDictionary] $Source)

    foreach ($name in @('DesktopManifest', 'Files')) {
        if (-not $Source.Contains($name) -or $null -eq $Source[$name]) {
            throw "Shell-isolation snapshot is missing '$name'."
        }
    }
    $files = @($Source.Files)
    Assert-SourceFileSet $files
    $knownPaths = @{}
    foreach ($file in $files) { $knownPaths[[string]$file.RelativePath] = [string]$file.Content }
    $librarySource = $knownPaths['lib.rs']
    $mainSource = $knownPaths['main.rs']
    $shellSource = $knownPaths['shell.rs']
    $previewCatalog = $knownPaths['preview.rs']

    $features = [regex]::Match([string]$Source.DesktopManifest, '(?ms)^\[features\]\s*(.*?)(?=^\[|\z)')
    if (-not $features.Success -or
        $features.Groups[1].Value -notmatch '(?m)^\s*default\s*=\s*\[\s*\]\s*$') {
        throw 'The Desktop Cargo manifest must keep default features empty.'
    }
    if ($features.Groups[1].Value -notmatch '(?m)^\s*design-preview\s*=\s*\[\s*\]\s*$') {
        throw 'The design-preview feature must remain explicit and dependency-free.'
    }

    if ($librarySource -notmatch '(?s)#\[cfg\(any\(feature\s*=\s*"design-preview"\s*,\s*test\)\)\]\s*pub\s+mod\s+preview\s*;') {
        throw 'The preview model must be gated to design-preview or unit-test compilation.'
    }
    if ($mainSource -notmatch '(?s)#\[cfg\(feature\s*=\s*"design-preview"\)\]\s*mod\s+ui\s*;') {
        throw 'The preview-only UI module tree (ui/mod.rs) must be feature-gated.'
    }
    if ($mainSource -notmatch '(?s)#\[cfg\(feature\s*=\s*"design-preview"\)\]\s*fn\s+run_preview\s*\(') {
        throw 'The preview entry point must be feature-gated.'
    }
    if ($mainSource -notmatch '(?s)#\[cfg\(not\(feature\s*=\s*"design-preview"\)\)\]\s*fn\s+main\s*\(\)\s*->\s*eframe::Result\s*\{\s*run_shell\(\)') {
        throw 'The default binary entry point must start the production shell.'
    }
    if ($shellSource -match '(?i)\bpreview\b|design-preview|encryptdrive_core|readiness\s*\(') {
        throw 'The production shell must not depend on preview or core readiness behavior.'
    }
    if ($shellSource -notmatch 'ShellState::new\(\)' -or
        $shellSource -notmatch 'No vault open' -or
        $shellSource -notmatch 'Trash is unavailable') {
        throw 'The production shell must start from its empty no-vault state and explain unavailable Trash.'
    }
    if ($shellSource -match '(?i)PreviewModel|PreviewScenario|OperationState|create_vault|authenticate|import_file|export_file|move_to_trash|restore_file|delete_file') {
        throw 'The production shell must not expose synthetic fixtures or real operation handlers.'
    }

    $scenarioCatalog = [regex]::Match($previewCatalog, '(?s)pub\s+const\s+ALL\s*:\s*\[\s*Self\s*;\s*(\d+)\s*\]\s*=\s*\[(.*?)\];')
    if (-not $scenarioCatalog.Success) {
        throw 'The preview scenario catalog must declare its fixed ALL array.'
    }
    $declaredScenarioCount = [int]$scenarioCatalog.Groups[1].Value
    $listedScenarioCount = [regex]::Matches($scenarioCatalog.Groups[2].Value, '\bSelf::[A-Za-z_]\w*').Count
    if ($declaredScenarioCount -ne 22 -or $listedScenarioCount -ne 22) {
        throw "The synthetic preview must keep all 22 scenarios in its catalog; declared=$declaredScenarioCount listed=$listedScenarioCount."
    }

    foreach ($file in $files) {
        Assert-SourceFilePolicy -File $file -KnownPaths $knownPaths
    }

    $allUiText = (@($files | Where-Object { ([string]$_.RelativePath).StartsWith('ui/', [StringComparison]::OrdinalIgnoreCase) } |
        ForEach-Object { [string]$_.Content }) -join "`n")
    if ($mainSource -notmatch 'EncryptDrive\s+—\s+Design preview' -or
        $allUiText -notmatch 'Sample data\s+—\s+no real file or vault is affected\.') {
        throw 'Preview UI must clearly identify its design-preview and synthetic-data status.'
    }

    return "Source and feature boundaries verified across $($files.Count) Rust files under $script:SourceRootDisplay; release feature identity is verified separately by the build manifest."
}

Export-ModuleMember -Function Get-ShellPreviewIsolationSource, Assert-ShellPreviewIsolation
