[CmdletBinding()]
param(
    [string]$ExecutablePath = 'build/windows/x64/runner/Release/yamantaka.exe',
    [string]$OutputDirectory = 'build/verification/visual',
    [string]$FixturePdf = 'build/verification/corpus/normal.pdf',
    [ValidateSet(100, 125, 150, 175, 200)]
    [int]$HostScalePercent = 100,
    [switch]$SkipLaunch,
    [switch]$ScaleConfirmed,
    [switch]$RequireSignature
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'windows_binary_trust.ps1')

function Resolve-RepositoryPath {
    param([Parameter(Mandatory = $true)][string]$Path)
    if ([System.IO.Path]::IsPathFullyQualified($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }
    return [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $Path))
}

$executable = Resolve-RepositoryPath $ExecutablePath
$fixture = Resolve-RepositoryPath $FixturePdf
$outputRoot = Resolve-RepositoryPath $OutputDirectory
$candidateRoot = Join-Path $outputRoot 'candidates'
New-Item -ItemType Directory -Force -Path $candidateRoot | Out-Null

if (-not $SkipLaunch) {
    if (-not (Test-Path -LiteralPath $executable)) {
        throw "Release executable not found: $executable"
    }
    if (-not (Test-Path -LiteralPath $fixture)) {
        throw "Fixture PDF not found: $fixture"
    }
    [void](Assert-YamantakaWindowsBundleRunnable `
            -BundlePath (Split-Path -Parent $executable) `
            -RepositoryRoot $repositoryRoot `
            -RequireSignature:$RequireSignature)
}

# Every case below uses a supported public entry point. Book mode is selected
# through the visible centered control; no private or ignored visual flags are
# passed to the application.
$cases = @(
    [pscustomobject]@{
        name = 'library-wide'
        width = 1600
        height = 900
        openFixture = $false
        selectBook = $false
        fullscreen = $false
    },
    [pscustomobject]@{
        name = 'reader-document'
        width = 1600
        height = 900
        openFixture = $true
        selectBook = $false
        fullscreen = $false
    },
    [pscustomobject]@{
        name = 'reader-book'
        width = 1600
        height = 900
        openFixture = $true
        selectBook = $true
        fullscreen = $false
    },
    [pscustomobject]@{
        name = 'reader-book-fullscreen'
        width = 1600
        height = 900
        openFixture = $true
        selectBook = $true
        fullscreen = $true
    }
)

$evidence = [System.Collections.Generic.List[object]]::new()
foreach ($case in $cases) {
    $outputPath = Join-Path $candidateRoot "$($case.name).png"
    if (Test-Path -LiteralPath $outputPath) {
        Remove-Item -LiteralPath $outputPath -Force
    }
    $process = $null
    try {
        if (-not $SkipLaunch) {
            $launch = @{
                FilePath = $executable
                PassThru = $true
                WorkingDirectory = (Split-Path -Parent $executable)
            }
            if ($case.openFixture) {
                $launch.ArgumentList = '"' + $fixture + '"'
            }
            $process = Start-Process @launch
            $capture = @{
                TargetProcessId = $process.Id
                OutputPath = $outputPath
                Width = $case.width
                Height = $case.height
                X = 80
                Y = 80
            }
            if ($case.openFixture) {
                # Select the requested public mode after pointer movement has
                # revealed the centered reader toolbar.
                $capture.ClickX = if ($case.selectBook) {
                    [int]($case.width / 2 + 75)
                }
                else {
                    [int]($case.width / 2 - 50)
                }
                $capture.ClickY = 78
            }
            if ($case.fullscreen) {
                $capture.SendKeys = '{F11}'
            }
            & (Join-Path $PSScriptRoot 'capture_windows_window.ps1') @capture
            if (-not (Test-Path -LiteralPath $outputPath)) {
                throw "Capture did not produce its candidate: $outputPath"
            }
        }
        elseif (-not (Test-Path -LiteralPath $outputPath)) {
            throw "-SkipLaunch was specified but the candidate is missing: $outputPath"
        }

        Add-Type -AssemblyName System.Drawing
        $bitmap = [System.Drawing.Bitmap]::new($outputPath)
        try {
            $actualWidth = $bitmap.Width
            $actualHeight = $bitmap.Height
        }
        finally {
            $bitmap.Dispose()
        }
        $evidence.Add([ordered]@{
            name = $case.name
            requestedWindowWidth = $case.width
            requestedWindowHeight = $case.height
            actualCaptureWidth = $actualWidth
            actualCaptureHeight = $actualHeight
            hostScalePercent = $HostScalePercent
            candidate = $outputPath
            mode = if ($case.selectBook) { 'book' } elseif ($case.openFixture) { 'document' } else { 'library' }
            fullscreen = [bool]$case.fullscreen
            nativeCapture = -not $SkipLaunch
            scaleConfirmed = [bool]$ScaleConfirmed
            capturedAt = [DateTime]::UtcNow.ToString('o')
        })
    }
    finally {
        if ($process -and -not $process.HasExited) {
            [void]$process.CloseMainWindow()
            if (-not $process.WaitForExit(5000)) {
                Stop-Process -Id $process.Id -Force
                [void]$process.WaitForExit(5000)
            }
        }
    }
}

$report = [ordered]@{
    format = 'yamantaka-visual-matrix'
    version = 2
    generatedAt = [DateTime]::UtcNow.ToString('o')
    executable = $executable
    hostScalePercent = $HostScalePercent
    scaleConfirmed = [bool]$ScaleConfirmed
    requireSignature = [bool]$RequireSignature
    cases = @($evidence)
    complete = (-not $SkipLaunch -and $ScaleConfirmed)
    note = 'Run the matrix once for each real Windows scale. The script records one host scale and never fabricates unobserved DPI variants.'
}
$reportPath = Join-Path $outputRoot 'matrix-report.json'
$report | ConvertTo-Json -Depth 10 | Set-Content -Encoding UTF8 $reportPath
Write-Host "Visual matrix evidence: $reportPath"
if (-not $ScaleConfirmed) {
    Write-Warning 'The host DPI was not confirmed. Capture evidence is intentionally incomplete.'
    exit 2
}
exit 0
