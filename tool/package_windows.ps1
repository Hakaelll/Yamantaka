[CmdletBinding()]
param(
    [string]$OutputDirectory = 'build/release/0.7.0+8',
    [string]$Version = '0.7.0',
    [string]$BuildNumber = '8',
    [string]$FlutterExecutable,
    [string]$CertificateThumbprint,
    [string]$SignToolPath,
    [string]$TimestampUrl = 'http://timestamp.digicert.com',
    [switch]$SkipTimestamp,
    [switch]$MachineCertificateStore,
    [switch]$RequireSignature,
    [switch]$Portable
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'windows_binary_trust.ps1')
$outputRoot = [System.IO.Path]::GetFullPath((Join-Path $root $OutputDirectory))
$bundle = Join-Path $outputRoot 'yamantaka'
$failurePath = Join-Path $outputRoot 'release-failure.json'
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
$toolStateRoot = Join-Path $root 'build/verification/tool-state'
New-Item -ItemType Directory -Force -Path $toolStateRoot | Out-Null
$env:APPDATA = $toolStateRoot
$env:LOCALAPPDATA = $toolStateRoot
$env:GIT_CONFIG_COUNT = '2'
$env:GIT_CONFIG_KEY_0 = 'safe.directory'
$env:GIT_CONFIG_VALUE_0 = $root.Replace('\', '/')
$env:GIT_CONFIG_KEY_1 = 'safe.directory'
$env:GIT_CONFIG_VALUE_1 = (Join-Path $root '.flutter-sdk').Replace('\', '/')
# Codex can inherit both PATH and Path; collapse them before launching Flutter.
$processPath = [System.Environment]::GetEnvironmentVariable('Path', 'Process')
[System.Environment]::SetEnvironmentVariable('PATH', $null, 'Process')
[System.Environment]::SetEnvironmentVariable('Path', $processPath, 'Process')
if (-not $FlutterExecutable) {
    $localFlutter = Join-Path $root '.flutter-sdk/bin/flutter.bat'
    $FlutterExecutable = if (Test-Path $localFlutter) {
        $localFlutter
    }
    else {
        $command = Get-Command flutter -ErrorAction SilentlyContinue
        if (-not $command) { throw 'flutter was not found on PATH or .flutter-sdk.' }
        $command.Source
    }
}

function Assert-PubPackageRoots {
    $packageConfigPath = Join-Path $root '.dart_tool/package_config.json'
    if (-not (Test-Path -LiteralPath $packageConfigPath)) {
        throw "Pub package configuration not found: $packageConfigPath"
    }
    $config = Get-Content -Raw -Encoding UTF8 -LiteralPath $packageConfigPath |
        ConvertFrom-Json
    $missing = @(
        $config.packages |
            Where-Object { $_.rootUri -like 'file:*' } |
            ForEach-Object {
                $uri = [System.Uri]$_.rootUri
                $path = $uri.LocalPath
                $available = $false
                try {
                    # Test-Path can report an ACL-protected directory as
                    # present; opening the root makes the preflight useful on
                    # locked-down workstations too.
                    Get-ChildItem -LiteralPath $path -ErrorAction Stop |
                        Select-Object -First 1 | Out-Null
                    $available = $true
                }
                catch {
                    $available = $false
                }
                if (-not $available) {
                    $_.name
                }
            }
    )
    if ($missing.Count -gt 0) {
        $names = ($missing | Sort-Object -Unique) -join ', '
        throw "Pub dependency roots are unavailable: $names"
    }
}

Push-Location $root
try {
    try {
        Assert-PubPackageRoots
        & $FlutterExecutable build windows --release --build-name $Version --build-number $BuildNumber --no-pub
        $buildExitCode = $LASTEXITCODE
        if ($buildExitCode -ne 0) { throw "Flutter release build failed with exit code $buildExitCode." }
    }
    catch {
        $failure = [ordered]@{
            format = 'yamantaka-windows-package-failure'
            version = 1
            generatedAt = [DateTime]::UtcNow.ToString('o')
            productVersion = $Version
            buildNumber = $BuildNumber
            flutterExecutable = $FlutterExecutable
            command = 'flutter build windows --release --build-name <Version> --build-number <BuildNumber> --no-pub'
            exitCode = $buildExitCode
            error = $_.Exception.Message
        }
        $failure | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $failurePath
        Write-Host "Release failure report: $failurePath" -ForegroundColor Red
        throw
    }
    if (Test-Path -LiteralPath $failurePath) {
        Remove-Item -LiteralPath $failurePath -Force
    }
    $releaseRoot = Join-Path $root 'build/windows/x64/runner/Release'
    if (-not (Test-Path $releaseRoot)) { throw "Release directory not found: $releaseRoot" }
    if (Test-Path $bundle) { Remove-Item -LiteralPath $bundle -Recurse -Force }
    Copy-Item -LiteralPath $releaseRoot -Destination $bundle -Recurse
    Copy-Item -LiteralPath (Join-Path $root 'third_party/pdfrx/LICENSE') -Destination (Join-Path $bundle 'THIRD_PARTY_PDfrx_LICENSE.txt')
    Copy-Item -LiteralPath (Join-Path $root 'docs/release/WINDOWS-BUNDLE.txt') -Destination (Join-Path $bundle 'LEEME.txt')
    if ($Portable) {
        New-Item -ItemType File -Force -Path (Join-Path $bundle '.yamantaka-portable') | Out-Null
    }
    if ($CertificateThumbprint) {
        $signingArguments = @{
            BundlePath = $bundle
            CertificateThumbprint = $CertificateThumbprint
            TimestampUrl = $TimestampUrl
        }
        if ($SignToolPath) { $signingArguments.SignToolPath = $SignToolPath }
        if ($SkipTimestamp) { $signingArguments.SkipTimestamp = $true }
        if ($MachineCertificateStore) {
            $signingArguments.MachineCertificateStore = $true
        }
        & (Join-Path $PSScriptRoot 'sign_windows_bundle.ps1') @signingArguments
        if ($LASTEXITCODE -ne 0) {
            throw "Windows bundle signing failed with exit code $LASTEXITCODE."
        }
    }
    $signatureState = Get-YamantakaWindowsBundleSignatureState -BundlePath $bundle
    $signed = $signatureState.readyForNativeExecution
    if ($RequireSignature -and -not $signed) {
        $invalid = @(
            $signatureState.binaries |
                Where-Object { -not $_.valid } |
                ForEach-Object { "$($_.relativePath) [$($_.status)]" }
        ) -join ', '
        throw "A fully signed package was required, but these native binaries are not valid: $invalid"
    }
    $commit = (git rev-parse HEAD).Trim()
    $dirty = @((git status --porcelain)).Count -gt 0
    $files = Get-ChildItem -LiteralPath $bundle -File -Recurse | ForEach-Object {
        [ordered]@{
            path = $_.FullName.Substring($bundle.Length + 1).Replace('\', '/')
            bytes = $_.Length
            sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    }
    $manifest = [ordered]@{
        format = 'yamantaka-windows-package'
        version = 1
        productVersion = $Version
        buildNumber = $BuildNumber
        commit = $commit
        dirty = $dirty
        generatedAt = [DateTime]::UtcNow.ToString('o')
        signed = $signed
        nativeBinaryCount = $signatureState.binaryCount
        validSignatureCount = $signatureState.validSignatureCount
        signatureSubjects = @(
            $signatureState.binaries.subject |
                Where-Object { $_ } |
                Sort-Object -Unique
        )
        nativeBinaries = @(
            $signatureState.binaries | ForEach-Object {
                [ordered]@{
                    path = $_.relativePath
                    status = $_.status
                    valid = $_.valid
                    subject = $_.subject
                    thumbprint = $_.thumbprint
                }
            }
        )
        distribution = if ($Portable) { 'portable' } else { 'installed-bundle' }
        dataIsExternal = $true
        containsPdfFiles = $false
        files = @($files)
    }
    $manifestPath = Join-Path $outputRoot 'package-manifest.json'
    $manifest | ConvertTo-Json -Depth 10 | Set-Content -Encoding UTF8 $manifestPath
    $archiveName = if ($Portable) {
        "yamantaka-$Version+$BuildNumber-windows-x64-portable.zip"
    }
    else {
        "yamantaka-$Version+$BuildNumber-windows-x64.zip"
    }
    $archive = Join-Path $outputRoot $archiveName
    if (Test-Path $archive) { Remove-Item -LiteralPath $archive -Force }
    Compress-Archive -Path (Join-Path $bundle '*') -DestinationPath $archive
    $hashesPath = Join-Path $outputRoot 'SHA256SUMS.txt'
    @($manifestPath, $archive) | ForEach-Object {
        $hash = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $([System.IO.Path]::GetFileName($_))"
    } | Set-Content -Encoding UTF8 $hashesPath
    Write-Host "Package manifest: $manifestPath"
    Write-Host "Package archive: $archive"
    Write-Host "Release hashes: $hashesPath"
}
finally {
    Pop-Location
}
