#requires -Version 7.0

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$javaSource = Join-Path $PSScriptRoot 'unicode-compat\JavaUnicodeOracle.java'
$caseTable = Join-Path $repositoryRoot 'crates\vault-format\src\java_case_fold_table.rs'
$java = (Get-Command java.exe -ErrorAction Stop).Source
$javac = Join-Path (Split-Path -Parent $java) 'javac.exe'
if (-not (Test-Path -LiteralPath $javac -PathType Leaf)) {
    throw 'The Java compiler must be installed beside the selected Java 21 runtime.'
}

$temporaryRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
$work = Join-Path $temporaryRoot ('encryptdrive-java-unicode-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
$resolvedWork = [System.IO.Path]::GetFullPath($work)
if (-not $resolvedWork.StartsWith($temporaryRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Refusing to use an oracle directory outside the operating-system temporary directory.'
}

try {
    & $javac -d $work $javaSource
    if ($LASTEXITCODE -ne 0) { throw "javac failed with exit code $LASTEXITCODE" }

    $mapping = @(& $java -cp $work JavaUnicodeOracle --mapping)
    if ($LASTEXITCODE -ne 0) { throw "Java mapping oracle failed with exit code $LASTEXITCODE" }
    if ($mapping.Count -lt 2 -or $mapping[0] -notmatch '^META\|Oracle Corporation\|21\.0\.2\+13-LTS-58\|Unicode-15\.0$') {
        throw 'The differential evidence is pinned to Oracle JDK 21.0.2+13-LTS-58 and Unicode 15.0.'
    }

    $expectedMap = @($mapping | Where-Object { $_.StartsWith('MAP|', [StringComparison]::Ordinal) })
    $tableSummary = @($mapping | Where-Object { $_.StartsWith('TABLE|', [StringComparison]::Ordinal) })
    if ($tableSummary.Count -ne 1 -or $tableSummary[0] -cne 'TABLE|1456|42556AF8AEFC14AB') {
        throw "Unexpected JDK-derived case-table summary: $($tableSummary -join ', ')"
    }
    $actualMap = [System.Collections.Generic.List[string]]::new()
    foreach ($match in [regex]::Matches((Get-Content -LiteralPath $caseTable -Raw), '(?m)^\s*\(0x([0-9a-f]+), 0x([0-9a-f]+)\),\r?$')) {
        $actualMap.Add(('MAP|{0}|{1}' -f $match.Groups[1].Value.ToUpperInvariant(), $match.Groups[2].Value.ToUpperInvariant()))
    }
    if ($actualMap.Count -ne $expectedMap.Count) {
        throw "Generated Java case map has $($expectedMap.Count) entries; checked-in map has $($actualMap.Count)."
    }
    for ($index = 0; $index -lt $expectedMap.Count; $index++) {
        if ($actualMap[$index] -cne $expectedMap[$index]) {
            throw "Java case mapping differs from the checked-in table at mapping index $index."
        }
    }

    $vectorsPath = Join-Path $work 'vectors.txt'
    $vectors = @(& $java -cp $work JavaUnicodeOracle --vectors)
    if ($LASTEXITCODE -ne 0) { throw "Java differential oracle failed with exit code $LASTEXITCODE" }
    if ($vectors[0] -notmatch '^META\|Oracle Corporation\|21\.0\.2\+13-LTS-58\|Unicode-15\.0$') {
        throw 'The vector oracle did not run on the required Oracle JDK 21.0.2 runtime.'
    }
    $summary = @($vectors | Where-Object { $_.StartsWith('SUMMARY|', [StringComparison]::Ordinal) })
    if ($summary.Count -ne 1 -or $summary[0] -notmatch '^SUMMARY\|validScalars=1112064\|whitespace=25\|mapped=1456\|queries=14418\|seed=454E435259505444$') {
        throw "Unexpected Java oracle corpus summary: $($summary -join ', ')"
    }
    Set-Content -LiteralPath $vectorsPath -Value $vectors -Encoding ascii

    Push-Location $repositoryRoot
    try {
        Get-Content -LiteralPath $vectorsPath | & cargo run --quiet -p vault-format --example java_unicode_probe --locked
        if ($LASTEXITCODE -ne 0) { throw "Rust differential probe failed with exit code $LASTEXITCODE" }
    }
    finally {
        Pop-Location
    }

    Write-Output "Oracle: $($vectors[0])"
    Write-Output "Mapping entries: $($expectedMap.Count)"
    Write-Output "Corpus: $($summary[0])"
}
finally {
    if ((Test-Path -LiteralPath $resolvedWork -PathType Container) -and
        $resolvedWork.StartsWith($temporaryRoot, [StringComparison]::OrdinalIgnoreCase)) {
        Remove-Item -LiteralPath $resolvedWork -Recurse -Force
    }
}
