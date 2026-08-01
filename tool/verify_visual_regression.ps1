[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$ManifestPath,
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [switch]$ApproveBaselineUpdate
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$manifestFile = [System.IO.Path]::GetFullPath($ManifestPath)
$manifestRoot = Split-Path -Parent $manifestFile
$outputRoot = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
$manifest = Get-Content $manifestFile -Raw | ConvertFrom-Json
$results = [System.Collections.Generic.List[object]]::new()

function Resolve-ManifestPath {
    param([string]$Path)
    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }
    return [System.IO.Path]::GetFullPath((Join-Path $manifestRoot $Path))
}

function Compare-Png {
    param(
        [Parameter(Mandatory = $true)][string]$BaselinePath,
        [Parameter(Mandatory = $true)][string]$CandidatePath,
        [int]$ChannelTolerance = 0,
        [double]$MaxChangedRatio = 0.0
    )
    $baseline = [System.Drawing.Bitmap]::new($BaselinePath)
    $candidate = [System.Drawing.Bitmap]::new($CandidatePath)
    try {
        if ($baseline.Width -ne $candidate.Width -or $baseline.Height -ne $candidate.Height) {
            return [ordered]@{
                passed = $false
                reason = 'dimensions'
                width = $candidate.Width
                height = $candidate.Height
                baselineWidth = $baseline.Width
                baselineHeight = $baseline.Height
                changedPixels = -1
                changedRatio = 1.0
                maxChannelDelta = 255
            }
        }
        $changed = 0L
        $maxDelta = 0
        for ($y = 0; $y -lt $baseline.Height; $y++) {
            for ($x = 0; $x -lt $baseline.Width; $x++) {
                $left = $baseline.GetPixel($x, $y)
                $right = $candidate.GetPixel($x, $y)
                $delta = [Math]::Max(
                    [Math]::Abs($left.R - $right.R),
                    [Math]::Max([Math]::Abs($left.G - $right.G), [Math]::Abs($left.B - $right.B))
                )
                if ($delta -gt $maxDelta) { $maxDelta = $delta }
                if ($delta -gt $ChannelTolerance) { $changed++ }
            }
        }
        $pixels = [double]($baseline.Width * $baseline.Height)
        $ratio = if ($pixels -eq 0) { 0 } else { $changed / $pixels }
        return [ordered]@{
            passed = $ratio -le $MaxChangedRatio
            reason = 'comparison'
            width = $baseline.Width
            height = $baseline.Height
            changedPixels = $changed
            changedRatio = $ratio
            maxChannelDelta = $maxDelta
        }
    }
    finally {
        $baseline.Dispose()
        $candidate.Dispose()
    }
}

foreach ($scenario in @($manifest.scenarios)) {
    $name = [string]$scenario.name
    $baselinePath = Resolve-ManifestPath ([string]$scenario.baseline)
    $candidatePath = Resolve-ManifestPath ([string]$scenario.candidate)
    $tolerance = if ($null -eq $scenario.channelTolerance) { 0 } else { [int]$scenario.channelTolerance }
    $allowedRatio = if ($null -eq $scenario.maxChangedRatio) { 0.0 } else { [double]$scenario.maxChangedRatio }
    $result = [ordered]@{
        name = $name
        baseline = $baselinePath
        candidate = $candidatePath
        scenario = $scenario
        status = 'fail'
    }
    if (-not (Test-Path $baselinePath)) {
        $result.reason = 'baseline-missing'
    }
    elseif (-not (Test-Path $candidatePath)) {
        $result.reason = 'candidate-missing'
    }
    else {
        $comparison = Compare-Png `
            -BaselinePath $baselinePath `
            -CandidatePath $candidatePath `
            -ChannelTolerance $tolerance `
            -MaxChangedRatio $allowedRatio
        $result.comparison = $comparison
        $result.status = if ($comparison.passed) { 'pass' } else { 'fail' }
        if ($result.status -eq 'fail') {
            $result.reason = $comparison.reason
        }
        if ($result.status -eq 'pass' -and $ApproveBaselineUpdate) {
            if (-not $scenario.allowBaselineUpdate) {
                throw "Scenario '$name' does not allow baseline updates."
            }
            Copy-Item -LiteralPath $candidatePath -Destination $baselinePath -Force
            $result.status = 'baseline-updated'
        }
    }
    $results.Add([pscustomobject]$result)
}

$report = [ordered]@{
    format = 'yamantaka-visual-regression'
    version = 1
    manifest = $manifestFile
    generatedAt = [DateTime]::UtcNow.ToString('o')
    approveBaselineUpdate = [bool]$ApproveBaselineUpdate
    passed = @($results | Where-Object { $_.status -in @('pass', 'baseline-updated') }).Count
    failed = @($results | Where-Object { $_.status -eq 'fail' }).Count
    results = @($results)
}
$reportPath = Join-Path $outputRoot 'visual-regression-report.json'
$report | ConvertTo-Json -Depth 12 | Set-Content -Encoding UTF8 $reportPath
$results | Format-Table name, status, reason -AutoSize
Write-Host "Evidence: $reportPath"
if ($report.failed -gt 0) { exit 1 }
exit 0
