[CmdletBinding()]
param(
    [string]$OutputPath = 'build/visual-evidence/v0.13.0/branding/hash-proof.json'
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$output = if ([System.IO.Path]::IsPathRooted($OutputPath)) {
    [System.IO.Path]::GetFullPath($OutputPath)
}
else {
    [System.IO.Path]::GetFullPath((Join-Path $root $OutputPath))
}
$assets = @(
    'assets/branding/yamantaka-logo-purple.png',
    'web/icons/Icon-512.png',
    'web/icons/Icon-192.png',
    'web/icons/Icon-maskable-512.png',
    'web/icons/Icon-maskable-192.png',
    'web/favicon.png',
    'web/favicon.ico',
    'windows/runner/resources/app_icon.ico'
)

$rows = @(foreach ($asset in $assets) {
        $path = Join-Path $root $asset
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Brand asset not found: $asset"
        }
        $currentBlob = (& git -C $root hash-object -- $asset).Trim()
        if ($LASTEXITCODE -ne 0) { throw "Could not hash brand asset: $asset" }
        $headBlob = (& git -C $root rev-parse "HEAD:$asset").Trim()
        if ($LASTEXITCODE -ne 0) { throw "Could not read baseline brand asset: $asset" }
        [ordered]@{
            path = $asset
            sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
            currentGitBlob = $currentBlob
            baselineGitBlob = $headBlob
            matchesBaseline = $currentBlob -eq $headBlob
        }
    })

$report = [ordered]@{
    format = 'yamantaka-brand-hash-proof'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    baselineVersion = '0.11.0'
    currentVersion = '0.13.0'
    geometryPolicy = 'Assets must remain byte-identical to the tracked 0.11 baseline.'
    status = if (@($rows | Where-Object { -not $_.matchesBaseline }).Count -eq 0) { 'pass' } else { 'fail' }
    assets = $rows
}
New-Item -ItemType Directory -Force -Path (Split-Path $output -Parent) | Out-Null
$report | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 -LiteralPath $output
Write-Host "Brand hash proof: $output ($($report.status))"
if ($report.status -ne 'pass') { exit 1 }
