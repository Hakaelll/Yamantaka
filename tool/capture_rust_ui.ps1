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
    @{ name='page-note-scanned-editor'; args=@('--reader','--page-note','--find-scanned') },
    @{ name='page-note-scanned-saved-es'; args=@('--reader','--page-note','--find-scanned','--annotation-save','--light','--es') },
    @{ name='annotation-multi-editor'; args=@('--reader','--annotations','--annotation-multi') },
    @{ name='annotation-multi-saved-es'; args=@('--reader','--annotations','--annotation-multi','--annotation-save','--light','--es') },
    @{ name='annotation-editor-dark'; args=@('--reader','--annotations') },
    @{ name='annotation-editor-light-es'; args=@('--reader','--annotations','--light','--es') },
    @{ name='annotation-editor-small-200'; args=@('--reader','--annotations','--size=640x480','--text-scale=2') },
    @{ name='annotation-saved-rotated'; args=@('--reader','--annotations','--annotation-save','--rotation=90') },
    @{ name='selection-keyboard-dark'; args=@('--reader','--selection','--selection-keyboard') },
    @{ name='selection-keyboard-rotated'; args=@('--reader','--selection','--selection-keyboard','--rotation=90') },
    @{ name='selection-keyboard-small-200'; args=@('--reader','--selection','--selection-keyboard','--size=640x480','--text-scale=2') },
    @{ name='selection-dark'; args=@('--reader','--selection') },
    @{ name='selection-light-es'; args=@('--reader','--selection','--light','--es') },
    @{ name='selection-rotated'; args=@('--reader','--selection','--rotation=90') },
    @{ name='selection-small-200'; args=@('--reader','--selection','--size=640x480','--text-scale=2') },
    @{ name='selection-empty-es'; args=@('--reader','--selection','--find-scanned','--light','--es') },
    @{ name='external-link-dark'; args=@('--links','--external-links','--links-activate') },
    @{ name='external-link-light-es'; args=@('--links','--external-links','--links-activate','--light','--es') },
    @{ name='external-link-small-200'; args=@('--links','--external-links','--links-activate','--size=640x480','--text-scale=2') },
    @{ name='external-link-rejected-es'; args=@('--links','--rejected-link','--links-activate','--es') },
    @{ name='links-dark'; args=@('--links') },
    @{ name='links-light-es'; args=@('--links','--light','--es') },
    @{ name='links-small-200'; args=@('--links','--size=640x480','--text-scale=2') },
    @{ name='links-keyboard'; args=@('--links','--links-activate') },
    @{ name='links-rotated'; args=@('--links','--rotation=90') },
    @{ name='find-no-matches'; args=@('--find','--find-absent') },
    @{ name='find-no-text-es'; args=@('--find','--find-scanned','--light','--es') },
    @{ name='find-dark'; args=@('--find') },
    @{ name='find-light-es'; args=@('--find','--light','--es') },
    @{ name='find-small-200'; args=@('--find','--size=640x480','--text-scale=2') },
    @{ name='find-closed-wrap'; args=@('--find','--find-previous','--find-closed') },
    @{ name='find-next'; args=@('--find','--find-next') },
    @{ name='find-previous-wrap'; args=@('--find','--find-previous') },
    @{ name='find-rotated'; args=@('--find','--rotation=90') },
    @{ name='thumbnails-dark'; args=@('--thumbnails') },
    @{ name='thumbnails-jump'; args=@('--thumbnails','--thumbnail-page=1') },
    @{ name='thumbnails-rotated'; args=@('--thumbnails','--rotation=90') },
    @{ name='thumbnails-5000-last'; args=@('--thumbnails','--thumbnails-long','--thumbnail-page=4999') },
    @{ name='thumbnails-light-es'; args=@('--thumbnails','--light','--es') },
    @{ name='thumbnails-small-200'; args=@('--thumbnails','--size=640x480','--text-scale=2') },
    @{ name='outline-dark'; args=@('--outline') },
    @{ name='outline-passage'; args=@('--outline','--outline-jump') },
    @{ name='outline-light-es'; args=@('--outline','--light','--es') },
    @{ name='outline-small-200'; args=@('--outline','--size=640x480','--text-scale=2') },
    @{ name='bookmarks-dark'; args=@('--bookmarks') },
    @{ name='bookmarks-light-es'; args=@('--bookmarks','--light','--es') },
    @{ name='bookmarks-small-200'; args=@('--bookmarks','--size=640x480','--text-scale=2') },
    @{ name='password-dark'; args=@('--protected') },
    @{ name='password-light-es'; args=@('--protected','--light','--es') },
    @{ name='password-small-200'; args=@('--protected','--size=640x480','--text-scale=2') },
    @{ name='password-rejected'; args=@('--protected','--password-wrong') },
    @{ name='password-unlocked'; args=@('--protected','--password-unlock') },
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
    @{ name='reader-text-light-es'; args=@('--reader','--text-panel','--light','--es') },
    @{ name='reader-text-small-200'; args=@('--reader','--text-panel','--size=640x480','--text-scale=2') },
    @{ name='reader-text-empty-es'; args=@('--reader','--text-panel','--find-scanned','--light','--es') },
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
