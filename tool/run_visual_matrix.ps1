[CmdletBinding()]
param(
    [string]$ExecutablePath = 'build/windows/x64/runner/Release/yamantaka.exe',
    [string]$OutputDirectory = 'build/verification/visual',
    [string]$FixturePdf = 'build/verification/corpus/normal.pdf',
    [switch]$SkipLaunch,
    [switch]$ScaleConfirmed
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$executable = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $ExecutablePath))
$outputRoot = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $OutputDirectory))
$candidateRoot = Join-Path $outputRoot 'candidates'
New-Item -ItemType Directory -Force -Path $candidateRoot | Out-Null

$layouts = @(
    [pscustomobject]@{ name = 'library-wide-light'; width = 1600; height = 900; args = @() },
    [pscustomobject]@{ name = 'library-compact-dark'; width = 1366; height = 768; args = @('--yamantaka-visual-dark') },
    [pscustomobject]@{ name = 'reader-fullhd'; width = 1920; height = 1080; args = @($FixturePdf) },
    [pscustomobject]@{ name = 'reader-book-2k'; width = 2560; height = 1440; args = @($FixturePdf, '--yamantaka-visual-book') },
    [pscustomobject]@{ name = 'settings-16x10'; width = 1920; height = 1200; args = @('--yamantaka-visual-settings') },
    [pscustomobject]@{ name = 'reader-4k'; width = 3840; height = 2160; args = @($FixturePdf) }
)
$matrix = foreach ($layout in $layouts) {
    foreach ($scale in @(100, 125, 150, 175, 200)) {
        [pscustomobject]@{
            name = "$($layout.name)-$scale"
            width = $layout.width
            height = $layout.height
            scale = $scale
            args = $layout.args
        }
    }
}

if (-not $SkipLaunch -and -not (Test-Path $executable)) {
    throw "Release executable not found: $executable"
}

$evidence = [System.Collections.Generic.List[object]]::new()
foreach ($case in $matrix) {
    $outputPath = Join-Path $candidateRoot "$($case.name).png"
    $process = $null
    try {
        if (-not $SkipLaunch) {
            $process = Start-Process -FilePath $executable -ArgumentList $case.args -PassThru -WorkingDirectory (Split-Path $executable -Parent)
            & (Join-Path $PSScriptRoot 'capture_windows_window.ps1') `
                -TargetProcessId $process.Id `
                -OutputPath $outputPath `
                -Width $case.width `
                -Height $case.height `
                -X 80 `
                -Y 80
            if ($LASTEXITCODE -ne 0) {
                throw "Capture failed with exit code $LASTEXITCODE."
            }
        }
        elseif (-not (Test-Path $outputPath)) {
            throw "-SkipLaunch was specified but the candidate is missing: $outputPath"
        }
        $evidence.Add([ordered]@{
            name = $case.name
            requestedPhysicalWidth = $case.width
            requestedPhysicalHeight = $case.height
            requestedScalePercent = $case.scale
            candidate = $outputPath
            dpr = $case.scale / 100
            theme = if ($case.name -like '*dark*') { 'dark' } else { 'light' }
            density = if ($case.width -le 1366) { 'compact' } else { 'comfortable' }
            nativeCapture = -not $SkipLaunch
            scaleConfirmed = [bool]$ScaleConfirmed
            validation = if ($ScaleConfirmed) { 'captured-and-scale-confirmed' } else { 'capture-only-scale-unverified' }
            capturedAt = [DateTime]::UtcNow.ToString('o')
        })
    }
    finally {
        if ($process -and -not $process.HasExited) {
            Stop-Process -Id $process.Id -Force
            $process.WaitForExit(5000)
        }
    }
}

$report = [ordered]@{
    format = 'yamantaka-visual-matrix'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    executable = $executable
    cases = @($evidence)
    physicalSizesAreRequested = $true
    scales = @(100, 125, 150, 175, 200)
    scaleConfirmationRequired = $true
    scaleConfirmed = [bool]$ScaleConfirmed
    complete = (-not $SkipLaunch -and $ScaleConfirmed)
    note = 'Windows scaling must be applied by the test host. Pass -ScaleConfirmed only after the host reports the requested DPI; capture dimensions alone do not prove Windows scaling.'
}
$reportPath = Join-Path $outputRoot 'matrix-report.json'
$report | ConvertTo-Json -Depth 10 | Set-Content -Encoding UTF8 $reportPath
Write-Host "Visual matrix evidence: $reportPath"
if (-not $ScaleConfirmed) {
    Write-Warning 'Matrix captured, but Windows DPI was not confirmed by the host. The evidence is intentionally not a passing gate.'
    exit 2
}
