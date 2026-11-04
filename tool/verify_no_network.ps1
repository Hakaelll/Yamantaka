[CmdletBinding()]
param(
    [string]$ExecutablePath = 'build/release/yamantaka-0.16.0/yamantaka/yamantaka.exe',
    [string]$FixturePath = 'build/verification/corpus/normal.pdf',
    [int]$DurationSeconds = 8,
    [ValidateRange(0, 20)]
    [int]$NavigationKeyCount = 2,
    [string]$OutputPath = 'build/verification/no-network.json',
    [switch]$RequireSignature
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'windows_binary_trust.ps1')

Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class YamantakaNetworkSmokeInput {
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr handle);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool PostMessage(
        IntPtr handle,
        uint message,
        IntPtr wParam,
        IntPtr lParam
    );
}
'@

function Resolve-RepositoryPath {
    param([Parameter(Mandatory = $true)][string]$Path)
    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }
    return [System.IO.Path]::GetFullPath((Join-Path $root $Path))
}

$executable = Resolve-RepositoryPath $ExecutablePath
$fixture = Resolve-RepositoryPath $FixturePath
$reportPath = Resolve-RepositoryPath $OutputPath
$toolStateRoot = Join-Path $root 'build/verification/tool-state/no-network'
New-Item -ItemType Directory -Force -Path $toolStateRoot | Out-Null
$env:APPDATA = $toolStateRoot
$env:LOCALAPPDATA = $toolStateRoot
if (-not (Test-Path $executable)) { throw "Executable not found: $executable" }
if (-not (Test-Path $fixture)) { throw "Fixture not found: $fixture" }
if (-not (Get-Command Get-NetTCPConnection -ErrorAction SilentlyContinue)) {
    throw 'Get-NetTCPConnection is required for the zero-network smoke.'
}
if (-not (Get-Command Get-NetUDPEndpoint -ErrorAction SilentlyContinue)) {
    throw 'Get-NetUDPEndpoint is required for the zero-network smoke.'
}
if ($DurationSeconds -lt 1 -or $DurationSeconds -gt 300) {
    throw 'DurationSeconds must be between 1 and 300.'
}
[void](Assert-YamantakaWindowsBundleRunnable `
        -BundlePath (Split-Path -Parent $executable) `
        -RepositoryRoot $root `
        -RequireSignature:$RequireSignature)

$process = Start-Process -FilePath $executable -ArgumentList @($fixture) -PassThru `
    -WorkingDirectory (Split-Path $executable -Parent)
$connections = @()
$navigationKeysSent = 0
$navigationErrors = [System.Collections.Generic.List[string]]::new()
$windowActivated = $false
$exitMode = 'normal'
$navigationStart = [DateTime]::UtcNow.AddSeconds([Math]::Min(2, [Math]::Max(1, $DurationSeconds / 3)))
try {
    $deadline = [DateTime]::UtcNow.AddSeconds($DurationSeconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($process.HasExited) { break }
        $process.Refresh()
        if ($navigationKeysSent -lt $NavigationKeyCount -and
            [DateTime]::UtcNow -ge $navigationStart -and
            $process.MainWindowHandle -ne [IntPtr]::Zero) {
            try {
                [void][YamantakaNetworkSmokeInput]::SetForegroundWindow(
                    $process.MainWindowHandle
                )
                $windowActivated = $true
                while ($navigationKeysSent -lt $NavigationKeyCount) {
                    if (-not [YamantakaNetworkSmokeInput]::PostMessage(
                            $process.MainWindowHandle,
                            0x0100,
                            [IntPtr]0x27,
                            [IntPtr]::Zero
                        )) {
                        throw 'Could not post ArrowRight key-down to the reader window.'
                    }
                    if (-not [YamantakaNetworkSmokeInput]::PostMessage(
                            $process.MainWindowHandle,
                            0x0101,
                            [IntPtr]0x27,
                            [IntPtr]::Zero
                        )) {
                        throw 'Could not post ArrowRight key-up to the reader window.'
                    }
                    $navigationKeysSent += 1
                    Start-Sleep -Milliseconds 150
                }
            }
            catch {
                $navigationErrors.Add($_.Exception.Message)
                $navigationStart = [DateTime]::UtcNow.AddMilliseconds(500)
            }
        }
        $connections += @(Get-NetTCPConnection -OwningProcess $process.Id `
                -ErrorAction SilentlyContinue |
            Where-Object { $_.State -in @('Established', 'Listen', 'SynSent', 'SynReceived') } |
            ForEach-Object {
                [pscustomobject]@{
                    protocol = 'tcp'
                    state = $_.State
                    localAddress = $_.LocalAddress
                    localPort = $_.LocalPort
                    remoteAddress = $_.RemoteAddress
                    remotePort = $_.RemotePort
                }
            })
        $connections += @(Get-NetUDPEndpoint -OwningProcess $process.Id `
                -ErrorAction SilentlyContinue |
            Select-Object @{Name = 'protocol'; Expression = { 'udp' } },
                @{Name = 'state'; Expression = { 'bound' } },
                LocalAddress, LocalPort,
                @{Name = 'remoteAddress'; Expression = { $null } },
                @{Name = 'remotePort'; Expression = { $null } })
        Start-Sleep -Milliseconds 250
    }
}
finally {
    if (-not $process.HasExited) {
        if ($process.CloseMainWindow() -and $process.WaitForExit(5000)) {
            $exitMode = 'harness-close'
        }
        else {
            Stop-Process -Id $process.Id -Force
            [void]$process.WaitForExit(5000)
            $exitMode = 'harness-force'
        }
    }
}

$report = [ordered]@{
    format = 'yamantaka-zero-network-smoke'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    requireSignature = [bool]$RequireSignature
    durationSeconds = $DurationSeconds
    processExited = $process.HasExited
    exitMode = $exitMode
    navigation = [ordered]@{
        requestedKeyCount = $NavigationKeyCount
        sentKeyCount = $navigationKeysSent
        key = 'ArrowRight'
        windowActivated = $windowActivated
        errors = @($navigationErrors | Select-Object -Unique)
    }
    connections = @($connections | Sort-Object protocol, state, localAddress, localPort, remoteAddress, remotePort -Unique)
    passed = $connections.Count -eq 0 -and
        $navigationKeysSent -eq $NavigationKeyCount -and
        $process.HasExited
}
New-Item -ItemType Directory -Force -Path (Split-Path $reportPath -Parent) | Out-Null
$report | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $reportPath
Write-Host "Zero-network evidence: $reportPath"
if (-not $report.passed) { exit 1 }
exit 0
