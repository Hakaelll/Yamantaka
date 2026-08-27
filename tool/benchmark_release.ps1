[CmdletBinding()]
param(
    [string]$BaselinePath = 'docs/verification/performance-baseline.json',
    [string]$OutputPath = 'build/verification/performance/library-benchmark.json',
    [double]$Tolerance = 0.10,
    [string]$PdfProfilePath = 'build/performance/performance-profile.json',
    [string]$DartExecutable
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$baseline = [System.IO.Path]::GetFullPath((Join-Path $root $BaselinePath))
$output = [System.IO.Path]::GetFullPath((Join-Path $root $OutputPath))
New-Item -ItemType Directory -Force -Path (Split-Path $output -Parent) | Out-Null
$stateRoot = Join-Path $root 'build/verification/tool-state'
New-Item -ItemType Directory -Force -Path $stateRoot | Out-Null
$env:APPDATA = $stateRoot
$env:LOCALAPPDATA = $stateRoot
if (-not $DartExecutable) {
    $localDart = Join-Path $root '.flutter-sdk/bin/dart.bat'
    $DartExecutable = if (Test-Path $localDart) {
        $localDart
    }
    else {
        $command = Get-Command dart -ErrorAction SilentlyContinue
        if (-not $command) { throw 'dart was not found on PATH or .flutter-sdk.' }
        $command.Source
    }
}
if (-not (Test-Path $baseline)) {
    throw "Performance baseline not found: $baseline"
}
& $DartExecutable run tool/benchmark_library.dart $output
if ($LASTEXITCODE -ne 0) { throw "Library benchmark failed with exit code $LASTEXITCODE." }
$current = Get-Content $output -Raw | ConvertFrom-Json
$comparison = [ordered]@{
    format = 'yamantaka-performance-comparison'
    version = 2
    generatedAt = [DateTime]::UtcNow.ToString('o')
    current = $current
    tolerance = $Tolerance
    baseline = $null
    checks = @()
    status = 'baseline-missing'
}
$comparison.baseline = Get-Content $baseline -Raw | ConvertFrom-Json
$checks = @()
if ($current.version -ge 2 -and $current.results) {
    if ($comparison.baseline.version -ge 2 -and $comparison.baseline.results) {
        foreach ($baselineResult in $comparison.baseline.results) {
            $currentResult = @($current.results | Where-Object { $_.documents -eq $baselineResult.documents }) | Select-Object -First 1
            if (-not $currentResult) { continue }
            foreach ($metric in @('firstLoadUs', 'queryMedianUs', 'queryP95Us', 'rssBytes')) {
                if ($null -eq $baselineResult.$metric -or $null -eq $currentResult.$metric) { continue }
                $baselineValue = [double]$baselineResult.$metric
                $currentValue = [double]$currentResult.$metric
                $limit = $baselineValue * (1 + $Tolerance)
                $checks += [pscustomobject]@{
                    documents = [int]$baselineResult.documents
                    metric = $metric
                    baseline = $baselineValue
                    current = $currentValue
                    limit = $limit
                    deltaPercent = if ($baselineValue -eq 0) { $null } else { (($currentValue - $baselineValue) / $baselineValue) * 100 }
                    status = if ($currentValue -le $limit) { 'pass' } else { 'fail' }
                }
            }
        }
    }
    elseif ($null -ne $comparison.baseline.queryP95Us -and $null -ne $comparison.baseline.documents) {
        $currentResult = @($current.results | Where-Object { $_.documents -eq $comparison.baseline.documents }) | Select-Object -First 1
        if ($currentResult) {
            $baselineValue = [double]$comparison.baseline.queryP95Us
            $currentValue = [double]$currentResult.queryP95Us
            $limit = $baselineValue * (1 + $Tolerance)
            $checks += [pscustomobject]@{
                documents = [int]$comparison.baseline.documents
                metric = 'queryP95Us'
                baseline = $baselineValue
                current = $currentValue
                limit = $limit
                deltaPercent = if ($baselineValue -eq 0) { $null } else { (($currentValue - $baselineValue) / $baselineValue) * 100 }
                status = if ($currentValue -le $limit) { 'pass' } else { 'fail' }
            }
        }
    }
}
elseif ($null -ne $current.queryP95Us -and $null -ne $comparison.baseline.queryP95Us) {
    $baselineValue = [double]$comparison.baseline.queryP95Us
    $currentValue = [double]$current.queryP95Us
    $limit = $baselineValue * (1 + $Tolerance)
    $checks += [pscustomobject]@{
        documents = [int]$current.documents
        metric = 'queryP95Us'
        baseline = $baselineValue
        current = $currentValue
        limit = $limit
        deltaPercent = if ($baselineValue -eq 0) { $null } else { (($currentValue - $baselineValue) / $baselineValue) * 100 }
        status = if ($currentValue -le $limit) { 'pass' } else { 'fail' }
    }
}
if ($checks.Count -eq 0) {
    throw 'The current benchmark and baseline do not contain comparable result rows.'
}
$comparison.checks = @($checks)
$comparison.status = if (@($checks | Where-Object { $_.status -eq 'fail' }).Count -eq 0) { 'pass' } else { 'fail' }
$comparisonPath = [System.IO.Path]::ChangeExtension($output, '.comparison.json')
$comparison | ConvertTo-Json -Depth 12 | Set-Content -Encoding UTF8 $comparisonPath
Write-Host "Performance comparison: $comparisonPath"
if ($comparison.status -eq 'fail') { exit 1 }

$pdfProfile = [System.IO.Path]::GetFullPath((Join-Path $root $PdfProfilePath))
if (Test-Path -LiteralPath $pdfProfile) {
    & (Join-Path $PSScriptRoot 'compare_pdf_profile.ps1') -ProfilePath $PdfProfilePath
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
