param(
    [Parameter(Mandatory)][string]$Baseline,
    [string]$Candidate,
    [Parameter(Mandatory)][string]$Pdf,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateRange(1, 30)][int]$Runs = 5
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/wait_rust_test_process.ps1"
$repository = (Resolve-Path -LiteralPath "$PSScriptRoot/..").Path
$env:YAMANTAKA_QA_UI_ROOT = Join-Path $repository 'rust/ui'
$pdfPath = (Resolve-Path -LiteralPath $Pdf).Path
$pdfHash = (Get-FileHash -LiteralPath $pdfPath).Hash
$builds = [ordered]@{ baseline = (Resolve-Path -LiteralPath $Baseline).Path }
if ($Candidate) { $builds.candidate = (Resolve-Path -LiteralPath $Candidate).Path }
$identities = [ordered]@{}
foreach ($build in $builds.GetEnumerator()) {
    $identities[$build.Key] = [ordered]@{
        binary = (Get-FileHash -LiteralPath $build.Value).Hash
        pdfium = (Get-FileHash -LiteralPath (Join-Path (Split-Path $build.Value) 'pdfium.dll')).Hash
    }
    if ($identities[$build.Key].pdfium -ne $identities.baseline.pdfium) { throw 'Use the same PDFium for both builds.' }
}
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Choose a new output directory for each measurement.' }
$output = (New-Item -ItemType Directory -Path $OutputDirectory).FullName
$measurements = @()
for ($run = 1; $run -le $Runs; $run++) {
    foreach ($build in $builds.GetEnumerator()) {
        $name = "$($build.Key)-$run"
        $frame = Join-Path $output "$name.png"
        $stderr = Join-Path $output "$name.stderr.log"
        $process = Start-Process -FilePath $build.Value -ArgumentList @(
            '--demo', ('"--capture={0}"' -f $frame), ('"--measure-reader-pdf={0}"' -f $pdfPath),
            '--size=1440x960', '--reduce-motion'
        ) -WorkingDirectory $repository -WindowStyle Hidden -RedirectStandardError $stderr -PassThru
        $timer = [Diagnostics.Stopwatch]::StartNew()
        $peak = 0L
        try {
            while (-not $process.WaitForExit(100) -and $timer.Elapsed.TotalSeconds -lt 60) {
                $process.Refresh()
                try { $peak = [Math]::Max($peak, $process.PeakWorkingSet64) }
                catch { if (-not $process.HasExited) { throw } }
            }
        } finally {
            Wait-RustTestProcess -Process $process -TimeoutMilliseconds 1000
        }
        if ($process.ExitCode -ne 0) { throw "Reader measurement failed for $name. See $stderr." }
        $controls = Get-Content -LiteralPath ([IO.Path]::ChangeExtension($frame,'controls.json')) -Raw | ConvertFrom-Json
        $report = $controls.readerPerformance
        if (-not $controls.synthetic -or $controls.pagePresenter -ne 'native-wgpu' -or
            ($report.samples.action -join ',') -cne 'open,next,next,next,previous,zoom,resume' -or
            ($report.samples.page -join ',') -cne '0,1,2,3,2,2,2' -or $report.zoomTickMs.Count -ne 7) {
            throw "Reader measurement is incomplete for $name."
        }
        foreach ($sample in $report.samples) {
            if ($sample.visible_ms -lt 0 -or $sample.sharp_ms -lt $sample.visible_ms) { throw "Invalid timing for $name." }
        }
        $measurements += [ordered]@{
            build = $build.Key; run = $run; viewport = $controls.viewport
            pixelsPerPoint = $controls.pixelsPerPoint; peakWorkingSetBytes = $peak
            samples = $report.samples; zoomTickMs = $report.zoomTickMs
        }
        Write-Output "Measured $name with a new reader process."
    }
}
if ((Get-FileHash -LiteralPath $pdfPath).Hash -ne $pdfHash) { throw 'The input PDF changed during measurement.' }
[ordered]@{ pdfSha256 = $pdfHash; builds = $identities; measurements = $measurements } |
    ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $output 'reader-performance.json') -Encoding utf8
