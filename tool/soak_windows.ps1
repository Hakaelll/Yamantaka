[CmdletBinding()]
param(
    [string]$ExecutablePath = 'build/windows/x64/runner/Release/yamantaka.exe',
    [string]$FixturePdf = 'build/verification/corpus/normal.pdf',
    [int]$Cycles = 30,
    [int]$SecondsPerCycle = 3,
    [string]$OutputPath = 'build/verification/soak-report.json',
    [switch]$RequireSignature
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'windows_binary_trust.ps1')
$executable = [System.IO.Path]::GetFullPath((Join-Path $root $ExecutablePath))
$fixture = [System.IO.Path]::GetFullPath((Join-Path $root $FixturePdf))
$reportPath = [System.IO.Path]::GetFullPath((Join-Path $root $OutputPath))
if (-not (Test-Path $executable)) { throw "Executable not found: $executable" }
if (-not (Test-Path $fixture)) { throw "Fixture not found: $fixture" }
if ($Cycles -lt 1 -or $Cycles -gt 1000) { throw 'Cycles must be between 1 and 1000.' }
if ($SecondsPerCycle -lt 1 -or $SecondsPerCycle -gt 3600) { throw 'SecondsPerCycle must be between 1 and 3600.' }
[void](Assert-YamantakaWindowsBundleRunnable `
        -BundlePath (Split-Path -Parent $executable) `
        -RepositoryRoot $root `
        -RequireSignature:$RequireSignature)

$samples = [System.Collections.Generic.List[object]]::new()
for ($cycle = 1; $cycle -le $Cycles; $cycle++) {
    $process = Start-Process -FilePath $executable -ArgumentList @($fixture) -PassThru -WorkingDirectory (Split-Path $executable -Parent)
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $peakRss = 0L
    $peakHandles = 0L
    try {
        while ($stopwatch.Elapsed.TotalSeconds -lt $SecondsPerCycle) {
            if ($process.HasExited) { throw "Yamantaka exited during soak cycle $cycle." }
            $process.Refresh()
            $peakRss = [Math]::Max($peakRss, [int64]$process.WorkingSet64)
            $peakHandles = [Math]::Max($peakHandles, [int64]$process.HandleCount)
            Start-Sleep -Milliseconds 250
        }
    }
    finally {
        if (-not $process.HasExited) {
            Stop-Process -Id $process.Id -Force
            $process.WaitForExit(5000)
        }
    }
    $samples.Add([ordered]@{
        cycle = $cycle
        peakRssBytes = $peakRss
        peakHandles = $peakHandles
        closed = $process.HasExited
        durationMs = $stopwatch.ElapsedMilliseconds
    })
    Write-Host "soak cycle $cycle/$Cycles rss=$peakRss handles=$peakHandles"
}

$rss = @($samples | ForEach-Object { $_.peakRssBytes })
$handles = @($samples | ForEach-Object { $_.peakHandles })
$report = [ordered]@{
    format = 'yamantaka-windows-soak'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    requireSignature = [bool]$RequireSignature
    cycles = $Cycles
    samples = @($samples)
    peakRssBytes = ($rss | Measure-Object -Maximum).Maximum
    finalRssBytes = $rss[-1]
    peakHandles = ($handles | Measure-Object -Maximum).Maximum
    finalHandles = $handles[-1]
    monotonicResourceGrowth = ($rss[-1] -gt $rss[0] * 1.25 -and $handles[-1] -gt $handles[0] * 1.25)
}
New-Item -ItemType Directory -Force -Path (Split-Path $reportPath -Parent) | Out-Null
$report | ConvertTo-Json -Depth 10 | Set-Content -Encoding UTF8 $reportPath
Write-Host "Soak evidence: $reportPath"
if ($report.monotonicResourceGrowth) { exit 1 }
exit 0
