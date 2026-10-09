param(
    [string]$Executable = "$PSScriptRoot/../target/debug/yamantaka-rust.exe",
    [string]$OutputDirectory = "$PSScriptRoot/../build/rust-visual/stage2",
    [string[]]$CaseNames = @()
)
$ErrorActionPreference = 'Stop'
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
$repositoryPath = (Resolve-Path -LiteralPath "$PSScriptRoot/..").Path
$null = New-Item -ItemType Directory -Path $OutputDirectory -Force
$outputPath = (Resolve-Path -LiteralPath $OutputDirectory).Path
$cases = @(
    @{ name='search-dark'; args=@('--search') },
    @{ name='search-open-page'; args=@('--search','--search-open-page') },
    @{ name='search-excluded'; args=@('--search','--search-excluded') },
    @{ name='search-light'; args=@('--search','--light') },
    @{ name='search-disabled-es'; args=@('--search-disabled','--es') },
    @{ name='search-small-200'; args=@('--search','--size=640x480','--text-scale=2') },
    @{ name='library-dark'; args=@() },
    @{ name='library-light'; args=@('--light') },
    @{ name='library-list'; args=@('--list') },
    @{ name='library-list-states'; args=@('--list-states') },
    @{ name='library-details'; args=@('--details') },
    @{ name='library-es'; args=@('--es') },
    @{ name='library-small-200'; args=@('--size=640x480','--text-scale=2') },
    @{ name='library-compact'; args=@('--size=640x480') },
    @{ name='library-tilted'; args=@('--tilt') },
    @{ name='library-reduced'; args=@('--reduce-motion') },
    @{ name='library-opengl'; args=@('--opengl') },
    @{ name='settings-light'; args=@('--settings','--light') },
    @{ name='settings-small-200'; args=@('--settings','--size=640x480','--text-scale=2') },
    @{ name='reader-dark'; args=@('--reader') },
    @{ name='reader-text'; args=@('--reader','--text-panel') },
    @{ name='reader-opengl'; args=@('--reader','--opengl') },
    @{ name='reader-facing'; args=@('--reader','--facing') },
    @{ name='reader-continuous'; args=@('--reader','--continuous') },
    @{ name='reader-layout-options'; args=@('--reader','--reader-options','--layout-options') },
    @{ name='reader-options'; args=@('--reader','--reader-options') },
    @{ name='reader-sepia-rotated'; args=@('--reader','--reader-theme=sepia','--rotation=90') },
    @{ name='reader-fit-width'; args=@('--reader','--fit-width') },
    @{ name='reader-options-small-200'; args=@('--reader','--reader-options','--size=640x480','--text-scale=2') },
    @{ name='reader-small-200'; args=@('--reader','--size=640x480','--text-scale=2') }
)
if ($CaseNames.Count -gt 0) {
    foreach ($requestedCase in $CaseNames) {
        if ($requestedCase -notin $cases.name) { throw "Unknown capture case: $requestedCase" }
    }
    $cases = @($cases | Where-Object { $_.name -in $CaseNames })
}
$captures = @()
foreach ($case in $cases) {
    $imagePath = Join-Path $outputPath "$($case.name).png"
    $controlPath = Join-Path $outputPath "$($case.name).controls.json"
    # Delete only the two exact outputs this run owns, so stale evidence cannot pass.
    foreach ($ownedPath in @($imagePath, $controlPath)) {
        if (Test-Path -LiteralPath $ownedPath) { Remove-Item -LiteralPath $ownedPath }
    }
    $arguments = @('--demo', "`"--capture=$imagePath`"") + $case.args
    $process = Start-Process -FilePath $executablePath -ArgumentList $arguments -WorkingDirectory $repositoryPath -WindowStyle Hidden -PassThru
    if (-not $process.WaitForExit(45000)) {
        $process.Kill()
        throw "Capture timed out: $($case.name)"
    }
    if ($process.ExitCode -ne 0) { throw "Capture failed ($($process.ExitCode)): $($case.name)" }
    if (-not (Test-Path -LiteralPath $imagePath) -or -not (Test-Path -LiteralPath $controlPath)) {
        throw "Native capture did not produce PNG and control bounds: $($case.name)"
    }
    $controls = Get-Content -LiteralPath $controlPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if (-not $controls.synthetic) { throw 'Only the isolated synthetic collection may be captured by this script.' }
    $captures += [ordered]@{
        name=$case.name; args=@('--demo') + $case.args
        image="$($case.name).png"; imageSha256=(Get-FileHash -LiteralPath $imagePath -Algorithm SHA256).Hash
        controls="$($case.name).controls.json"; controlsSha256=(Get-FileHash -LiteralPath $controlPath -Algorithm SHA256).Hash
        viewport=$controls.viewport; pixelsPerPoint=$controls.pixelsPerPoint
        exitCode=$process.ExitCode
    }
    Write-Output "Captured $($case.name)"
}
$sources = @(Get-ChildItem -LiteralPath "$repositoryPath/rust/ui/src" -File -Recurse | Sort-Object FullName | ForEach-Object {
    [ordered]@{ path=$_.FullName.Substring($repositoryPath.Length + 1).Replace('\','/'); sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash }
})
$manifest = [ordered]@{
    version=1; capturedAtUtc=[DateTime]::UtcNow.ToString('o'); synthetic=$true
    executableSha256=(Get-FileHash -LiteralPath $executablePath -Algorithm SHA256).Hash
    cargoLockSha256=(Get-FileHash -LiteralPath "$repositoryPath/Cargo.lock" -Algorithm SHA256).Hash
    sources=$sources; captures=$captures
    scope='Native synthetic screenshots and custom-control bounds. Text zoom is not physical monitor DPI certification; no Narrator or multi-monitor acceptance is implied.'
}
$manifest | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $outputPath 'capture-manifest.json') -Encoding UTF8
