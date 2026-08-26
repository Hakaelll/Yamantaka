[CmdletBinding()]
param(
    [string]$BeforeDirectory = 'build/visual-evidence/v0.8.0',
    [string]$AfterDirectory = 'build/visual-evidence/v0.9.0',
    [string]$OutputDirectory = 'build/visual-evidence/v0.9.0/comparisons'
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))

function Resolve-RepositoryPath {
    param([Parameter(Mandatory = $true)][string]$Path)
    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }
    return [System.IO.Path]::GetFullPath((Join-Path $root $Path))
}

$beforeRoot = Resolve-RepositoryPath $BeforeDirectory
$afterRoot = Resolve-RepositoryPath $AfterDirectory
$outputRoot = Resolve-RepositoryPath $OutputDirectory
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
Add-Type -AssemblyName System.Drawing

$cases = @(
    @{ name = 'library-expanded'; path = 'library/library-many-1600x900-light-widget.png' },
    @{ name = 'library-compact'; path = 'library/library-many-640x480-light-widget.png' },
    @{ name = 'reader-active'; path = 'reader/reader-document-1600x900-light-widget.png' },
    @{ name = 'reader-resting'; path = 'reader/reader-document-1600x900-resting-widget.png' },
    @{ name = 'reader-panel-docked'; path = 'reader/reader-document-1600x900-panel-docked-widget.png' },
    @{ name = 'reader-compact'; path = 'reader/reader-document-640x480-light-widget.png' },
    @{ name = 'reader-high-contrast'; path = 'reader/reader-document-1280x720-high-contrast-widget.png' },
    @{ name = 'reader-reduced-motion'; path = 'reader/reader-document-1280x720-reduced-motion-widget.png' },
    @{ name = 'settings-expanded'; path = 'settings/settings-1600x900-light-widget.png' },
    @{ name = 'settings-dark'; path = 'settings/settings-1280x720-dark-widget.png' }
)

$evidence = [System.Collections.Generic.List[object]]::new()
foreach ($case in $cases) {
    $relativePath = $case.path.Replace('/', [System.IO.Path]::DirectorySeparatorChar)
    $beforePath = Join-Path $beforeRoot $relativePath
    $afterPath = Join-Path $afterRoot $relativePath
    if (-not (Test-Path -LiteralPath $beforePath)) {
        throw "Missing before evidence: $beforePath"
    }
    if (-not (Test-Path -LiteralPath $afterPath)) {
        throw "Missing after evidence: $afterPath"
    }
    $before = [System.Drawing.Bitmap]::new($beforePath)
    $after = [System.Drawing.Bitmap]::new($afterPath)
    try {
        $headerHeight = 36
        $gap = 2
        $width = $before.Width + $after.Width + $gap
        $height = [Math]::Max($before.Height, $after.Height) + $headerHeight
        $comparison = [System.Drawing.Bitmap]::new($width, $height)
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($comparison)
            try {
                $graphics.Clear([System.Drawing.Color]::FromArgb(24, 24, 24))
                $font = [System.Drawing.Font]::new('Segoe UI', 11, [System.Drawing.FontStyle]::Bold)
                $brush = [System.Drawing.Brushes]::White
                try {
                    $graphics.DrawString('0.8', $font, $brush, 12, 8)
                    $graphics.DrawString('0.9', $font, $brush, $before.Width + $gap + 12, 8)
                    $graphics.DrawImageUnscaled($before, 0, $headerHeight)
                    $graphics.DrawImageUnscaled($after, $before.Width + $gap, $headerHeight)
                }
                finally {
                    $font.Dispose()
                }
            }
            finally {
                $graphics.Dispose()
            }
            $outputPath = Join-Path $outputRoot "$($case.name)-0.8-vs-0.9.png"
            $comparison.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
        }
        finally {
            $comparison.Dispose()
        }
    }
    finally {
        $before.Dispose()
        $after.Dispose()
    }
    $evidence.Add([ordered]@{
            name = $case.name
            before = $beforePath
            beforeSha256 = (Get-FileHash -LiteralPath $beforePath -Algorithm SHA256).Hash.ToLowerInvariant()
            after = $afterPath
            afterSha256 = (Get-FileHash -LiteralPath $afterPath -Algorithm SHA256).Hash.ToLowerInvariant()
            comparison = $outputPath
            comparisonSha256 = (Get-FileHash -LiteralPath $outputPath -Algorithm SHA256).Hash.ToLowerInvariant()
            source = 'side-by-side of executed widget captures'
        })
}

$manifest = [ordered]@{
    format = 'yamantaka-visual-comparisons'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    beforeVersion = '0.8.0'
    afterVersion = '0.9.0'
    cases = @($evidence)
}
$manifestPath = Join-Path $outputRoot 'comparisons.json'
$manifest | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 $manifestPath
Write-Host "Visual comparisons: $manifestPath"
