# Bulk tags in reader Notes

The selected-notes action group now adds one tag to the selected rows on the current list page. It preserves existing tags and uses the typed AddTag mutation, the application actor and the repository's transactional batch. Existing normalized names reuse the same tag definition, including case variants. No SQL runs in the UI.

The labeled single-line field, visible character count and Add to selected control use the existing theme and English/Spanish copy. The action requires a nonblank name of at most 80 user-perceived Unicode characters, matching the legacy dialog's visible-character limit. Extended grapheme counting preserves emoji and combining accents. The repository's 4096-byte tag boundary also applies. The edit field bounds scalar input and disables submission when a grapheme or byte limit is exceeded.

Review and tag commands share the same selected-ID snapshot. The UI freezes up to 80 IDs from its current page, disables actions during editing or pending work, and clears the tag draft with successful refresh, new reads, changed filters and document switches. Failed mutations preserve the draft and selection. The existing repository refuses a thirteenth distinct tag and rolls back the entire batch.

## Verification

Two new SQLite-backed controller regressions verify selected-only addition, preservation of existing tags, unchanged note/review data, trimmed names, case-insensitive idempotency, pending-work exclusion and released governed memory. Validation rejects blank/81-character names while accepting 80 emoji or 80 combining-accent graphemes. A late twelve-tag target causes rollback for all selected notes and leaves the failed draft available for recovery. Filter and document changes clear it.

Formatting, warning-denied workspace/all-target Clippy and the full locked workspace test suite pass. Local log: `build/verification/annotation-tag-bulk-tests.log`. The alternative executable build has its two expected output-path warnings; Clippy passes without warnings.

Three native wgpu captures use synthetic annotations. Dark English shows the tag draft with three selected rows. Light Spanish shows the saved tag and cleared selection. The export case traverses the real worker/publication path and publishes JSON containing all 181 annotations, with exactly three carrying Study. All cases retain 181 visible PDF marks, page 0 and zoom 1.0. The manifest hashes match the executable, lockfile, all UI Rust sources, images, control files and exported JSON.

Capture directory: `build/rust-visual/annotation-tag-bulk-native`. Preview executable: `build/verification/yamantaka-annotation-tag-bulk-preview.exe`. Manifest SHA-256: `01F86302F0765C9F43B028796F7D9E0766655A7AA764D886A20F6B8C5D1E8086`.

## Remaining scope

Synthetic capture hooks configure the draft and selection, then invoke the connected actor command. They do not prove physical keyboard/pointer interaction, compact scrolling, Narrator or monitor DPI acceptance. Selection remains scoped to one list page. Cross-document review, mutation history and broader annotation acceptance remain pending. No full rewrite or release completion is claimed.
