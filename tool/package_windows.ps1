[CmdletBinding()]
param(
    [string]$OutputDirectory = 'build/release',
    [string]$Version = '0.1.0',
    [string]$BuildNumber = '1',
    [string]$FlutterExecutable,
    [string]$CertificateThumbprint,
    [switch]$RequireSignature,
    [switch]$Portable
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$outputRoot = [System.IO.Path]::GetFullPath((Join-Path $root $OutputDirectory))
$bundle = Join-Path $outputRoot 'yamantaka'
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
        $failurePath = Join-Path $outputRoot 'release-failure.json'
        $failure | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $failurePath
        Write-Host "Release failure report: $failurePath" -ForegroundColor Red
        throw
    }
    $releaseRoot = Join-Path $root 'build/windows/x64/runner/Release'
    if (-not (Test-Path $releaseRoot)) { throw "Release directory not found: $releaseRoot" }
    if (Test-Path $bundle) { Remove-Item -LiteralPath $bundle -Recurse -Force }
    Copy-Item -LiteralPath $releaseRoot -Destination $bundle -Recurse
    Copy-Item -LiteralPath (Join-Path $root 'third_party/pdfrx/LICENSE') -Destination (Join-Path $bundle 'THIRD_PARTY_PDfrx_LICENSE.txt')
    if ($Portable) {
        New-Item -ItemType File -Force -Path (Join-Path $bundle '.yamantaka-portable') | Out-Null
    }
    $signed = $false
    $executable = Get-ChildItem -LiteralPath $bundle -Filter '*.exe' -Recurse | Select-Object -First 1
    if ($CertificateThumbprint) {
        $signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
        if (-not $signtool) { throw 'Certificate thumbprint was provided but signtool.exe was not found.' }
        & $signtool.Source sign /sha1 $CertificateThumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $executable.FullName
        if ($LASTEXITCODE -ne 0) { throw "signtool failed with exit code $LASTEXITCODE." }
        $signed = $true
    }
    if ($RequireSignature -and -not $signed) { throw 'A signed package was required but no certificate was supplied.' }
    $commit = (git rev-parse HEAD).Trim()
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
        generatedAt = [DateTime]::UtcNow.ToString('o')
        signed = $signed
        distribution = if ($Portable) { 'portable' } else { 'installed-bundle' }
        dataIsExternal = $true
        containsPdfFiles = $false
        files = @($files)
    }
    $manifestPath = Join-Path $outputRoot 'package-manifest.json'
    $manifest | ConvertTo-Json -Depth 10 | Set-Content -Encoding UTF8 $manifestPath
    $archiveName = if ($Portable) { 'yamantaka-portable.zip' } else { 'yamantaka-bundle.zip' }
    $archive = Join-Path $outputRoot $archiveName
    if (Test-Path $archive) { Remove-Item -LiteralPath $archive -Force }
    Compress-Archive -Path (Join-Path $bundle '*') -DestinationPath $archive
    Write-Host "Package manifest: $manifestPath"
    Write-Host "Package archive: $archive"
}
finally {
    Pop-Location
}
