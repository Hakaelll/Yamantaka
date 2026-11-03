[CmdletBinding()]
param([Parameter(Mandatory)][string]$Executable)
$ErrorActionPreference = 'Stop'
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$directory = (Resolve-Path -LiteralPath (Join-Path $repository 'build/verification')).Path
foreach ($ancestor in @($repository, (Join-Path $repository 'build'), $directory)) {
    if ((Get-Item -LiteralPath $ancestor).Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw 'Preview management requires ordinary workspace directories.'
    }
}
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
function Test-PreviewPath([string]$Path) {
    return $Path -and [IO.Path]::GetDirectoryName($Path).Equals($directory, [StringComparison]::OrdinalIgnoreCase) -and
        [IO.Path]::GetFileName($Path) -cmatch '^yamantaka-[a-z0-9-]+\.exe$'
}
if (-not (Test-PreviewPath $executablePath) -or
    ((Get-Item -LiteralPath $executablePath).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    throw 'Use an ordinary frozen Yamantaka executable directly inside build/verification.'
}
$closed = @()
foreach ($entry in @(Get-CimInstance Win32_Process -Filter "Name LIKE 'yamantaka%'")) {
    if (-not (Test-PreviewPath $entry.ExecutablePath) -or
        $entry.CommandLine -notmatch '(?:^|\s)--demo(?:\s|$)' -or
        $entry.CommandLine -notmatch '(?:^|\s)--demo-window(?:\s|$)') { continue }
    $old = Get-Process -Id $entry.ProcessId -ErrorAction SilentlyContinue
    if (-not $old) { continue }
    if (-not $old.Path.Equals($entry.ExecutablePath, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Preview identity changed; no process was closed.'
    }
    if ($old.MainWindowHandle -ne [IntPtr]::Zero) { [void]$old.CloseMainWindow() }
    if (-not $old.WaitForExit(5000)) {
        # Only the verified synthetic preview may need forced cleanup.
        $old.Kill()
        if (-not $old.WaitForExit(5000)) { throw 'The previous test preview has not exited.' }
    }
    $closed += $entry.ProcessId
}
$log = Join-Path $directory ('preview-' + [guid]::NewGuid().ToString('N') + '.log')
$preview = Start-Process -FilePath $executablePath -ArgumentList @('--demo', '--demo-window', '--pdf-covers', '--es') `
    -WorkingDirectory $repository -WindowStyle Normal -RedirectStandardError $log -PassThru
[pscustomobject]@{ closedTestPreviews = $closed.Count; previewProcessId = $preview.Id } | ConvertTo-Json
# Keep the parent alive while the user reviews the application.
$preview.WaitForExit()
