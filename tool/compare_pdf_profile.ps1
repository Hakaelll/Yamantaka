[CmdletBinding()]
param(
    [string]$ProfilePath = 'build/performance/performance-profile.json',
    [string]$BaselinePath = 'docs/verification/pdf-profile-baseline.json',
    [string]$OutputPath = 'build/performance/pdf-profile-comparison.json'
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$profileFile = [System.IO.Path]::GetFullPath((Join-Path $root $ProfilePath))
$baselineFile = [System.IO.Path]::GetFullPath((Join-Path $root $BaselinePath))
$outputFile = [System.IO.Path]::GetFullPath((Join-Path $root $OutputPath))
if (-not (Test-Path -LiteralPath $profileFile)) {
    throw "PDF performance profile not found: $profileFile"
}
if (-not (Test-Path -LiteralPath $baselineFile)) {
    throw "PDF performance baseline not found: $baselineFile"
}

function Get-MetricValue {
    param([object]$Root, [string]$Path)
    $value = $Root
    foreach ($segment in $Path.Split('.')) {
        $property = $value.PSObject.Properties[$segment]
        if ($null -eq $property) { throw "Metric is missing from profile: $Path" }
        $value = $property.Value
    }
    return [double]$value
}

$profile = Get-Content -Raw -Encoding UTF8 -LiteralPath $profileFile | ConvertFrom-Json
$baseline = Get-Content -Raw -Encoding UTF8 -LiteralPath $baselineFile | ConvertFrom-Json
if ($profile.environment.contains_paths_or_titles -ne $false) {
    throw 'The PDF profile is not marked as free of paths and titles.'
}
$results = @(
    foreach ($metric in $baseline.metrics) {
        $current = Get-MetricValue -Root $profile -Path $metric.path
        $limit = [double]$metric.value * (1 + [double]$metric.tolerance)
        [ordered]@{
            metric = $metric.path
            baseline = [double]$metric.value
            current = $current
            tolerance = [double]$metric.tolerance
            limit = $limit
            status = if ($current -le $limit) { 'pass' } else { 'fail' }
        }
    }
)
$comparison = [ordered]@{
    format = 'yamantaka-pdf-profile-comparison'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    baselineRelease = $baseline.release
    containsPersonalData = $false
    status = if (@($results | Where-Object { $_.status -eq 'fail' }).Count -eq 0) { 'pass' } else { 'fail' }
    metrics = $results
}
New-Item -ItemType Directory -Force -Path (Split-Path $outputFile -Parent) | Out-Null
$comparison | ConvertTo-Json -Depth 12 | Set-Content -Encoding UTF8 -LiteralPath $outputFile
Write-Host "PDF profile comparison: $outputFile ($($comparison.status))"
if ($comparison.status -eq 'fail') { exit 1 }
