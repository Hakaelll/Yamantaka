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
    @{ name='smart-annotated'; args=@('--pdf-covers','--smart-view=annotated') },
    @{ name='smart-recent-es'; args=@('--pdf-covers','--smart-view=recentlyOpened','--es','--light','--reduce-motion') },
    @{ name='smart-menu-small-200'; args=@('--pdf-covers','--smart-view=annotated','--smart-menu','--size=640x480','--text-scale=2') },
    @{ name='collections-manager'; args=@('--pdf-covers','--collections') },
    @{ name='collections-filter-es'; args=@('--pdf-covers','--collections','--collection-filter','--es','--light','--reduce-motion') },
    @{ name='collections-delete-review'; args=@('--pdf-covers','--collections','--collection-delete-review') },
    @{ name='collections-small-200'; args=@('--pdf-covers','--collections','--size=640x480','--text-scale=2') },
    @{ name='cover-cache-dark'; args=@('--settings','--cache-maintenance') },
    @{ name='cover-cache-cleared-es'; args=@('--settings','--cache-maintenance','--clear-cover-cache','--light','--es','--reduce-motion') },
    @{ name='cover-cache-small-200'; args=@('--settings','--cache-maintenance','--size=640x480','--text-scale=2') },
    @{ name='import-reference-dark'; args=@('--pdf-covers','--folder-import','--import-reference') },
    @{ name='import-reference-light-es'; args=@('--pdf-covers','--folder-import','--import-reference','--es','--light','--reduce-motion') },
    @{ name='import-reference-small-200'; args=@('--pdf-covers','--folder-import','--import-reference','--size=640x480','--text-scale=2') },
    @{ name='library-check-summary'; args=@('--pdf-covers','--library-check') },
    @{ name='library-check-modified-es'; args=@('--pdf-covers','--library-check','--library-check-modified','--es','--light','--reduce-motion') },
    @{ name='library-check-cancelled'; args=@('--pdf-covers','--library-check','--library-check-cancel') },
    @{ name='library-check-small-200'; args=@('--pdf-covers','--library-check','--size=640x480','--text-scale=2') },
    @{ name='relocation-review'; args=@('--pdf-covers','--relocate') },
    @{ name='relocation-saved-es'; args=@('--pdf-covers','--relocate','--relocation-confirm','--es','--light','--reduce-motion') },
    @{ name='relocation-mismatch'; args=@('--pdf-covers','--relocate','--relocation-mismatch') },
    @{ name='relocation-legacy-small-200'; args=@('--pdf-covers','--relocate','--relocation-legacy','--size=640x480','--text-scale=2') },
    @{ name='file-check-baseline'; args=@('--pdf-covers','--file-check') },
    @{ name='file-check-missing-es'; args=@('--pdf-covers','--file-check','--file-missing','--es','--light','--reduce-motion') },
    @{ name='file-check-changed'; args=@('--pdf-covers','--file-check','--file-changed') },
    @{ name='file-check-restored-small-200'; args=@('--pdf-covers','--file-check','--file-restored','--size=640x480','--text-scale=2') },
    @{ name='import-recovery-pending'; args=@('--pdf-covers','--import-recovery') },
    @{ name='import-recovery-resumed-es'; args=@('--pdf-covers','--import-recovery','--resume-import','--es','--light','--reduce-motion') },
    @{ name='import-recovery-discarded'; args=@('--pdf-covers','--import-recovery','--discard-import') },
    @{ name='import-recovery-small-200'; args=@('--pdf-covers','--import-recovery','--size=640x480','--text-scale=2') },
    @{ name='folder-import-dark'; args=@('--pdf-covers','--folder-import') },
    @{ name='folder-import-tools-es'; args=@('--pdf-covers','--folder-import','--tools-menu','--light','--es','--reduce-motion') },
    @{ name='folder-import-small-200'; args=@('--pdf-covers','--folder-import','--tools-menu','--size=640x480','--text-scale=2') },
    @{ name='compare-pdf-identical'; args=@('--pdf-covers','--compare-pdf') },
    @{ name='compare-pdf-different-es'; args=@('--pdf-covers','--compare-different','--light','--es','--reduce-motion') },
    @{ name='cover-capacity-retry-dark'; args=@('--pdf-covers','--cover-capacity-retry') },
    @{ name='cover-capacity-retry-light-es'; args=@('--pdf-covers','--cover-capacity-retry','--light','--es','--reduce-motion') },
    @{ name='metadata-reanalysis-dark'; args=@('--metadata-review','--metadata-reanalyze') },
    @{ name='metadata-reanalysis-light-es'; args=@('--metadata-review','--metadata-reanalyze','--light','--es','--reduce-motion') },
    @{ name='metadata-reanalysis-manual'; args=@('--metadata-review','--metadata-reanalyze','--metadata-reanalyze-manual') },
    @{ name='metadata-reanalysis-limit'; args=@('--metadata-review','--metadata-reanalyze','--metadata-reanalyze-limit') },
    @{ name='metadata-reanalysis-small-200'; args=@('--metadata-review','--metadata-reanalyze','--size=640x480','--text-scale=2') },
    @{ name='metadata-source-dark'; args=@('--metadata-review','--metadata-reanalyze','--metadata-evidence-later') },
    @{ name='metadata-source-light-es'; args=@('--metadata-review','--metadata-reanalyze','--metadata-evidence-later','--light','--es','--reduce-motion') },
    @{ name='metadata-source-small-200'; args=@('--metadata-review','--metadata-reanalyze','--metadata-evidence-later','--size=640x480','--text-scale=2') },
    @{ name='metadata-source-small-reference-200'; args=@('--metadata-review','--metadata-reanalyze','--metadata-evidence-later','--metadata-reference-visible','--size=640x480','--text-scale=2','--reduce-motion') },
    @{ name='metadata-source-open'; args=@('--metadata-review','--metadata-reanalyze','--metadata-evidence-later','--metadata-source-open') },
    @{ name='metadata-source-return'; args=@('--metadata-review','--metadata-reanalyze','--metadata-evidence-later','--metadata-source-return') },
    @{ name='metadata-source-draft'; args=@('--metadata-review','--metadata-reanalyze','--metadata-evidence-later','--metadata-source-draft') },
    @{ name='metadata-source-stale'; args=@('--metadata-review','--metadata-reanalyze','--metadata-evidence-later','--metadata-source-stale') },
    @{ name='metadata-review-dark'; args=@('--metadata-review') },
    @{ name='metadata-review-light-es'; args=@('--metadata-review','--light','--es','--reduce-motion') },
    @{ name='metadata-review-small-200'; args=@('--metadata-review','--size=640x480','--text-scale=2') },
    @{ name='metadata-review-saved'; args=@('--metadata-review','--metadata-save') },
    @{ name='metadata-review-dismissed'; args=@('--metadata-review','--metadata-dismiss') },
    @{ name='metadata-review-reset'; args=@('--metadata-review','--metadata-reset') },
    @{ name='metadata-review-failed'; args=@('--metadata-review','--metadata-failed') },
    @{ name='cover-refresh-version'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-stale-version','--details') },
    @{ name='cover-refresh-source'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-stale-source','--details','--light','--es','--reduce-motion') },
    @{ name='cover-refresh-manual'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-manual','--details') },
    @{ name='cover-restore-controls'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-manual','--cover-picker') },
    @{ name='cover-restored-dark'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-manual','--cover-picker-restore','--details') },
    @{ name='cover-restored-light-es'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-manual','--cover-picker-restore','--details','--light','--es','--reduce-motion') },
    @{ name='cover-restore-failed'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-manual','--cover-picker-restore-failed') },
    @{ name='cover-restore-small-200'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-manual','--cover-picker','--size=640x480','--text-scale=2') },
    @{ name='automatic-cover-dark'; args=@('--pdf-covers','--automatic-cover','--details') },
    @{ name='automatic-cover-light-es'; args=@('--pdf-covers','--automatic-cover','--details','--light','--es','--reduce-motion') },
    @{ name='automatic-cover-manual'; args=@('--pdf-covers','--automatic-cover','--automatic-cover-manual','--details') },
    @{ name='cover-picker-dark'; args=@('--pdf-covers','--cover-picker') },
    @{ name='cover-picker-light-es'; args=@('--pdf-covers','--cover-picker','--light','--es','--reduce-motion') },
    @{ name='cover-picker-saved'; args=@('--pdf-covers','--cover-picker-save','--details') },
    @{ name='cover-picker-cancelled'; args=@('--pdf-covers','--cover-picker-cancel','--details') },
    @{ name='cover-picker-failed-save'; args=@('--pdf-covers','--cover-picker-failed-save') },
    @{ name='cover-picker-small-200'; args=@('--pdf-covers','--cover-picker','--size=640x480','--text-scale=2') },
    @{ name='pdf-saved-cover-page'; args=@('--pdf-covers','--saved-cover-page','--details') },
    @{ name='pdf-saved-cover-page-es'; args=@('--pdf-covers','--saved-cover-page','--details','--light','--es','--reduce-motion') },
    @{ name='pdf-covers-dark'; args=@('--pdf-covers') },
    @{ name='pdf-covers-light-es'; args=@('--pdf-covers','--light','--es') },
    @{ name='pdf-covers-details'; args=@('--pdf-covers','--details') },
    @{ name='pdf-covers-tilted'; args=@('--pdf-covers','--tilt') },
    @{ name='pdf-covers-reduced'; args=@('--pdf-covers','--reduce-motion') },
    @{ name='pdf-covers-small-200'; args=@('--pdf-covers','--size=640x480','--text-scale=2') },
    @{ name='tabs-close-others'; args=@('--reader','--tabs-close-others') },
    @{ name='tabs-close-others-es'; args=@('--reader','--tabs-close-others','--light','--es','--reduced-motion') },
    @{ name='tabs-close-right'; args=@('--reader','--tabs-close-right') },
    @{ name='tabs-close-right-small-200'; args=@('--reader','--tabs-close-right','--size=640x480','--text-scale=2') },
    @{ name='tabs-inactive-closed'; args=@('--reader','--tabs-inactive-close') },
    @{ name='tabs-close-reopen'; args=@('--reader','--tabs-close-reopen') },
    @{ name='tabs-close-reopen-es'; args=@('--reader','--tabs-close-reopen','--light','--es','--reduced-motion') },
    @{ name='tabs-last-reopen'; args=@('--reader','--tabs-last-reopen') },
    @{ name='tabs-last-reopen-small-200'; args=@('--reader','--tabs-last-reopen','--size=640x480','--text-scale=2') },
    @{ name='tabs-last-closed'; args=@('--reader','--tabs-last-close','--sessions-preview') },
    @{ name='reader-tabs'; args=@('--reader','--tabs') },
    @{ name='reader-tabs-light-es'; args=@('--reader','--tabs','--light','--es','--reduced-motion') },
    @{ name='reader-tabs-small-200'; args=@('--reader','--tabs','--size=640x480','--text-scale=2') },
    @{ name='reader-preserved'; args=@('--reader','--open-replacement-failure') },
    @{ name='reader-preserved-light-es'; args=@('--reader','--open-replacement-failure','--light','--es','--reduced-motion') },
    @{ name='reader-preserved-small-200'; args=@('--reader','--open-replacement-failure','--size=640x480','--text-scale=2') },
    @{ name='goal-editor'; args=@('--details','--goal-editor','--sessions-preview') },
    @{ name='goal-editor-light-es'; args=@('--details','--goal-editor','--light','--es','--reduced-motion') },
    @{ name='goal-editor-small-200'; args=@('--details','--goal-editor','--size=640x480','--text-scale=2') },
    @{ name='goal-saved'; args=@('--details','--goal-save','--sessions-preview') },
    @{ name='goal-cleared'; args=@('--details','--goal-clear','--sessions-preview','--light','--es') },
    @{ name='history-deleted'; args=@('--history-delete','--sessions-preview') },
    @{ name='history-deleted-es'; args=@('--history-delete','--sessions-preview','--light','--es','--reduced-motion') },
    @{ name='history-restored'; args=@('--history-restore') },
    @{ name='history-restored-es'; args=@('--history-restore','--es','--light','--reduced-motion') },
    @{ name='history-invalid'; args=@('--history-invalid') },
    @{ name='history-dark'; args=@('--history') },
    @{ name='history-light-es'; args=@('--history','--light','--es','--reduced-motion') },
    @{ name='history-next'; args=@('--history-next') },
    @{ name='history-empty'; args=@('--history-empty') },
    @{ name='history-small-200'; args=@('--history','--size=640x480','--text-scale=2') },
    @{ name='global-notes-export-json'; args=@('--annotation-many','--global-notes-export-json') },
    @{ name='global-notes-export-markdown-es'; args=@('--annotation-many','--global-notes-export-markdown','--light','--es') },
    @{ name='global-notes-passage'; args=@('--annotation-many','--global-notes-passage') },
    @{ name='global-notes-passage-reduced-es'; args=@('--annotation-many','--global-notes-passage','--reduce-motion','--light','--es') },
    @{ name='global-notes-filters'; args=@('--annotation-many','--global-notes-filters') },
    @{ name='global-notes-open'; args=@('--annotation-many','--global-notes-open') },
    @{ name='global-notes-dark'; args=@('--annotation-many','--global-notes') },
    @{ name='global-notes-review-es'; args=@('--annotation-many','--global-notes-review','--light','--es') },
    @{ name='global-notes-tag'; args=@('--annotation-many','--global-notes-tag') },
    @{ name='global-notes-empty'; args=@('--annotation-many','--global-notes-empty') },
    @{ name='global-notes-small-200'; args=@('--annotation-many','--global-notes','--size=640x480','--text-scale=2') },
    @{ name='annotation-bulk-tag-dark'; args=@('--reader','--annotation-many','--annotation-bulk-tag') },
    @{ name='annotation-bulk-tag-saved-es'; args=@('--reader','--annotation-many','--annotation-bulk-tag-save','--light','--es') },
    @{ name='annotation-bulk-tag-export'; args=@('--reader','--annotation-many','--annotation-bulk-tag-save','--export-json') },
    @{ name='annotation-bulk-small-200'; args=@('--reader','--annotation-many','--annotation-bulk','--size=640x480','--text-scale=2') },
    @{ name='annotation-bulk-dark'; args=@('--reader','--annotation-many','--annotation-bulk') },
    @{ name='annotation-bulk-saved-es'; args=@('--reader','--annotation-many','--annotation-bulk-save','--light','--es') },
    @{ name='annotation-bulk-filtered'; args=@('--reader','--annotation-many','--annotation-filter','--annotation-bulk-save') },
    @{ name='annotation-filter-dark'; args=@('--reader','--annotation-many','--annotation-filter') },
    @{ name='annotation-filter-light-es'; args=@('--reader','--annotation-many','--annotation-filter','--light','--es') },
    @{ name='annotation-filter-empty'; args=@('--reader','--annotation-many','--annotation-filter-empty') },
    @{ name='annotation-filter-export'; args=@('--reader','--annotation-many','--annotation-filter','--export-json') },
    @{ name='annotation-filter-small-200'; args=@('--reader','--annotation-many','--annotation-filter','--size=640x480','--text-scale=2') },
    @{ name='annotation-export-json'; args=@('--reader','--annotation-many','--export-json') },
    @{ name='annotation-export-markdown-es'; args=@('--reader','--annotation-many','--export-markdown','--light','--es') },
    @{ name='annotation-export-small-200'; args=@('--reader','--annotation-many','--export-json','--size=640x480','--text-scale=2') },
    @{ name='annotation-many-rotated'; args=@('--reader','--annotation-many','--rotation=90') },
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
    @{ name='privacy-sessions-dark'; args=@('--settings','--privacy') },
    @{ name='privacy-sessions-light-es'; args=@('--settings','--privacy','--light','--es','--sessions-disabled','--reduce-motion') },
    @{ name='privacy-sessions-small-200'; args=@('--settings','--privacy','--size=640x480','--text-scale=2') },
    @{ name='reader-session-recorded'; args=@('--reader','--sessions-preview') },
    @{ name='reader-session-disabled'; args=@('--reader','--sessions-preview','--sessions-disabled') },
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
    $export = $null
    if ($controls.annotationExport.artifact) {
        $artifactName = [string]$controls.annotationExport.artifact
        if ([IO.Path]::GetFileName($artifactName) -ne $artifactName) { throw 'Export artifact must be a file name.' }
        $artifactPath = Join-Path $outputPath $artifactName
        if (-not $controls.annotationExport.saved -or $controls.annotationExport.pending -or -not (Test-Path -LiteralPath $artifactPath -PathType Leaf)) { throw 'Export capture must contain a successfully published file.' }
        $export = [ordered]@{ file=$artifactName; sha256=(Get-FileHash -LiteralPath $artifactPath -Algorithm SHA256).Hash; count=$controls.annotationExport.count }
    }
    $captures += [ordered]@{
        name=$case.name; args=@('--demo') + $case.args
        image="$($case.name).png"; imageSha256=(Get-FileHash -LiteralPath $imagePath -Algorithm SHA256).Hash
        controls="$($case.name).controls.json"; controlsSha256=(Get-FileHash -LiteralPath $controlPath -Algorithm SHA256).Hash
        viewport=$controls.viewport; pixelsPerPoint=$controls.pixelsPerPoint
        exitCode=$process.ExitCode; annotationExport=$export
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
