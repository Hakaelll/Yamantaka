[CmdletBinding()]
param(
    [string]$ReleaseDirectory = 'build/release/0.13.0+14',
    [string]$ArchiveName = 'yamantaka-0.13.0+14-windows-x64.zip',
    [string]$ExpectedVersion = '0.13.0',
    [string]$ExpectedBuildNumber = '14'
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
if ($manifest.productVersion -ne $ExpectedVersion -or
    $manifest.buildNumber -ne $ExpectedBuildNumber) {
    throw "Unexpected package version/build: $($manifest.productVersion)+$($manifest.buildNumber)"
}
if ($manifest.containsPdfFiles -ne $false) {
    throw 'Manifest must declare containsPdfFiles=false.'
}
$sbomPath = Join-Path $releaseRoot 'yamantaka/SBOM.cdx.json'
if (-not (Test-Path -LiteralPath $sbomPath)) {
    throw 'CycloneDX SBOM is missing from the packaged bundle.'
}
$sbom = Get-Content -Raw -Encoding UTF8 -LiteralPath $sbomPath | ConvertFrom-Json
if ($sbom.bomFormat -ne 'CycloneDX' -or $sbom.specVersion -ne '1.5') {
    throw 'Packaged SBOM is not CycloneDX 1.5.'
}
$sbomNames = @($sbom.components | ForEach-Object { $_.name })
foreach ($component in @($sbom.components)) {
    if ([string]::IsNullOrWhiteSpace([string]$component.version) -or
        ([string]$component.version).StartsWith('unknown-', [StringComparison]::OrdinalIgnoreCase)) {
        throw "SBOM component has no resolved version: $($component.name)"
    }
    $componentLicenses = @($component.licenses)
    if ($componentLicenses.Count -eq 0 -or
        (($componentLicenses | ConvertTo-Json -Compress) -match 'NOASSERTION')) {
        throw "SBOM component has no resolved license: $($component.name)"
    }
    $componentHash = @($component.hashes | Where-Object { $_.alg -eq 'SHA-256' }) |
        Select-Object -First 1 -ExpandProperty content
    if ([string]::IsNullOrWhiteSpace([string]$componentHash)) {
        throw "SBOM component has no SHA-256: $($component.name)"
    }
    $source = @($component.properties | Where-Object { $_.name -eq 'yamantaka:source' }) |
        Select-Object -First 1 -ExpandProperty value
    if ([string]::IsNullOrWhiteSpace([string]$source)) {
        throw "SBOM component has no source provenance: $($component.name)"
    }
    if ($component.type -eq 'file') {
        $provenance = @($component.properties | Where-Object { $_.name -eq 'yamantaka:provenance' }) |
            Select-Object -First 1 -ExpandProperty value
        if ([string]::IsNullOrWhiteSpace([string]$provenance)) {
            throw "Native SBOM component has no build provenance: $($component.name)"
        }
    }
}
foreach ($binary in $manifest.nativeBinaries) {
    if ($sbomNames -notcontains $binary.path) {
        throw "Native binary is absent from SBOM: $($binary.path)"
    }
    $component = @($sbom.components | Where-Object { $_.name -eq $binary.path }) |
        Select-Object -First 1
    $expectedHash = @($component.hashes | Where-Object { $_.alg -eq 'SHA-256' }) |
        Select-Object -First 1 -ExpandProperty content
    $binaryPath = Join-Path (Join-Path $releaseRoot 'yamantaka') $binary.path
    $actualHash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expectedHash -ne $actualHash) {
        throw "Native binary hash differs from SBOM: $($binary.path)"
    }
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

$requiredReleaseFiles = @(
    'package-manifest.json',
    $ArchiveName,
    'signature-state.json',
    'zip-no-network.json',
    'no-network.json',
    'test-report.json',
    'performance-report.json'
)
$hashRows = Get-Content -Encoding UTF8 -LiteralPath (Join-Path $releaseRoot 'SHA256SUMS.txt')
$hashedNames = @{}
foreach ($row in $hashRows) {
    if ($row -notmatch '^([0-9a-fA-F]{64})\s{2}(.+)$') {
        throw "Malformed checksum row: $row"
    }
    $expectedHash = $Matches[1].ToLowerInvariant()
    $name = $Matches[2]
    if ([System.IO.Path]::GetFileName($name) -ne $name) {
        throw "Checksum paths must remain at release root: $name"
    }
    if ($hashedNames.ContainsKey($name)) {
        throw "Duplicate checksum row for $name."
    }
    $hashedNames[$name] = $true
    $candidate = Join-Path $releaseRoot $name
    if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) {
        throw "Checksummed release file not found: $name"
    }
    $actualHash = (Get-FileHash -LiteralPath $candidate -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $expectedHash) { throw "Release checksum mismatch for $name." }
}
foreach ($name in $requiredReleaseFiles) {
    if (-not $hashedNames.ContainsKey($name)) {
        throw "Required release file is not checksummed: $name"
    }
}

$zipSmoke = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $releaseRoot 'zip-no-network.json') |
    ConvertFrom-Json
if (-not $zipSmoke.passed -or @($zipSmoke.connections).Count -ne 0 -or
    $zipSmoke.navigation.sentKeyCount -ne $zipSmoke.navigation.requestedKeyCount -or
    -not $zipSmoke.processExited) {
    throw 'The executable extracted from the ZIP did not pass its offline navigation smoke.'
}
$testReport = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $releaseRoot 'test-report.json') |
    ConvertFrom-Json
if ($testReport.status -ne 'passed' -or $testReport.skippedPhaseCount -ne 0 -or
    @($testReport.phases | Where-Object { $_.status -ne 'passed' }).Count -ne 0) {
    throw 'The packaged verification report is not a complete passing gate.'
}
$signatureState = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $releaseRoot 'signature-state.json') |
    ConvertFrom-Json
if ([bool]$manifest.signed -ne [bool]$signatureState.readyForNativeExecution) {
    throw 'Manifest and signature evidence disagree about signed state.'
}

$archiveHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Host "Verified $($manifest.files.Count) package files against ZIP paths, sizes, and SHA-256."
Write-Host 'Verified that the archive contains no PDF document or unsafe path.'
Write-Host "Verified $($hashedNames.Count) release hashes and complete gate/smoke/signature evidence."
Write-Host "Archive SHA-256: $archiveHash"
