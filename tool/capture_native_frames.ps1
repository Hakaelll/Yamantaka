[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateRange(1, 2147483647)][int]$TargetProcessId,
    [Parameter(Mandatory)][string]$ExpectedExecutablePath,
    [Parameter(Mandatory)][ValidatePattern('^[a-z0-9][a-z0-9-]{0,79}$')][string]$CaptureName,
    [ValidateRange(1, 30)][int]$DurationSeconds = 10
)
$ErrorActionPreference = 'Stop'
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$verification = Join-Path $repository 'build/verification'
foreach ($ancestor in @($repository, (Join-Path $repository 'build'), $verification)) {
    if (-not (Test-Path -LiteralPath $ancestor -PathType Container) -or
        ((Get-Item -LiteralPath $ancestor).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw 'Capture requires ordinary workspace verification directories.'
    }
}
$expected = (Resolve-Path -LiteralPath $ExpectedExecutablePath).Path
if (-not $expected.StartsWith($repository + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or
    [IO.Path]::GetFileNameWithoutExtension($expected) -notlike 'yamantaka*') {
    throw 'Select a Yamantaka executable inside this workspace.'
}
$target = Get-Process -Id $TargetProcessId -ErrorAction Stop
if (-not $target.Path.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) { throw 'Target process differs from the expected executable.' }
$started = $target.StartTime
$tool = & (Join-Path $PSScriptRoot 'fetch_presentmon.ps1') -VerifyOnly | ConvertFrom-Json
$output = Join-Path $verification $CaptureName
if (Test-Path -LiteralPath $output) { throw 'Use a fresh capture name; previous evidence was preserved.' }
New-Item -ItemType Directory -Path $output | Out-Null
$sessionName = 'Yamantaka-' + [guid]::NewGuid().ToString('N')
$arguments = @('--process_id', "$TargetProcessId", '--timed', "$DurationSeconds", '--terminate_after_timed',
    '--no_track_input', '--no_console_stats', '--qpc_time', '--session_name', $sessionName, '--output_file', 'frames.csv')
$capture = Start-Process -FilePath $tool.executable -ArgumentList $arguments -WorkingDirectory $output -WindowStyle Hidden `
    -RedirectStandardOutput (Join-Path $output 'stdout.log') -RedirectStandardError (Join-Path $output 'stderr.log') -PassThru -Wait
$report = [ordered]@{
    format = 'yamantaka-presentation-observation-v1'
    targetExecutable = [IO.Path]::GetFileName($expected)
    targetSha256 = (Get-FileHash -LiteralPath $expected -Algorithm SHA256).Hash.ToLowerInvariant()
    targetProcessId = $TargetProcessId
    targetStartUtc = $started.ToUniversalTime().ToString('o')
    toolVersion = $tool.version
    toolSha256 = $tool.sha256
    requestedSeconds = $DurationSeconds
    inputTracking = $false
    traceExitCode = $capture.ExitCode
    status = 'capture-failed'
    frameRows = 0
    performanceVerdict = 'not-established'
}
if ($capture.ExitCode -eq 0) {
    $current = Get-Process -Id $TargetProcessId -ErrorAction SilentlyContinue
    if (-not $current -or $current.StartTime -ne $started -or -not $current.Path.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) {
        $report.status = 'target-changed-or-exited'
    } else {
        $csv = Join-Path $output 'frames.csv'
        if (Test-Path -LiteralPath $csv) {
            $rows = @(Import-Csv -LiteralPath $csv)
            if (@($rows | Where-Object { $_.ProcessID -ne "$TargetProcessId" }).Count -ne 0) { throw 'Trace contains a different process; evidence was preserved locally.' }
            $report.frameRows = $rows.Count
            $report.status = if ($rows.Count -gt 0) { 'frames-recorded' } else { 'no-frame-evidence' }
        } else { $report.status = 'no-frame-evidence' }
    }
}
$report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'observation.json') -Encoding utf8
$report | ConvertTo-Json
if ($capture.ExitCode -ne 0) { exit $capture.ExitCode }
if ($report.status -ne 'frames-recorded') { exit 2 }
