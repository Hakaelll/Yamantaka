[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string]$BundlePath = 'build/windows/x64/runner/Release',
    [Parameter(Mandatory = $true)][string]$CertificateThumbprint,
    [string]$SignToolPath,
    [string]$TimestampUrl = 'http://timestamp.digicert.com',
    [switch]$SkipTimestamp,
    [switch]$MachineCertificateStore
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'windows_binary_trust.ps1')

$bundle = if ([System.IO.Path]::IsPathRooted($BundlePath)) {
    [System.IO.Path]::GetFullPath($BundlePath)
}
else {
    [System.IO.Path]::GetFullPath((Join-Path $root $BundlePath))
}
$thumbprint = $CertificateThumbprint.Replace(' ', '').ToUpperInvariant()
if ($thumbprint -notmatch '^[0-9A-F]{40}$') {
    throw 'CertificateThumbprint must be a 40-character SHA-1 certificate thumbprint.'
}
$signTool = Resolve-YamantakaSignTool -SignToolPath $SignToolPath
$binaries = @(Get-YamantakaWindowsNativeBinaries -BundlePath $bundle)

foreach ($binary in $binaries) {
    $current = Get-AuthenticodeSignature -LiteralPath $binary.FullName
    if ($current.Status -eq [System.Management.Automation.SignatureStatus]::Valid) {
        Write-Host "Already signed: $($binary.FullName)"
        continue
    }

    $arguments = @('sign', '/sha1', $thumbprint, '/s', 'My', '/fd', 'SHA256')
    if ($MachineCertificateStore) { $arguments += '/sm' }
    if (-not $SkipTimestamp) {
        $arguments += @('/tr', $TimestampUrl, '/td', 'SHA256')
    }
    $arguments += $binary.FullName

    if ($PSCmdlet.ShouldProcess($binary.FullName, 'Apply Authenticode signature')) {
        & $signTool @arguments
        if ($LASTEXITCODE -ne 0) {
            throw "SignTool failed for $($binary.FullName) with exit code $LASTEXITCODE."
        }
        & $signTool verify /pa /all $binary.FullName
        if ($LASTEXITCODE -ne 0) {
            throw "Authenticode verification failed for $($binary.FullName)."
        }
    }
}

if ($WhatIfPreference) {
    Write-Host 'WhatIf completed; signatures were not changed.'
    exit 0
}

$state = Assert-YamantakaWindowsBundleSigned -BundlePath $bundle
Write-Host "Signed Windows bundle: $($state.validSignatureCount)/$($state.binaryCount) native binaries valid."
Write-Warning 'A locally valid signature is only a preflight. Smart App Control makes the final trust decision and requires a trusted RSA provider or cloud reputation.'
