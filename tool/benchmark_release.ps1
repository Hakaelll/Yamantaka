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
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    current = $current
    tolerance = $Tolerance
    baseline = $null
    status = 'baseline-missing'
}
$comparison.baseline = Get-Content $baseline -Raw | ConvertFrom-Json
$limit = [double]$comparison.baseline.queryP95Us * (1 + $Tolerance)
$comparison.status = if ([double]$current.queryP95Us -le $limit) { 'pass' } else { 'fail' }
$comparison.baselineLimitP95Us = $limit
$comparisonPath = [System.IO.Path]::ChangeExtension($output, '.comparison.json')
$comparison | ConvertTo-Json -Depth 12 | Set-Content -Encoding UTF8 $comparisonPath
Write-Host "Performance comparison: $comparisonPath"
if ($comparison.status -eq 'fail') { exit 1 }

$pdfProfile = [System.IO.Path]::GetFullPath((Join-Path $root $PdfProfilePath))
if (Test-Path -LiteralPath $pdfProfile) {
    & (Join-Path $PSScriptRoot 'compare_pdf_profile.ps1') -ProfilePath $PdfProfilePath
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
