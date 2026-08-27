[CmdletBinding()]
param(
    [string]$ReleaseDirectory = 'build/release/0.10.0+11',
    [string]$ArchiveName = 'yamantaka-0.10.0+11-windows-x64.zip'
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$releaseRoot = [System.IO.Path]::GetFullPath((Join-Path $root $ReleaseDirectory))
$manifestPath = Join-Path $releaseRoot 'package-manifest.json'
$archivePath = Join-Path $releaseRoot $ArchiveName

if (-not $releaseRoot.StartsWith((Join-Path $root 'build'), [StringComparison]::OrdinalIgnoreCase)) {
    throw "Release directory must remain below the repository build directory: $releaseRoot"
}
if (-not (Test-Path -LiteralPath $manifestPath)) {
    throw "Package manifest not found: $manifestPath"
}
if (-not (Test-Path -LiteralPath $archivePath)) {
    throw "Package archive not found: $archivePath"
}

$manifest = Get-Content -Raw -Encoding UTF8 -LiteralPath $manifestPath |
    ConvertFrom-Json
if ($manifest.format -ne 'yamantaka-windows-package') {
    throw "Unexpected manifest format: $($manifest.format)"
}
if ($manifest.containsPdfFiles -ne $false) {
    throw 'Manifest must declare containsPdfFiles=false.'
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::OpenRead($archivePath)
try {
    $zipFiles = @(
        $archive.Entries |
            Where-Object { -not [string]::IsNullOrEmpty($_.Name) }
    )
    $zipByPath = @{}
    foreach ($entry in $zipFiles) {
        $path = $entry.FullName.Replace('\', '/')
        if ($path.StartsWith('/') -or $path.Contains('../') -or $path -match '^[A-Za-z]:') {
            throw "Unsafe archive path: $path"
        }
        if ([System.IO.Path]::GetExtension($path) -ieq '.pdf') {
            throw "PDF documents must not be bundled: $path"
        }
        if ($zipByPath.ContainsKey($path)) {
            throw "Duplicate archive path: $path"
        }
        $zipByPath[$path] = $entry
    }

    $manifestByPath = @{}
    foreach ($file in @($manifest.files)) {
        $path = ([string]$file.path).Replace('\', '/')
        if ([System.IO.Path]::GetExtension($path) -ieq '.pdf') {
            throw "PDF documents must not appear in the manifest: $path"
        }
        if ($manifestByPath.ContainsKey($path)) {
            throw "Duplicate manifest path: $path"
        }
        $manifestByPath[$path] = $file
    }

    $missingFromZip = @($manifestByPath.Keys | Where-Object { -not $zipByPath.ContainsKey($_) })
    $missingFromManifest = @($zipByPath.Keys | Where-Object { -not $manifestByPath.ContainsKey($_) })
    if ($missingFromZip.Count -gt 0 -or $missingFromManifest.Count -gt 0) {
        throw "Archive/manifest inventory differs. Missing from ZIP: $($missingFromZip -join ', '); missing from manifest: $($missingFromManifest -join ', ')"
    }

    foreach ($path in $manifestByPath.Keys) {
        $entry = $zipByPath[$path]
        $expected = $manifestByPath[$path]
        if ($entry.Length -ne [long]$expected.bytes) {
            throw "Size mismatch for ${path}: ZIP=$($entry.Length), manifest=$($expected.bytes)"
        }
        $stream = $entry.Open()
        try {
            $sha = [System.Security.Cryptography.SHA256]::Create()
            try {
                $hashBytes = $sha.ComputeHash($stream)
                $actualHash = ([BitConverter]::ToString($hashBytes)).Replace('-', '').ToLowerInvariant()
            }
            finally {
                $sha.Dispose()
            }
        }
        finally {
            $stream.Dispose()
        }
        if ($actualHash -ne [string]$expected.sha256) {
            throw "SHA-256 mismatch for $path"
        }
    }
}
finally {
    $archive.Dispose()
}

$hashRows = Get-Content -Encoding UTF8 -LiteralPath (Join-Path $releaseRoot 'SHA256SUMS.txt')
foreach ($name in @('package-manifest.json', $ArchiveName)) {
    $row = @($hashRows | Where-Object { $_ -match "  $([regex]::Escape($name))$" })
    if ($row.Count -ne 1) { throw "Expected one checksum row for $name." }
    $expectedHash = ($row[0] -split '\s+', 2)[0]
    $actualHash = (Get-FileHash -LiteralPath (Join-Path $releaseRoot $name) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $expectedHash) { throw "Release checksum mismatch for $name." }
}

$archiveHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Host "Verified $($manifest.files.Count) package files against ZIP paths, sizes, and SHA-256."
Write-Host 'Verified that the archive contains no PDF document or unsafe path.'
Write-Host "Archive SHA-256: $archiveHash"
