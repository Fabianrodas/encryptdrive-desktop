#requires -Version 7.0

$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path
$targetRoot = Join-Path $repositoryRoot 'target\phase-1b'
$testRoot = Join-Path $targetRoot ('pwsh-wrapper-tests-' + [guid]::NewGuid().ToString('N'))
if (-not $testRoot.StartsWith($targetRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Wrapper-test files must remain below ignored target/phase-1b.'
}

function Invoke-PowerShellChild {
    param([Parameter(Mandatory = $true)] [string[]] $Arguments)
    $output = (& pwsh @Arguments 2>&1 | Out-String)
    return [pscustomobject]@{ exit_code = $LASTEXITCODE; output = $output }
}

function Get-GitHubPwshCommand {
    param([Parameter(Mandatory = $true)] [string] $ScriptPath)
    $literalPath = $ScriptPath.Replace("'", "''")
    return "`$ErrorActionPreference = 'stop'; . '$literalPath'; if ((Test-Path -LiteralPath variable:\LASTEXITCODE)) { exit `$LASTEXITCODE }"
}

try {
    $null = New-Item -ItemType Directory -Path $testRoot -Force
    $suite = Join-Path $PSScriptRoot 'verify-build-artifacts.tests.ps1'
    $commonArguments = @('-NoLogo', '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Unrestricted')

    $standalone = Invoke-PowerShellChild ($commonArguments + @('-File', $suite))
    if ($standalone.exit_code -ne 0) {
        throw "Standalone manifest rejection suite failed with exit $($standalone.exit_code):`n$($standalone.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: standalone manifest rejection suite exits 0.'

    $wrapped = Invoke-PowerShellChild ($commonArguments + @('-Command', (Get-GitHubPwshCommand $suite)))
    if ($wrapped.exit_code -ne 0) {
        throw "GitHub Actions pwsh wrapper emulation failed with exit $($wrapped.exit_code):`n$($wrapped.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: GitHub Actions pwsh wrapper emulation exits 0.'

    $probe = Join-Path $testRoot 'intentional-assertion-failure.ps1'
    $probeText = ". '$($suite.Replace("'", "''"))'`r`nAssert-True `$false 'intentional assertion failure probe'`r`n"
    Set-Content -LiteralPath $probe -Value $probeText -Encoding utf8

    $standaloneFailure = Invoke-PowerShellChild ($commonArguments + @('-File', $probe))
    if ($standaloneFailure.exit_code -eq 0 -or $standaloneFailure.output -notmatch 'ASSERTION FAILED: intentional assertion failure probe') {
        throw "Standalone failure probe did not fail at its intended assertion: exit=$($standaloneFailure.exit_code):`n$($standaloneFailure.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: standalone invocation propagates an intentional assertion failure.'

    $wrappedFailure = Invoke-PowerShellChild ($commonArguments + @('-Command', (Get-GitHubPwshCommand $probe)))
    if ($wrappedFailure.exit_code -eq 0 -or $wrappedFailure.output -notmatch 'ASSERTION FAILED: intentional assertion failure probe') {
        throw "Wrapped failure probe did not fail at its intended assertion: exit=$($wrappedFailure.exit_code):`n$($wrappedFailure.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: GitHub Actions pwsh wrapper propagates an intentional assertion failure.'

    $dependencySuite = Join-Path $PSScriptRoot 'verify-dependency-inventory.tests.ps1'
    $dependencyStandalone = Invoke-PowerShellChild ($commonArguments + @('-File', $dependencySuite))
    if ($dependencyStandalone.exit_code -ne 0) {
        throw "Standalone dependency inventory suite failed with exit $($dependencyStandalone.exit_code):`n$($dependencyStandalone.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: standalone dependency inventory suite exits 0.'

    $dependencyWrapped = Invoke-PowerShellChild ($commonArguments + @('-Command', (Get-GitHubPwshCommand $dependencySuite)))
    if ($dependencyWrapped.exit_code -ne 0) {
        throw "GitHub Actions dependency inventory wrapper emulation failed with exit $($dependencyWrapped.exit_code):`n$($dependencyWrapped.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: GitHub Actions dependency inventory wrapper emulation exits 0.'

    $dependencyRoot = Join-Path $testRoot 'dependency-inventory'
    $dependencyTests = Join-Path $dependencyRoot 'tests'
    $null = New-Item -ItemType Directory -Path $dependencyTests -Force
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot '..\DependencyInventory.psm1') -Destination (Join-Path $dependencyRoot 'DependencyInventory.psm1')
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot '..\verify-dependency-inventory.ps1') -Destination (Join-Path $dependencyRoot 'verify-dependency-inventory.ps1')

    $dependencySource = Get-Content -LiteralPath $dependencySuite -Raw
    $cargoGenerationStart = $dependencySource.IndexOf('$repositoryRoot = (Resolve-Path')
    if ($cargoGenerationStart -lt 0) { throw 'Could not isolate the dependency inventory assertions before the Cargo generation block.' }
    $isolatedSource = $dependencySource.Substring(0, $cargoGenerationStart) + "Write-Output 'PASS: isolated dependency inventory assertions.'`r`n"
    $isolatedSuite = Join-Path $dependencyTests 'verify-dependency-inventory-isolated.tests.ps1'
    [IO.File]::WriteAllText($isolatedSuite, $isolatedSource, [Text.UTF8Encoding]::new($false))

    $isolatedWrapped = Invoke-PowerShellChild ($commonArguments + @('-Command', (Get-GitHubPwshCommand $isolatedSuite)))
    if ($isolatedWrapped.exit_code -ne 0) {
        throw "Isolated dependency inventory assertions failed under GitHub Actions wrapper emulation with exit $($isolatedWrapped.exit_code):`n$($isolatedWrapped.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: isolated dependency inventory assertions without Cargo generation exit 0 under wrapper emulation.'

    $badAssertion = Join-Path $dependencyTests 'intentional-dependency-assertion-failure.tests.ps1'
    [IO.File]::WriteAllText($badAssertion, $isolatedSource + "Assert-True `$false 'intentional dependency assertion failure probe'`r`n", [Text.UTF8Encoding]::new($false))
    $badStandalone = Invoke-PowerShellChild ($commonArguments + @('-File', $badAssertion))
    if ($badStandalone.exit_code -eq 0 -or $badStandalone.output -notmatch 'ASSERTION FAILED: intentional dependency assertion failure probe') {
        throw "Standalone dependency assertion probe did not fail at its intended assertion: exit=$($badStandalone.exit_code):`n$($badStandalone.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: standalone dependency inventory suite propagates an intentional assertion failure.'

    $badWrapped = Invoke-PowerShellChild ($commonArguments + @('-Command', (Get-GitHubPwshCommand $badAssertion)))
    if ($badWrapped.exit_code -eq 0 -or $badWrapped.output -notmatch 'ASSERTION FAILED: intentional dependency assertion failure probe') {
        throw "Wrapped dependency assertion probe did not fail at its intended assertion: exit=$($badWrapped.exit_code):`n$($badWrapped.output)"
    }
    $global:LASTEXITCODE = 0
    Write-Output 'PASS: GitHub Actions dependency inventory wrapper propagates an intentional assertion failure.'
}
finally {
    if (Test-Path -LiteralPath $testRoot) {
        $resolvedTestRoot = (Resolve-Path -LiteralPath $testRoot).Path
        if (-not $resolvedTestRoot.StartsWith($targetRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Refusing to remove wrapper-test files outside ignored target/phase-1b.'
        }
        Remove-Item -LiteralPath $resolvedTestRoot -Recurse -Force
    }
}
