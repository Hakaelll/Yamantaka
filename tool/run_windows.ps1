[CmdletBinding()]
param(
    [string]$ExecutablePath = 'build/windows/x64/runner/Release/yamantaka.exe',
    [string]$PdfPath,
    [switch]$Wait,
    [switch]$RequireSignature
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'windows_binary_trust.ps1')

$executable = if ([System.IO.Path]::IsPathRooted($ExecutablePath)) {
    [System.IO.Path]::GetFullPath($ExecutablePath)
}
else {
    [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $ExecutablePath))
}
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "Yamantaka executable not found: $executable"
}

$bundlePath = Split-Path -Parent $executable
$executionState = Assert-YamantakaWindowsBundleRunnable `
    -BundlePath $bundlePath `
    -RepositoryRoot $repositoryRoot `
    -RequireSignature:$RequireSignature

$launchArguments = @()
if ($PdfPath) {
    $pdf = if ([System.IO.Path]::IsPathRooted($PdfPath)) {
        [System.IO.Path]::GetFullPath($PdfPath)
    }
    else {
        [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $PdfPath))
    }
    if (-not (Test-Path -LiteralPath $pdf -PathType Leaf)) {
        throw "PDF not found: $pdf"
    }
    $launchArguments = @('"' + $pdf.Replace('"', '\"') + '"')
}

$launch = @{
    FilePath = $executable
    PassThru = $true
    WorkingDirectory = $bundlePath
}
if ($launchArguments.Count -gt 0) {
    $launch.ArgumentList = $launchArguments
}

$process = Start-Process @launch
if ($Wait) {
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) {
        throw "Yamantaka exited with code $($process.ExitCode)."
    }
    Write-Host "Yamantaka exited normally (mode: $($executionState.mode))."
}
elseif ($process.WaitForExit(1500)) {
    if ($process.ExitCode -ne 0) {
        throw "Yamantaka exited during startup with code $($process.ExitCode)."
    }
    Write-Host "Yamantaka exited during startup with code 0."
}
else {
    Write-Host "Yamantaka started (PID $($process.Id), mode: $($executionState.mode))."
}
