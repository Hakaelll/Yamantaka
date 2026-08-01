[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$ExecutablePath,
    [Parameter(Mandatory = $true)][string]$FixturePath,
    [int]$DurationSeconds = 8,
    [string]$OutputPath = 'build/verification/no-network.json'
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$executable = [System.IO.Path]::GetFullPath((Join-Path $root $ExecutablePath))
$fixture = [System.IO.Path]::GetFullPath((Join-Path $root $FixturePath))
$reportPath = [System.IO.Path]::GetFullPath((Join-Path $root $OutputPath))
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

$process = Start-Process -FilePath $executable -ArgumentList @($fixture) -PassThru `
    -WorkingDirectory (Split-Path $executable -Parent)
$connections = @()
try {
    $deadline = [DateTime]::UtcNow.AddSeconds($DurationSeconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($process.HasExited) { break }
        $process.Refresh()
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
        Stop-Process -Id $process.Id -Force
        $process.WaitForExit(5000)
    }
}

$report = [ordered]@{
    format = 'yamantaka-zero-network-smoke'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    durationSeconds = $DurationSeconds
    processExited = $process.HasExited
    connections = @($connections | Sort-Object protocol, state, localAddress, localPort, remoteAddress, remotePort -Unique)
    passed = $connections.Count -eq 0
}
New-Item -ItemType Directory -Force -Path (Split-Path $reportPath -Parent) | Out-Null
$report | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $reportPath
Write-Host "Zero-network evidence: $reportPath"
if (-not $report.passed) { exit 1 }
exit 0
