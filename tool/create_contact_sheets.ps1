[CmdletBinding()]
param(
    [string]$InputDirectory = 'build/visual-evidence/v0.11.0',
    [string]$OutputDirectory = 'build/visual-review-011',
    [ValidateRange(1, 24)][int]$MaxPerSheet = 12
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

function Get-RelativeEvidencePath {
    param(
        [Parameter(Mandatory = $true)][string]$BasePath,
        [Parameter(Mandatory = $true)][string]$Path
    )
    return $Path.Substring($BasePath.Length).TrimStart([char[]]'\/')
}

$inputRoot = Resolve-RepositoryPath $InputDirectory
$outputRoot = Resolve-RepositoryPath $OutputDirectory
if (-not (Test-Path -LiteralPath $inputRoot -PathType Container)) {
    throw "Visual evidence directory not found: $inputRoot"
}
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
Add-Type -AssemblyName System.Drawing

$files = @(Get-ChildItem -LiteralPath $inputRoot -Recurse -File -Filter '*.png' | Sort-Object FullName)
if ($files.Count -eq 0) { throw "No PNG evidence found under: $inputRoot" }

$cellWidth = 500
$cellHeight = 330
$columns = 3
$font = [System.Drawing.Font]::new('Segoe UI', 10, [System.Drawing.FontStyle]::Regular)
$brush = [System.Drawing.Brushes]::White
$manifestSheets = [System.Collections.Generic.List[object]]::new()

try {
    $groups = $files | Group-Object {
        $relative = Get-RelativeEvidencePath -BasePath $inputRoot -Path $_.FullName
        $relative.Split([System.IO.Path]::DirectorySeparatorChar)[0]
    }
    foreach ($group in $groups) {
        $surface = $group.Name
        $surfaceFiles = @($group.Group)
        for ($offset = 0; $offset -lt $surfaceFiles.Count; $offset += $MaxPerSheet) {
            $last = [Math]::Min($offset + $MaxPerSheet - 1, $surfaceFiles.Count - 1)
            $chunk = @($surfaceFiles[$offset..$last])
            $rows = [Math]::Ceiling($chunk.Count / $columns)
            $canvas = [System.Drawing.Bitmap]::new($cellWidth * $columns, $cellHeight * $rows)
            try {
                $graphics = [System.Drawing.Graphics]::FromImage($canvas)
                try {
                    $graphics.Clear([System.Drawing.Color]::FromArgb(24, 24, 24))
                    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                    for ($index = 0; $index -lt $chunk.Count; $index++) {
                        $source = [System.Drawing.Bitmap]::new($chunk[$index].FullName)
                        try {
                            $column = $index % $columns
                            $row = [Math]::Floor($index / $columns)
                            $x = $column * $cellWidth
                            $y = $row * $cellHeight
                            $label = Get-RelativeEvidencePath -BasePath $inputRoot -Path $chunk[$index].FullName
                            $graphics.DrawString($label, $font, $brush, $x + 10, $y + 8)
                            $availableWidth = $cellWidth - 20
                            $availableHeight = $cellHeight - 44
                            $scale = [Math]::Min($availableWidth / $source.Width, $availableHeight / $source.Height)
                            $width = [Math]::Max(1, [Math]::Round($source.Width * $scale))
                            $height = [Math]::Max(1, [Math]::Round($source.Height * $scale))
                            $imageX = $x + [Math]::Floor(($cellWidth - $width) / 2)
                            $imageY = $y + 36 + [Math]::Floor(($availableHeight - $height) / 2)
                            $graphics.DrawImage($source, $imageX, $imageY, $width, $height)
                        }
                        finally {
                            $source.Dispose()
                        }
                    }
                }
                finally {
                    $graphics.Dispose()
                }
                $sheetNumber = [Math]::Floor($offset / $MaxPerSheet) + 1
                $sheetPath = Join-Path $outputRoot "$surface-$sheetNumber.png"
                $canvas.Save($sheetPath, [System.Drawing.Imaging.ImageFormat]::Png)
                $manifestSheets.Add([ordered]@{
                        surface = $surface
                        sheet = $sheetNumber
                        path = $sheetPath
                        sha256 = (Get-FileHash -LiteralPath $sheetPath -Algorithm SHA256).Hash.ToLowerInvariant()
                        sources = @($chunk | ForEach-Object {
                                [ordered]@{
                                    path = $_.FullName
                                    sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                                }
                            })
                    })
            }
            finally {
                $canvas.Dispose()
            }
        }
    }
}
finally {
    $font.Dispose()
}

$manifest = [ordered]@{
    format = 'yamantaka-visual-contact-sheets'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    inputDirectory = $inputRoot
    imageCount = $files.Count
    sheets = @($manifestSheets)
}
$manifestPath = Join-Path $outputRoot 'contact-sheets.json'
$manifest | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 -LiteralPath $manifestPath
Write-Host "Visual contact sheets: $manifestPath ($($files.Count) source images)"
