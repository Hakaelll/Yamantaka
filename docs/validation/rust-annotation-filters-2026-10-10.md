# Reader annotation filters

The Notes panel now combines passage/note text, an exact normalized tag name, annotation kind and editorial review state. The six persisted mark kinds and all three review states use the existing typed repository query. Each field has an accessible label and English/Spanish copy. Applying a draft starts at the first page; the active-filter count and Reset remain visible when the form is collapsed. Empty filtered results explain how to recover.

Draft fields do not change the query until Apply filters. Applying cancels an older read and rejects its late reply by request identity. Filters are disabled while an editor or accepted mutation exists, preserving unsaved work and submitted writes. Pagination and subsequent mutations use the applied query. Changing documents resets the filters. Applying a filter releases the previous list without changing the PDF page, zoom or annotation-geometry revision.

PDF marks use the complete visible-page geometry query independently of the filtered Notes list. Markdown/JSON export still includes the whole document, as the export controls explain. SQL and PDFium calls remain outside the UI.

## Verification

Two controller regressions exercise real SQLite filtering, trimmed text/tag values, combined kind/review queries, pagination, no matches, Reset, document changes and released governed memory. The cancellation regression proves that a stale read cannot replace the newer request and that filters cannot cancel an accepted write or discard an editor.

Formatting, warning-denied workspace/all-target Clippy and the full locked workspace tests pass. The log is `build/verification/annotation-filter-tests.log`. The alternative preview executable build emits only its two expected output-path warnings; the separate Clippy invocation has no warnings.

Five final native captures use synthetic records and the wgpu presenter. Dark English and light Spanish show the expanded filter form. An empty-results capture shows recovery copy and Reset. The export case publishes JSON containing all 181 annotations, including `synthetic-mark-180`, while the list contains only the 20 matching annotations. All four normal-size cases retain 181 painted marks and page 0 at zoom 1.0. The 640 × 480 case with text at 200% retains 20 matching records, but its initial drawer only shows the heading and hides the PDF. It is not evidence of compact scrolling or keyboard acceptance.

Executable, Cargo.lock, every UI Rust source, screenshot, control file and the exported JSON hashes match the final capture manifest. Capture directory: `build/rust-visual/annotation-filter-native-final`. Preview executable: `build/verification/yamantaka-annotation-filter-preview.exe`. Manifest SHA-256: `01E4AC28D8905F29BEE6B6367438ECBC2FBCCE3DD407D38D8F308F39A2D23659`.

## Remaining scope

Global review across documents, bulk review actions, filter persistence, mutation history and broader annotation interaction acceptance remain pending. These captures set the draft through the synthetic preview hook; they do not certify physical pointer/keyboard interaction, Narrator or monitor DPI. Expanded filters and long note lists require scrolling in this narrow panel; compact access remains an acceptance task. No full rewrite or release completion is claimed.
