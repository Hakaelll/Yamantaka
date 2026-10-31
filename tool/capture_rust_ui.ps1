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
    @{ name='rule-simulation-extension'; args=@('--saved-rules','--rule-simulation','--es') },
    @{ name='rule-simulation-duplicates'; args=@('--saved-rules','--rule-simulation','--rule-simulation-duplicates','--light') },
    @{ name='rule-simulation-cancel'; args=@('--saved-rules','--rule-simulation','--rule-simulation-cancel','--es') },
    @{ name='rule-simulation-small-200'; args=@('--saved-rules','--rule-simulation','--es','--light','--reduce-motion','--size=640x480','--text-scale=2') },
    @{ name='saved-rules-legacy'; args=@('--saved-rules','--es') },
    @{ name='saved-rules-changed'; args=@('--saved-rules','--saved-rules-change','--light') },
    @{ name='saved-rules-review-small-200'; args=@('--saved-rules','--saved-rules-review','--es','--light','--reduce-motion','--size=640x480','--text-scale=2') },
    @{ name='saved-rules-paged'; args=@('--saved-rules','--saved-rules-many') },
    @{ name='revision-groups-legacy'; args=@('--revision-groups','--es') },
    @{ name='revision-groups-created'; args=@('--revision-groups','--revision-create','--light') },
    @{ name='revision-groups-current-small-200'; args=@('--revision-groups','--revision-current','--es','--light','--reduce-motion','--size=640x480','--text-scale=2') },
    @{ name='relations-backlinks'; args=@('--relations') },
    @{ name='relations-review-es'; args=@('--relations','--relations-review','--es','--light') },
    @{ name='relations-review-small-200'; args=@('--relations','--relations-review','--es','--light','--reduce-motion','--size=640x480','--text-scale=2') },
    @{ name='reader-reference-note-export-json'; args=@('--reader','--reference','--reference-book','--reference-notes','--reference-note-bulk','--reference-note-bulk-save','--reference-export-json','--es') },
    @{ name='reader-reference-note-export-markdown'; args=@('--reader','--reference','--reference-book','--reference-notes','--reference-note-bulk','--reference-note-bulk-save','--reference-export-md','--light') },
    @{ name='reader-reference-note-bulk'; args=@('--reader','--reference','--reference-book','--reference-notes','--reference-note-bulk','--es') },
    @{ name='reader-reference-note-bulk-saved'; args=@('--reader','--reference','--reference-book','--reference-notes','--reference-note-bulk','--reference-note-bulk-save','--light') },
    @{ name='reader-reference-note-filters'; args=@('--reader','--reference','--reference-book','--reference-notes','--reference-note-filters','--light','--es') },
    @{ name='reader-reference-note-filters-empty'; args=@('--reader','--reference','--reference-book','--reference-notes','--reference-note-filters','--reference-note-filters-empty') },
    @{ name='reader-reference-notes'; args=@('--reader','--reference','--reference-book','--reference-notes','--es') },
    @{ name='reader-reference-notes-shared-close'; args=@('--reader','--reference','--reference-notes','--es') },
    @{ name='reader-reference-notes-editor-small-200'; args=@('--reader','--reference','--reference-book','--reference-notes','--reference-notes-editor','--es','--size=640x480','--text-scale=2','--light','--reduce-motion') },
    @{ name='reader-reference-marks'; args=@('--reader','--reference','--reference-book','--reference-marks','--es') },
    @{ name='reader-reference-marks-shared'; args=@('--reader','--reference','--reference-marks','--es') },
    @{ name='reader-reference-marks-small-200'; args=@('--reader','--reference','--reference-marks','--rotation=90','--es','--size=640x480','--text-scale=2','--light','--reduce-motion') },
    @{ name='reader-reference-selection'; args=@('--reader','--reference','--reference-book','--reference-selection','--reference-swap','--es') },
    @{ name='reader-reference-selection-shared'; args=@('--reader','--reference','--reference-selection','--reference-selection-shared','--es') },
    @{ name='reader-reference-selection-small-200'; args=@('--reader','--reference','--reference-book','--reference-selection','--es','--size=640x480','--text-scale=2','--light','--reduce-motion') },
    @{ name='reader-reference-selection-close'; args=@('--reader','--reference','--reference-book','--reference-selection','--reference-selection-close','--es') },
    @{ name='reader-reference-history'; args=@('--reader','--reference','--reference-book','--reference-history','--reference-swap','--es') },
    @{ name='reader-reference-history-small-200'; args=@('--reader','--reference','--reference-book','--reference-history','--es','--size=640x480','--text-scale=2','--light','--reduce-motion') },
    @{ name='reader-primary-history'; args=@('--reader','--reference','--reference-book','--reference-history','--history-primary') },
    @{ name='reader-reference-shortcuts'; args=@('--reader','--reference','--reference-book','--reference-shortcuts','--es') },
    @{ name='reader-reference-shortcuts-small-200'; args=@('--reader','--reference','--reference-book','--reference-shortcuts','--es','--size=640x480','--text-scale=2','--light','--reduce-motion') },
    @{ name='reader-reference-shortcuts-close'; args=@('--reader','--reference','--reference-book','--reference-shortcuts','--reference-swap','--reference-shortcuts-close','--es') },
    @{ name='reader-reference-text'; args=@('--reader','--reference','--reference-book','--reference-text','--text-panel','--es') },
    @{ name='reader-reference-text-small-200'; args=@('--reader','--reference','--reference-book','--reference-text','--es','--size=640x480','--text-scale=2','--light','--reduce-motion') },
    @{ name='reader-reference-text-empty'; args=@('--reader','--reference','--reference-book','--reference-text','--reference-text-empty','--es','--light') },
    @{ name='reader-reference-bookmarks'; args=@('--reader','--reference','--reference-book','--reference-bookmarks','--es') },
    @{ name='reader-reference-bookmarks-small-200'; args=@('--reader','--reference','--reference-book','--reference-bookmarks','--es','--size=640x480','--text-scale=2','--light','--reduce-motion') },
    @{ name='reader-reference-bookmarks-shared'; args=@('--reader','--reference','--reference-bookmarks','--es') },
    @{ name='reader-reference-thumbnails'; args=@('--reader','--reference','--reference-book','--reference-thumbnails','--es') },
    @{ name='reader-reference-thumbnails-small-200'; args=@('--reader','--reference','--reference-book','--reference-thumbnails','--es','--size=640x480','--text-scale=2','--light','--reduce-motion') },
    @{ name='reader-reference-outline'; args=@('--reader','--reference','--reference-book','--reference-outline','--es') },
    @{ name='reader-reference-links'; args=@('--reader','--reference','--reference-book','--reference-links','--es') },
    @{ name='reader-reference-external'; args=@('--reader','--reference','--reference-book','--reference-links','--reference-link-external','--es') },
    @{ name='reader-reference-sync'; args=@('--reader','--reference','--reference-sync','--es') },
    @{ name='reader-reference-chooser'; args=@('--reader','--reference','--reference-chooser','--es') },
    @{ name='reader-reference-password-import'; args=@('--reader','--reference','--reference-book','--reference-protected','--reference-password-import','--es') },
    @{ name='reader-reference-password-retry'; args=@('--reader','--reference','--reference-book','--reference-protected','--es') },
    @{ name='reader-reference-password-cancel'; args=@('--reader','--reference','--reference-book','--reference-protected','--reference-password-cancel','--es') },
    @{ name='reader-reference-book'; args=@('--reader','--reference','--reference-book','--reference-swap','--reference-find','--es') },
    @{ name='reader-reference-closed'; args=@('--reader','--reference','--reference-book','--reference-close','--es') },
    @{ name='reader-reference'; args=@('--reader','--reference','--es') },
    @{ name='reader-reference-stacked'; args=@('--reader','--reference','--size=900x700','--light','--es') },
    @{ name='workspaces'; args=@('--workspaces','--es') },
    @{ name='reading-queue'; args=@('--reading-queue','--es') },
    @{ name='restore-review'; args=@('--restore-preview','--es') },
    @{ name='backup-saved'; args=@('--backup-export=save') },
    @{ name='backup-cancelled-es'; args=@('--backup-export=cancel','--es','--light','--reduce-motion') },
    @{ name='backup-blocked'; args=@('--backup-export=blocked') },
    @{ name='backup-saved-small-200'; args=@('--backup-export=save','--size=640x480','--text-scale=2','--es','--light','--reduce-motion') },
    @{ name='migration-files'; args=@('--legacy-import=files') },
    @{ name='migration-review'; args=@('--legacy-import=review') },
    @{ name='migration-copy-es'; args=@('--legacy-import=copy','--es','--light','--reduce-motion') },
    @{ name='migration-selected'; args=@('--legacy-import=activate') },
    @{ name='migration-original'; args=@('--legacy-import=original') },
    @{ name='migration-stale'; args=@('--legacy-import=stale') },
    @{ name='migration-cancelled'; args=@('--legacy-import=cancel') },
    @{ name='migration-blocked-small-200'; args=@('--legacy-import=blocked','--size=640x480','--text-scale=2','--es','--light','--reduce-motion') },
    @{ name='grouping-author'; args=@('--pdf-covers','--grouping=author') },
    @{ name='grouping-author-folded'; args=@('--pdf-covers','--grouping=author','--fold-group') },
    @{ name='grouping-collections'; args=@('--pdf-covers','--grouping=collection','--es','--light','--reduce-motion') },
    @{ name='grouping-tags-list'; args=@('--pdf-covers','--grouping=tag','--list') },
    @{ name='grouping-reading-small-200'; args=@('--pdf-covers','--grouping=readingState','--size=640x480','--text-scale=2','--es','--light','--reduce-motion') },
    @{ name='bulk-remove-review'; args=@('--pdf-covers','--bulk-remove=review') },
    @{ name='bulk-remove-review-small-200'; args=@('--pdf-covers','--bulk-remove=review','--light','--es','--size=640x480','--text-scale=2','--reduce-motion') },
    @{ name='bulk-remove-applied'; args=@('--pdf-covers','--bulk-remove=apply','--reduce-motion') },
    @{ name='bulk-remove-cancelled-es'; args=@('--pdf-covers','--bulk-remove=cancel','--light','--es','--reduce-motion') },
    @{ name='bulk-remove-partial-es'; args=@('--pdf-covers','--bulk-remove=failure','--es','--reduce-motion') },
    @{ name='bulk-remove-readings-menu'; args=@('--pdf-covers','--bulk-remove=apply','--tools-menu','--reduce-motion') },
    @{ name='bulk-remove-last-page'; args=@('--pdf-covers','--bulk-remove=last-page','--light','--reduce-motion') },
    @{ name='bulk-covers-review'; args=@('--pdf-covers','--bulk-covers=review') },
    @{ name='bulk-covers-review-small-200'; args=@('--pdf-covers','--bulk-covers=review','--light','--es','--size=640x480','--text-scale=2','--reduce-motion') },
    @{ name='bulk-covers-applied'; args=@('--pdf-covers','--bulk-covers=apply') },
    @{ name='bulk-covers-pressure-es'; args=@('--pdf-covers','--bulk-covers=pressure','--light','--es','--reduce-motion') },
    @{ name='bulk-tags-long-small-200'; args=@('--pdf-covers','--bulk-organization=tag-picker','--bulk-long-target','--size=640x480','--text-scale=2','--reduce-motion') },
    @{ name='bulk-tags-picker-small-200'; args=@('--pdf-covers','--bulk-organization=tag-picker','--size=640x480','--text-scale=2','--light','--es','--reduce-motion') },
    @{ name='bulk-collections-picker'; args=@('--pdf-covers','--bulk-organization=collection-picker') },
    @{ name='bulk-tags-applied'; args=@('--pdf-covers','--bulk-organization=tag','--light','--es','--reduce-motion') },
    @{ name='bulk-collections-applied'; args=@('--pdf-covers','--bulk-organization=collection') },
    @{ name='bulk-collections-removed'; args=@('--pdf-covers','--bulk-organization=remove') },
    @{ name='bulk-archive-applied'; args=@('--pdf-covers','--bulk-organization=archive','--light','--es','--reduce-motion') },
    @{ name='library-selection-covers'; args=@('--pdf-covers','--library-selection') },
    @{ name='library-selection-list-es'; args=@('--pdf-covers','--library-selection','--list','--es','--light','--reduce-motion') },
    @{ name='library-selection-small-200'; args=@('--pdf-covers','--library-selection','--size=640x480','--text-scale=2') },
    @{ name='library-selection-menu-small-200'; args=@('--pdf-covers','--library-selection','--selection-menu','--size=640x480','--text-scale=2') },
    @{ name='library-selection-favorites'; args=@('--pdf-covers','--library-selection','--selection-favorite') },
    @{ name='filters-empty-smart'; args=@('--smart-view=missing') },
    @{ name='filters-reset-smart-es'; args=@('--pdf-covers','--smart-view=missing','--reset-library-filters','--es','--light','--reduce-motion') },
    @{ name='tags-manager'; args=@('--pdf-covers','--library-tags') },
    @{ name='tags-renamed'; args=@('--pdf-covers','--library-tags','--tag-management=rename') },
    @{ name='tags-color-small-200'; args=@('--pdf-covers','--library-tags','--tag-management=color','--size=640x480','--text-scale=2') },
    @{ name='tags-merge-picker-small-200'; args=@('--pdf-covers','--library-tags','--tag-management=picker','--size=640x480','--text-scale=2') },
    @{ name='tags-merge-paged'; args=@('--pdf-covers','--library-tags','--tag-management=paged-picker') },
    @{ name='tags-merge-review-small-200'; args=@('--pdf-covers','--library-tags','--tag-management=merge-review','--size=640x480','--text-scale=2','--es','--light','--reduce-motion') },
    @{ name='tags-merged'; args=@('--pdf-covers','--library-tags','--tag-management=merge') },
    @{ name='tags-delete-review-small-200'; args=@('--pdf-covers','--library-tags','--tag-management=delete-review','--size=640x480','--text-scale=2','--es','--light','--reduce-motion') },
    @{ name='tags-deleted-es'; args=@('--pdf-covers','--library-tags','--tag-management=delete','--es','--light','--reduce-motion') },
    @{ name='tags-failed-rename'; args=@('--pdf-covers','--library-tags','--tag-management=failure') },
    @{ name='tags-review-cancelled'; args=@('--pdf-covers','--library-tags','--tag-management=cancel') },
    @{ name='tags-actions'; args=@('--pdf-covers','--library-tags','--tag-management=actions') },
    @{ name='tags-filter-es'; args=@('--pdf-covers','--library-tags','--tag-filter','--es','--light','--reduce-motion') },
    @{ name='tags-small-200'; args=@('--pdf-covers','--library-tags','--size=640x480','--text-scale=2') },
    @{ name='smart-annotated'; args=@('--pdf-covers','--smart-view=annotated') },
    @{ name='smart-recent-es'; args=@('--pdf-covers','--smart-view=recentlyOpened','--es','--light','--reduce-motion') },
    @{ name='smart-menu-small-200'; args=@('--pdf-covers','--smart-view=annotated','--smart-menu','--size=640x480','--text-scale=2') },
    @{ name='collections-manager'; args=@('--pdf-covers','--collections') },
    @{ name='collections-actions-small-200'; args=@('--pdf-covers','--collections','--collection-actions','--size=640x480','--text-scale=2') },
    @{ name='collections-actions'; args=@('--pdf-covers','--collections','--collection-actions') },
    @{ name='collections-ordered'; args=@('--pdf-covers','--collections','--collection-order') },
    @{ name='collections-order-search-es'; args=@('--pdf-covers','--collections','--collection-order-search','--es','--light','--reduce-motion') },
    @{ name='collections-icon-small-200'; args=@('--pdf-covers','--collections','--collection-icon-review','--size=640x480','--text-scale=2') },
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
$duplicateCases = @($cases | Group-Object -Property name | Where-Object { $_.Count -gt 1 })
if ($duplicateCases.Count -gt 0) { throw 'Capture case names must be unique before any process is launched.' }
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
    $stderrPath = Join-Path $outputPath "$($case.name).stderr.log"
    $stdoutPath = Join-Path $outputPath "$($case.name).stdout.log"
    $process = Start-Process -FilePath $executablePath -ArgumentList $arguments -WorkingDirectory $repositoryPath -WindowStyle Hidden -RedirectStandardError $stderrPath -RedirectStandardOutput $stdoutPath -PassThru
    if (-not $process.WaitForExit(45000)) {
        $process.Kill()
        $process.WaitForExit()
        Get-Content -LiteralPath $stderrPath -Tail 12 | Write-Output
        throw "Capture timed out: $($case.name)"
    }
    if ($process.ExitCode -ne 0) {
        Get-Content -LiteralPath $stderrPath -Tail 12 | Write-Output
        throw "Capture failed ($($process.ExitCode)): $($case.name)"
    }
    if (-not (Test-Path -LiteralPath $imagePath) -or -not (Test-Path -LiteralPath $controlPath)) {
        throw "Native capture did not produce PNG and control bounds: $($case.name)"
    }
    $controls = Get-Content -LiteralPath $controlPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if (-not $controls.synthetic) { throw 'Only the isolated synthetic collection may be captured by this script.' }
    if ($case.name -in @('reader-reference','reader-reference-stacked')) {
        if (-not $controls.reference -or $controls.reference.images -lt 1 -or $controls.reference.page -ne 1 -or $controls.currentReaderPage -ne 0 -or $controls.reference.document -ne 'demo-0') { throw 'Page comparison did not retain independent pages and a native reference image.' }
    }
    if ($case.name -eq 'reader-reference-chooser') {
        if ($controls.referenceChooser.demoStage -ne 3 -or $controls.referenceChooser.pagedRows -ne 30 -or $controls.referenceChooser.count -ne 1 -or $controls.referenceChooser.rows -ne 1 -or -not $controls.referenceChooser.outsideVisible -or $controls.libraryDocumentCount -ne 110) { throw 'Reference selection did not page and search beyond the visible main catalogue page.' }
    }
    if ($case.name -eq 'reader-reference-sync') {
        if ($controls.referenceSync.demoStage -ne 10 -or -not $controls.referenceSync.differencesConfirmed -or @($controls.reference.differences).Count -ne 1 -or $controls.reference.differences[0] -ne 1 -or -not $controls.referenceSync.reverseConfirmed -or -not $controls.reference.bidirectional -or -not $controls.referenceSync.positionConfirmed -or $controls.reference.followPosition -or -not $controls.referenceSync.linkedConfirmed -or -not $controls.reference.followZoom -or -not $controls.reference.followPage -or [Math]::Abs($controls.referenceSync.primaryScale-1.1) -gt 0.0001 -or [Math]::Abs($controls.reference.scale-1.1) -gt 0.0001 -or $controls.currentReaderPage -ne 1 -or $controls.reference.page -ne 1) { throw 'Reference page/zoom/position following or independent manual zoom did not settle correctly.' }
    }
    if ($case.name -eq 'reader-reference-book') {
        $referenceClose = @($controls.controls | Where-Object { $_.label -eq 'Cerrar referencia' })
        $referenceMatch = @($controls.controls | Where-Object { $_.label -like 'Página 2*Mixed*' })
        if ($referenceMatch.Count -ne 1) { throw 'The reference search must display its selectable result.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @($referenceMatch[0].label)

        if ($controls.referenceAdmission.demoStage -ne 11 -or $controls.reference.page -ne 1 -or $controls.reference.find.matches -ne 1 -or $controls.reference.find.page -ne 1 -or $controls.documentFind -or $controls.reference.images -lt 1 -or $controls.referenceAdmission.nativeCloses -ne 0 -or -not $controls.reference.swapped -or $controls.reference.document -ne 'demo-1' -or [Math]::Abs($controls.reference.scale-1.2) -gt 0.0001 -or $controls.currentReaderPage -ne 0 -or $referenceClose.Count -ne 1 -or $referenceClose[0].rect[0] -ge (($controls.viewport[2]-$controls.viewport[0])/2)) { throw 'Swapping comparison positions did not preserve both readings or place the reference on the left.' }
    }
    if ($case.name -in @('reader-reference-links','reader-reference-external')) {
        if ($controls.referenceAdmission.demoStage -ne 13 -or $controls.reference.document -ne 'demo-1' -or -not $controls.reference.links.keyboardActivated -or $controls.currentReaderPage -ne 0 -or $controls.documentLinks.confirmationOpen) { throw 'Reference link activation changed the main reading or its confirmation state.' }
        if ($case.name -eq 'reader-reference-links' -and ($controls.reference.page -ne 1 -or $controls.reference.links.confirmationOpen)) { throw 'The reference internal link did not navigate its own PDF destination.' }
        if ($case.name -eq 'reader-reference-external') {
            if (-not $controls.reference.links.confirmationOpen -or $controls.reference.page -ne 0) { throw 'The reference external link must await its own confirmation.' }
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Cancelar','Abrir')
        }
    }
    if ($case.name -like 'rule-simulation-*') {
        $result=$controls.ruleSimulation
        if ($result.phase -ne 10 -or $result.pending) { throw 'Rule simulation did not settle.' }
        if ($case.name -eq 'rule-simulation-cancel') {
            if ($result.state -ne 'cancelled' -or $result.scanned -le 0 -or $result.ids.Count -ne 0) { throw 'Cancellation retained an incomplete simulation plan.' }
        } elseif ($case.name -eq 'rule-simulation-duplicates') {
            if ($result.state -ne 'complete' -or $result.scanned -ne 3 -or $result.matched -ne 2 -or $result.unverified -ne 0 -or ($result.ids -join '|') -ne 'demo-0|demo-1') { throw 'Full-hash duplicate verification is incorrect.' }
        } elseif ($result.state -ne 'complete' -or $result.scanned -ne 10 -or $result.matched -ne 10) { throw 'Extension simulation lost matching documents.' }
        $closeLabel=if ($case.args -contains '--es') {'Cerrar vista previa'} else {'Close preview'}
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @($closeLabel)
    }
    if ($case.name -like 'saved-rules-*') {
        $state=$controls.automationRules
        $total=if ($case.name -eq 'saved-rules-paged') {82} elseif ($case.name -eq 'saved-rules-review-small-200') {3} else {2}
        if ($state.phase -ne 10 -or $state.pending -or $state.failed -or $state.total -ne $total) {throw 'Saved rule reads or guarded mutations did not settle.'}
        if ($case.name -eq 'saved-rules-paged') {
            if ($state.offset -ne 80 -or $state.entries.Count -ne 2) {throw 'Saved rules must retrieve the next bounded page.'}
        } else {
            if (@($state.entries | Where-Object { $_.id -ceq 'legacy-rule' -and $_.actions -eq 8 }).Count -ne 1 -or @($state.entries | Where-Object { $_.id -ceq 'future-rule' -and -not $_.available }).Count -ne 1) {throw 'Inherited multi-action or future-format rules were lost.'}
        }
        if ($case.name -eq 'saved-rules-review-small-200') {
            if (-not $state.confirmDelete) {throw 'Rule deletion must remain at review before confirmation.'}
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Confirmar eliminación','Cancelar')
        } elseif ($case.name -eq 'saved-rules-changed') {
            if (@($state.entries | Where-Object name -CEQ 'Reading PDFs').Count -ne 0 -or ($state.entries | Where-Object id -CEQ 'future-rule').enabled) {throw 'Rule creation, toggling or deletion failed.'}
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('New rule')
        } else {
            $label=if ($case.name -eq 'saved-rules-legacy') {'Nueva regla'} else {'New rule'}
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @($label)
        }
    }
    if ($case.name -like 'revision-groups-*') {
        $state=$controls.revisionGroups
        $total=if ($case.name -eq 'revision-groups-legacy') {1} else {2}
        $current=if ($case.name -eq 'revision-groups-created') {'demo-0'} else {'demo-1'}
        if ($state.phase -ne 10 -or $state.pending -or $state.failed -or $state.total -ne $total -or $state.memberTotal -ne 2 -or $state.current -cne $current -or @($state.groups | Where-Object id -CEQ 'legacy-revisions').Count -ne 1) {throw 'Revision groups lost inherited data, membership or current-edition state.'}
        $labels=if ($case.name -eq 'revision-groups-current-small-200') {@('Marcar como actual')} elseif ($case.name -eq 'revision-groups-created') {@('New group','View editions')} else {@('Nuevo grupo','Ver ediciones')}
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels $labels
    }
    if ($case.name -like 'relations-*') {
        if ($controls.relations.phase -ne 10 -or $controls.relations.pending -or $controls.relations.writing -or $controls.relations.failed) { throw 'The connected relation workflow did not settle.' }
        if ($case.name -eq 'relations-backlinks') {
            if ($controls.relations.document -cne 'demo-1' -or $controls.relations.total -ne 3 -or @($controls.relations.entries | Where-Object { -not $_.backlink -or $_.other -cne 'demo-0' }).Count -ne 0) { throw 'Inverse relation ownership or directional presentation failed.' }
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Back to library','Open document')
        } else {
            if ($controls.relations.document -cne 'demo-0' -or $controls.relations.total -ne 5 -or -not $controls.relations.confirmRemove) { throw 'Relation removal review must preserve records before confirmation.' }
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Confirmar eliminación','Cancelar')
        }
    }
    if ($case.name -like 'reader-reference-note-bulk*') {
        $saved = $case.name -eq 'reader-reference-note-bulk-saved'
        $phase = if ($saved) { 41 } else { 40 }
        $selected = if ($saved) { 0 } else { 80 }
        if ($controls.referenceNotesPhase -ne $phase -or $controls.referenceAnnotations.document -cne 'demo-1' -or $controls.referenceAnnotations.total -ne 82 -or $controls.referenceAnnotations.offset -ne 0 -or $controls.referenceAnnotations.selected -ne $selected -or $controls.primaryAnnotations.total -ne 2 -or $controls.primaryAnnotations.selected -ne 0) { throw 'Reference bulk mutation or page-scoped selection failed.' }
        if ($saved) {
            if (@($controls.referenceAnnotations.entries | Where-Object { $_.review -ne 'accepted' -or 'Reference batch' -notin $_.tags -or 'Existing study' -notin $_.tags }).Count -ne 0) { throw 'Reference review and additive tags were not persisted together.' }
        } else {
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Quitar selección','Aceptar','Resolver')
        }
    }
    if ($case.name -like 'reader-reference-note-filters*') {
        $empty = $case.name -eq 'reader-reference-note-filters-empty'
        $phase = if ($empty) { 26 } else { 25 }
        $total = if ($empty) { 0 } else { 1 }
        if ($controls.referenceNotesPhase -ne $phase -or $controls.referenceAnnotations.document -cne 'demo-1' -or $controls.referenceAnnotations.filterCount -ne 4 -or $controls.referenceAnnotations.total -ne $total -or $controls.primaryAnnotations.total -ne 2 -or $controls.primaryAnnotations.filterCount -ne 0) { throw 'Reference filters or reset affected the wrong records or document.' }
        $labels = if ($empty) { @('Active filters (4)','Reset') } else { @('Filtros activos (4)','Restablecer') }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels $labels
    }
    if ($case.name -eq 'reader-reference-notes') {
        if ($controls.referenceNotesPhase -ne 12 -or $controls.referenceAnnotations.document -cne 'demo-1' -or $controls.referenceAnnotations.total -ne 1 -or $controls.referenceAnnotations.entries[0].note -cne 'Final reference note' -or $controls.primaryAnnotations.total -ne 2 -or $controls.retiredAnnotationWrites -ne 0) { throw 'Reference note creation, editing, removal or document ownership failed.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Notas','Editar','Eliminar')
    }
    if ($case.name -eq 'reader-reference-notes-shared-close') {
        if ($controls.referenceNotesPhase -ne 16 -or $controls.reference -or $controls.primaryAnnotations.total -ne 3 -or $controls.retiredAnnotationWrites -ne 0 -or @($controls.primaryAnnotations.entries | Where-Object note -CEQ 'Updated reference note').Count -ne 1 -or $controls.referenceAdmission.nativeCloses -ne 0) { throw 'An accepted reference write was lost or misrouted after shared-pane closure.' }
    }
    if ($case.name -eq 'reader-reference-notes-editor-small-200') {
        if ($controls.referenceNotesPhase -ne 10 -or -not $controls.referenceAnnotations.editing -or $controls.referenceAnnotations.document -cne 'demo-1' -or $controls.primaryAnnotations.total -ne 2) { throw 'The compact reference editor did not retain its own target.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Notas','Guardar nota','Cancelar')
    }
    if ($case.name -in @('reader-reference-marks','reader-reference-marks-shared','reader-reference-marks-small-200')) {
        $shared = $case.name -ne 'reader-reference-marks'
        $expectedCount = if ($shared) { 1 } else { 3 }
        $expectedDocument = if ($shared) { 'demo-0' } else { 'demo-1' }
        $expectedColor = if ($shared) { 'Blue' } else { 'Green' }
        if ($controls.referenceMarksPhase -ne 6 -or $controls.currentReaderPage -ne 0 -or $controls.reference.page -ne 0 -or $controls.annotationPaint.document -cne 'demo-0' -or $controls.annotationPaint.marks -ne 1 -or $controls.annotationPaint.details[0].colors[0] -cne 'Blue' -or $controls.referenceMarks.document -cne $expectedDocument -or $controls.referenceMarks.marks -ne $expectedCount -or $controls.referenceMarks.details[0].colors[0] -cne $expectedColor -or $controls.referenceMarks.residentPages -ne 1 -or $controls.referenceMarks.readyPages -ne 1) { throw 'Visible annotation marks did not retain document ownership, update across shared views or reload after navigation.' }
        if ($case.name -eq 'reader-reference-marks-small-200' -and $controls.referenceMarksRotation -ne 90) { throw 'The compact reference did not retain its rotated annotation projection.' }
    }
    if ($case.name -in @('reader-reference-selection','reader-reference-selection-shared','reader-reference-selection-small-200')) {
        if ($controls.referenceSelectionPhase -ne 10 -or $controls.readingFocus -cne 'Reference' -or -not $controls.referenceSelection.enabled -or $controls.textSelection.start[0] -ne 0 -or $controls.textSelection.end[1] -ne 1 -or $controls.textSelection.residentPages -gt 2 -or $controls.referenceSelection.residentPages -gt 2) { throw 'Reference selection did not retain isolated source selection and bounded text ownership.' }
        $expectedDocument = if ($case.name -eq 'reader-reference-selection-shared') { 'demo-0' } else { 'demo-1' }
        $expectedText = if ($case.name -eq 'reader-reference-selection-shared') { 'Yamantaka Rust: text, links and vector geometry' } else { 'Mixed landscape page' }
        if ($controls.selectionClipboard[0] -cne $expectedDocument -or $controls.selectionClipboard[1].Trim() -cne $expectedText) { throw 'The scoped selection copy command returned the wrong PDF passage.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Copiar','Borrar','Terminar')
    }
    if ($case.name -eq 'reader-reference-selection-close') {
        if ($controls.referenceSelectionPhase -ne 14 -or $controls.reference -or $controls.referenceSelection -or $controls.selectionClipboard[0] -cne 'demo-0' -or $controls.selectionClipboard[1] -cne 'Y' -or $controls.referenceAdmission.nativeCloses -ne 1 -or $controls.textSelection.end[1] -ne 1) { throw 'Reference close did not preserve independent source selection and copy.' }
    }
    if ($case.name -in @('reader-reference-history','reader-reference-history-small-200','reader-primary-history')) {
        if ($controls.referenceShortcutPhase -ne 34 -or $controls.currentReaderPage -ne 1 -or $controls.reference.page -ne 1 -or $controls.primaryNavigation.markA -ne 1 -or $null -ne $controls.primaryNavigation.markB -or $controls.referenceNavigation.markA -ne 0 -or $controls.referenceNavigation.markB -ne 1 -or $controls.primaryNavigation.forward -or $controls.referenceNavigation.forward) { throw 'Independent history traversal, branch replacement or A/B marks failed.' }
        if ($case.name -eq 'reader-primary-history') {
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Positions','Save position A','Save position B')
        } else {
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Posiciones','Guardar posición A')
        }
    }
    if ($case.name -in @('reader-reference-shortcuts','reader-reference-shortcuts-small-200')) {
        if ($controls.referenceShortcutPhase -ne 14 -or $controls.readingFocus -cne 'Reference' -or $controls.currentReaderPage -ne 1 -or $controls.reference.page -ne 0 -or $controls.reference.document -ne 'demo-1' -or $controls.documentFind -or $controls.reference.find.matches -lt 2) { throw 'Keyboard and pointer routing did not keep independent page, zoom and find state.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Lectura de referencia · F6 cambia de panel')
    }
    if ($case.name -eq 'reader-reference-shortcuts-close') {
        if ($controls.referenceShortcutPhase -ne 16 -or $controls.readingFocus -cne 'Primary' -or $controls.reference -or $controls.currentReaderPage -ne 1 -or $controls.referenceAdmission.nativeCloses -ne 1) { throw 'Closing the focused swapped reference did not preserve the main reading.' }
    }
    if ($case.name -in @('reader-reference-text','reader-reference-text-small-200')) {
        if ($controls.referenceAdmission.demoStage -ne 39 -or $controls.reference.document -ne 'demo-1' -or $controls.reference.page -ne 1 -or $controls.reference.text.document -ne 'demo-1' -or $controls.reference.text.page -ne 1 -or -not $controls.reference.text.ready -or $controls.reference.text.failed -or $controls.reference.text.characters -lt 1 -or $controls.currentReaderPage -ne 0) { throw 'Reference page text did not recover independently after cancelled requests and navigator closure.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Texto','Copiar texto')
        if ($case.name -eq 'reader-reference-text') {
            if ($controls.pageText.document -ne 'demo-0' -or $controls.pageText.page -ne 0 -or -not $controls.pageText.ready) { throw 'Reference text replaced the primary page payload.' }
            if (@($controls.controls | Where-Object { $_.label -ceq 'Copiar texto' -and $_.enabled }).Count -lt 2) { throw 'Both independent text views must offer their own copy control.' }
        }
    }
    if ($case.name -eq 'reader-reference-text-empty') {
        if ($controls.referenceAdmission.demoStage -ne 40 -or $controls.reference.text.page -ne 0 -or -not $controls.reference.text.ready -or $controls.reference.text.failed -or $controls.reference.text.characters -ne 0 -or $controls.currentReaderPage -ne 0) { throw 'The scanned reference page did not reach its empty-text state.' }
        if (@($controls.controls | Where-Object { $_.label -ceq 'Copiar texto' }).Count -ne 0) { throw 'An empty PDF page must not offer copying text.' }
    }
    if ($case.name -in @('reader-reference-bookmarks','reader-reference-bookmarks-small-200')) {
        if ($controls.referenceAdmission.demoStage -ne 28 -or $controls.reference.document -ne 'demo-1' -or $controls.reference.page -ne 1 -or $controls.reference.bookmarks.total -ne 1 -or $controls.reference.bookmarks.entries[0].label -cne 'Compare this passage' -or $controls.documentBookmarks.total -ne 2 -or $controls.currentReaderPage -ne 0) { throw 'Reference bookmark CRUD or independent navigation did not complete.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Marcadores','Guardar página 2')
    }
    if ($case.name -eq 'reader-reference-bookmarks-shared') {
        if ($controls.referenceAdmission.demoStage -ne 35 -or $controls.reference -or $controls.documentBookmarks.total -ne 1 -or $controls.documentBookmarks.entries[0].label -cne 'Shared reading note' -or $controls.documentBookmarks.entries[0].page_index -ne 1 -or $controls.currentReaderPage -ne 0) { throw 'Shared bookmark changes or the accepted write after reference closure were lost.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Página 2')
    }
    if ($case.name -like 'reader-reference-thumbnails*') {
        if ($controls.referenceAdmission.demoStage -ne 19 -or $controls.reference.document -ne 'demo-1' -or $controls.reference.page -ne 1 -or $controls.reference.thumbnailSlots -lt 1 -or $controls.reference.thumbnailSlots -gt 16 -or $controls.visibleThumbnailSlots -ne 0 -or $controls.currentReaderPage -ne 0) { throw 'Reference thumbnails did not navigate independently within their bounded resident slots.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Páginas','Página 2')
    }
    if ($case.name -eq 'reader-reference-outline') {
        if ($controls.referenceAdmission.demoStage -ne 16 -or $controls.reference.document -ne 'demo-1' -or $controls.reference.page -ne 1 -or [Math]::Abs($controls.reference.scale-1.5) -gt 0.0001 -or $controls.reference.outline.entries -ne 6 -or $controls.reference.outline.rows -ne 6 -or $controls.documentOutline.entries -ne 0 -or $controls.currentReaderPage -ne 0) { throw 'The reference outline did not navigate independently through its resolved PDF destination.' }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Make room for thought · 2')
    }
    if ($case.name -eq 'reader-reference-closed') {
        if ($controls.referenceAdmission.demoStage -ne 5 -or $controls.referenceAdmission.nativeCloses -ne 1 -or $controls.reference -or $controls.currentReaderPage -ne 0) { throw 'Reference closure did not acknowledge one native close and preserve the primary reading.' }
    }
    if ($case.name -eq 'reader-reference-password-retry') {
        if ($controls.referenceAdmission.demoStage -ne 3 -or $controls.referenceAdmission.passwordAttempts -ne 2 -or $controls.reference.document -ne 'demo-1' -or $controls.reference.images -lt 1 -or $controls.currentReaderPage -ne 0) { throw 'The protected reference did not recover from an incorrect password without replacing the primary reader.' }
    }
    if ($case.name -in @('reader-reference-password-cancel','reader-reference-password-import')) {
        if ($controls.referenceAdmission.demoStage -ne 7 -or $controls.referenceAdmission.passwordAttempts -ne 0 -or $controls.reference.document -ne 'demo-0' -or $controls.currentReaderPage -ne 0) { throw 'Cancelling reference unlocking did not retain the original reading and reference.' }
    }
    if ($case.name -like 'bulk-remove-*') {
        $result = $controls.librarySelection.removal
        if ($null -eq $result -or -not $controls.libraryQueryLoaded -or $controls.librarySelection.writing) { throw 'Library removal has not reached its verified state.' }
        if ($case.name -like '*review*') {
            if ($result.mode -ne 'review' -or $result.selected -ne 3 -or $controls.libraryDocumentCount -ne 10) { throw 'Removal review must preserve its fixed selection and records.' }
            $cancelLabel = if ($case.args -contains '--es') { 'Cancelar' } else { 'Cancel' }
            $removeLabel = if ($case.args -contains '--es') { 'Retirar' } else { 'Remove' }
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @($cancelLabel,$removeLabel)
        } else {
            $expected = switch ($result.mode) { 'cancel' { 10 }; 'failure' { 8 }; 'last-page' { 80 }; default { 7 } }
            if ($controls.libraryDocumentCount -ne $expected -or $result.remaining -ne $expected -or -not $result.continuitySaved) { throw 'Removal count or acknowledged continuity is incorrect.' }
            if ($result.mode -eq 'last-page') { if ($result.recoveredOffset -ne 0) { throw 'Removing the final page must recover a valid catalogue page.' } }
            elseif (-not $result.otherReadingKept) { throw 'Unselected reading context was not preserved.' }
            if ($result.mode -in @('apply','failure') -and -not $result.sessionsSettled) { throw 'Selected reading sessions must finish saving before removal.' }
            if ($result.mode -in @('apply','failure') -and -not $result.otherReadingReopened) { throw 'The retained reading must reopen with its original zoom.' }
            if ($case.name -eq 'bulk-remove-readings-menu') {
                & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Open readings (1)','Reopen last reading')
            }
        }
    }
    if ($case.name -like 'bulk-covers-*') {
        $coverResult = $controls.libraryCovers.bulkRegeneration
        if ($null -eq $coverResult -or $coverResult.targets.Count -ne 3 -or $controls.librarySelection.writing -or $controls.libraryCovers.paused -ne 0) { throw 'Bulk cover preview has not reached its verified state.' }
        if ($case.name -like '*review*') {
            if (-not $coverResult.review -or -not $coverResult.oldImagesKept -or $controls.librarySelection.ids.Count -ne 3) { throw 'Cover review must preserve all selected covers.' }
            $cancelLabel = if ($case.args -contains '--es') { 'Cancelar' } else { 'Cancel' }
            $applyLabel = if ($case.args -contains '--es') { 'Regenerar' } else { 'Regenerate' }
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @($cancelLabel,$applyLabel)
        } else {
            if ($coverResult.review -or -not $coverResult.newImages -or $coverResult.manualPageBefore -ne 1 -or $coverResult.automaticPageAfter -ne 0 -or $controls.libraryCovers.deferredUploads -ne 0) { throw 'Regeneration must replace the persisted manual page and visible image.' }
            if ($case.name -eq 'bulk-covers-pressure-es' -and -not $coverResult.pressureRecovery) { throw 'Expected recovery after governed GPU capacity was restored.' }
        }
    }
    if ($case.name -match '^bulk-(tags|collections|archive)-') {
        if (-not $controls.libraryQueryLoaded -or $controls.librarySelection.writing -or $controls.bulkOrganization.selected -ne 3) { throw 'Bulk organization did not reach its verified terminal state.' }
        if ($case.name -like '*picker*' -or $case.name -eq 'bulk-tags-long-small-200') {
            if (-not $controls.bulkTargetPicker.open -or $controls.bulkTargetPicker.busy -or $controls.bulkTargetPicker.failed -or $controls.bulkTargetPicker.total -ne 81 -or $controls.bulkTargetPicker.offset -ne 80 -or $controls.bulkTargetPicker.visible -ne 1) { throw 'The picker must resolve its target on the second bounded page.' }
            $visibleTarget = if ($case.name -eq 'bulk-tags-long-small-200') { 'Study — littérature française, manuscrits et annotations · 0 books' } elseif ($case.args -contains '--es') { 'Study · 0 libros' } else { 'Study · 0 books' }
            $visibleCancel = if ($case.args -contains '--es') { 'Cancelar' } else { 'Cancel' }
            $visiblePrevious = if ($case.args -contains '--es') { 'Anterior' } else { 'Previous' }
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @($visibleTarget,$visibleCancel,$visiblePrevious)

        } else {
            $expectedMembers = if ($case.name -eq 'bulk-collections-removed') { 1 } else { 3 }
            if ($controls.bulkTargetPicker.open -or $controls.bulkOrganization.members -ne $expectedMembers -or $controls.libraryDocumentCount -ne $expectedMembers -or $controls.librarySelection.ids.Count -ne 0) { throw 'Bulk organization membership or filtered readback is incorrect.' }
            if ($case.name -eq 'bulk-tags-applied' -and $controls.bulkOrganization.annotationUses -ne 1) { throw 'Document tag assignment must preserve the existing annotation membership.' }
        }
    }
    if ($case.name -eq 'restore-review') {
        if (-not $controls.restore.reviewed -or $controls.restore.busy -or $controls.restore.books -ne 10) { throw 'Restore review did not load the synthetic backup.' }
    }
    if ($case.name -like 'backup-*') {
        $backup=$controls.backup
        if (-not $backup.verified -or $backup.busy) { throw 'Native backup scenario did not finish.' }
        if ($case.name -eq 'backup-cancelled-es') {
            if (-not $backup.cancelled -or $backup.saved) { throw 'Backup cancellation was not preserved.' }
        } elseif ($case.name -eq 'backup-blocked') {
            if (-not $backup.failed -or $backup.saved) { throw 'Library-internal backup destination was not rejected.' }
        } elseif (-not $backup.saved) { throw 'Backup was not saved and verified.' }
        if ($case.name -eq 'backup-saved-small-200') { & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Exportar copia JSON') }
    }
    if ($case.name -like 'migration-*') {
        $migration=$controls.legacyImport
        if (-not $migration.verified -or $migration.busy -or -not $migration.open) { throw 'Native migration scenario did not finish.' }
        if ($case.name -in @('migration-stale','migration-cancelled','migration-blocked-small-200')) {
            if (-not $migration.failed -or $migration.copied) { throw 'Migration rejection or cancellation must not install a copy.' }
        } elseif ($case.name -eq 'migration-selected' -and -not $migration.selectedNext) { throw 'Imported library was not selected for next launch.' }
        elseif ($case.name -eq 'migration-original' -and -not $migration.originalNext) { throw 'Original Rust library was not restored for next launch.' }
        elseif ($case.name -eq 'migration-copy-es' -and -not $migration.copied) { throw 'Independent migration copy was not created.' }
        if ($case.name -eq 'migration-selected') { & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Close','Use original') }
        if ($case.name -eq 'migration-blocked-small-200') { & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Cerrar','Revisar origen') }
    }
    if ($case.name -like 'grouping-*') {
        $grouping=$controls.libraryGrouping
        if (-not $grouping.verified -or $grouping.groups.Count -lt 2) { throw 'Grouping did not verify the native catalogue result.' }
        $groupedCount=($grouping.groups | Measure-Object -Property count -Sum).Sum
        if ($groupedCount -ne $controls.libraryDocumentCount) { throw 'Grouping lost or duplicated documents.' }
        if ($case.name -eq 'grouping-author-folded') {
            if (-not $grouping.groups[0].collapsed -or $grouping.visible.Count -ge $groupedCount) { throw 'Folding must remove only that group from visible navigation.' }
        } elseif ($grouping.visible.Count -ne $groupedCount) { throw 'Expanded grouping must retain every page document.' }
    }
    if ($case.name -like 'library-selection-*') {
        if (-not $controls.libraryQueryLoaded -or $controls.librarySelection.writing -or $null -ne $controls.librarySelection.preview) { throw 'Library selection preview has not completed.' }
        if ($case.name -eq 'library-selection-favorites') {
            if ($controls.librarySelection.ids.Count -ne 0 -or $controls.libraryVisibleFavorites -ne 3 -or $controls.librarySelection.verifiedFavoriteTargets.Count -ne 3) { throw 'Bulk favorite targets or persisted results are incorrect.' }
        } elseif ($controls.librarySelection.ids.Count -ne 3) { throw 'Range and additive gestures must leave three selected books.' }
        if ($case.name -eq 'library-selection-small-200') {
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Selection (3)')
        } elseif ($case.name -eq 'library-selection-menu-small-200') {
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Add to favorites','Remove from favorites')
        }

    }
    if ($case.name -eq 'filters-empty-smart') {
        if (-not $controls.libraryQueryLoaded -or -not $controls.libraryFiltersActive -or $controls.libraryDocumentCount -ne 0 -or $controls.smartView -ne 'missing') {
            throw 'The empty smart view must finish its query and retain its active filter.'
        }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Reset filters')
    } elseif ($case.name -eq 'filters-reset-smart-es') {
        if (-not $controls.libraryQueryLoaded -or $controls.libraryFiltersActive -or $controls.libraryDocumentCount -ne 10 -or $null -ne $controls.smartView) {
            throw 'Reset must clear every filter and return the ten synthetic books.'
        }
    }

    if ($case.args -like '--tag-management=*') {
        $expectedFailure=$case.name -eq 'tags-failed-rename'
        if (-not $controls.libraryTags.managementVerified -or $controls.libraryTags.failed -ne $expectedFailure -or $controls.libraryTags.busy -or -not $controls.libraryQueryLoaded) { throw 'Shared tag workflow did not settle correctly.' }
        $changed=$case.name -in @('tags-merged','tags-deleted-es')
        $expectedFilters=if ($changed) { 'Reference' } else { 'Reading notes|Reference' }
        if (($controls.libraryTags.filters -join '|') -ne $expectedFilters) { throw 'Selected tag filters were not reconciled.' }
        $expectedBooks=if ($case.name -eq 'tags-deleted-es') { 1 } else { 2 }
        if ($controls.libraryDocumentCount -ne $expectedBooks) { throw 'Tag filter query returned an incorrect book count.' }
        if ($case.name -eq 'tags-merged' -and ($controls.libraryTags.entries[0].books -ne 2 -or $controls.libraryTags.entries[0].annotations -ne 1)) { throw 'Merged tag lost book or annotation associations.' }
        if ($case.name -eq 'tags-merge-paged' -and ($controls.libraryTags.mergeTotal -ne 82 -or $controls.libraryTags.mergeOffset -ne 80 -or $controls.libraryTags.mergeVisible -ne 2)) { throw 'Merge destinations did not reach the second bounded page.' }
        $visibleLabels=switch ($case.name) {
            'tags-color-small-200' { @('Save color','Cancel') }
            'tags-merge-picker-small-200' { @('Search destination tag','Reference · 1 book','Cancel') }
            'tags-merge-review-small-200' { @('Cancelar','Fusionar') }
            'tags-delete-review-small-200' { @('Cancelar','Eliminar') }
            'tags-actions' { @('Rename','Change color','Merge into…','Delete tag…') }
            'tags-failed-rename' { @('New name','Save name','Cancel') }
            default { @() }
        }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels $visibleLabels
    }
    if ($case.name -match '^collections-(actions|actions-small-200|ordered|order-search-es|icon-small-200)$') {
        if (-not $controls.collections.loaded -or $controls.collections.failed -or $controls.collections.busy -or -not $controls.collections.orderVerified -or $controls.collections.total -ne 2 -or $controls.collections.assigned -ne 1) { throw 'Collection changes did not settle correctly.' }
        $expectedNames = if ($case.name -eq 'collections-order-search-es') { @('Reading seminar','Archive notes') } else { @('Archive notes','Reading seminar') }
        if (($controls.collections.entries.name -join '|') -ne ($expectedNames -join '|')) { throw 'Collection order differs from the expected result.' }
        if (($controls.collections.entries | Where-Object name -eq 'Reading seminar').icon -ne 'reference') { throw 'Saved collection icon did not reload.' }
        if ($case.name -eq 'collections-icon-small-200') {
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Reference','Star','Cancel')
        } elseif ($case.name -in @('collections-actions','collections-actions-small-200')) {
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Move earlier','Move later','Change icon')
        } elseif ($case.name -eq 'collections-ordered') {
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Refresh','View books','Collection actions')
            $createControl = @($controls.controls | Where-Object label -eq 'Create')[0]
            $refreshControl = @($controls.controls | Where-Object label -eq 'Refresh')[0]
            if (-not $createControl -or -not $refreshControl -or [Math]::Abs($refreshControl.rect[1]-$createControl.rect[1]) -gt 120) { throw 'Collection form rows reserve excessive empty height.' }
        }
    }
    if ($case.name -match '^(tags|collections)-(manager|small-200)$') {
        $visibleLabels = @()
        if ($case.name -eq 'tags-small-200') {
            $visibleLabels = if ($case.args -contains '--es') { @('En este libro: Study','Filtrar biblioteca: Study') } else { @('On this book: Study','Filter library: Study') }
        } elseif ($case.name -eq 'collections-small-200') {
            $visibleLabels = if ($case.args -contains '--es') { @('Ver libros','Acciones de colección') } else { @('View books','Collection actions') }
        }
        & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels $visibleLabels
    }
    $export = $null
    if ($controls.annotationExport.artifact) {
        $artifactName = [string]$controls.annotationExport.artifact
        if ([IO.Path]::GetFileName($artifactName) -ne $artifactName) { throw 'Export artifact must be a file name.' }
        $artifactPath = Join-Path $outputPath $artifactName
        if (-not $controls.annotationExport.saved -or $controls.annotationExport.pending -or -not (Test-Path -LiteralPath $artifactPath -PathType Leaf)) { throw 'Export capture must contain a successfully published file.' }
        $export = [ordered]@{ file=$artifactName; sha256=(Get-FileHash -LiteralPath $artifactPath -Algorithm SHA256).Hash; count=$controls.annotationExport.count }
        if ($case.name -like 'reader-reference-note-export-*') {
            if ($controls.annotationExport.document -cne 'demo-1' -or $controls.annotationExport.count -ne 82 -or $controls.referenceNotesPhase -ne 41 -or $controls.primaryAnnotations.total -ne 2) { throw 'Reference export lost its source identity or omitted records beyond the first list page.' }
            if ($case.name -like '*-json') {
                $document = Get-Content -LiteralPath $artifactPath -Raw -Encoding UTF8 | ConvertFrom-Json
                if ($document.metadata.documentId -cne 'demo-1' -or $document.annotations.Count -ne 82 -or @($document.annotations | Where-Object { $_.id -notlike 'reference-bulk-*' }).Count -ne 0) { throw 'Reference JSON contains the wrong document or incomplete records.' }
            } else {
                $markdown = Get-Content -LiteralPath $artifactPath -Raw -Encoding UTF8
                if ($markdown -notmatch 'Reference bulk note 000' -or $markdown -notmatch 'Reference bulk note 081' -or $markdown -match 'Source note [AB]') { throw 'Reference Markdown omitted notes or included the primary document.' }
            }
            & "$PSScriptRoot/check_rust_control_bounds.ps1" -ControlFiles @($controlPath) -VisibleLabels @('Markdown','JSON')
        }
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
$sources = @(Get-ChildItem -LiteralPath "$repositoryPath/rust" -File -Recurse | Where-Object { $_.Extension -in @('.rs','.toml') } | Sort-Object FullName | ForEach-Object {
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
