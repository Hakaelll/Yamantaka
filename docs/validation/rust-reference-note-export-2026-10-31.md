# Reference note export checkpoint

Reference Notes now provides Markdown and JSON export through the same ExportState presentation, native destination selector, application request, read-only export worker, and guarded publication path as the main reader. The extracted view emits a format choice. The application resolves an explicit Primary, Reference, or Review target before constructing the owned request.

A reference document request freezes its identity, display title, author, and reading progress. It exports all saved document annotations, independently of list filters, selection, and the visible 80-row batch. Global review keeps its existing filtered-page export semantics. One shared pending state prevents parallel UI submissions, retains scoped cancellation and exact-token reply acceptance, and identifies another document's result when viewed from a different pane.

Closing a reference does not cancel or retarget an accepted export. Shutdown retains the existing cancellation and worker-join behavior. Existing destination protection, coherent SQLite snapshots, publication, and error handling stay in the shared service. An unsaved editor disables starting a new export for that reading. Global review export also respects drafts in either reader pane.

## Evidence

The final binary compiled offline, and cargo fmt --all --check passed. Four native scenarios completed through tool/capture_rust_ui.ps1:

- reader-reference-note-export-json reviews and tags 80 of 82 synthetic reference notes, then exports the complete reference document. The resulting version 1 JSON contains exactly 82 records and metadata.documentId demo-1. Every exported ID belongs to that reference fixture. The main document retains its two notes. The final Spanish dark-theme capture checks visible Markdown and JSON controls.
- reader-reference-note-export-markdown repeats the complete export in English and light theme. The file includes the first and last reference notes and excludes the main document's Source note A and Source note B. The export reports 82 records.
- annotation-export-json retains the existing main-reader export path and produces 181 notes after extracting the presentation.
- global-notes-export-markdown-es retains the global filtered-page path and produces 80 notes in Spanish and light theme.

The new export scenarios use an isolated file-backed synthetic store so that the worker opens its production read-only connection. Seeded annotations use valid RFC 3339 timestamps, as required by the formatter. No timestamp validation or publication checks were relaxed. Draft and selection setup is programmatic; typed mutations and export use the real runtime and SQLite worker. These runs do not exercise the physical Windows save dialog or pointer input.

The [capture manifest](../screenshots/rust-reference-note-export/capture-manifest.json) records the four export file hashes, counts, executable hash, Rust source hashes, images, and control files. All were checked against the final local artifacts before publication. Export files remain local under build/rust-visual/reference-note-export; published evidence contains synthetic captures and their manifests. No personal PDFs or library data were used, no Computer Use was used, and no standalone tests or broad workspace suite were added or run.

## Remaining acceptance

Physical destination-selector interaction, compact export controls, cancellation and occupied-destination presentation, reference closure during a pending export, shared-state scope changes, fault injection, and resource-pressure acceptance need further interaction evidence. Physical DPI, Narrator, complete performance comparison, and Release packaging remain pending. This checkpoint does not establish complete annotation or rewrite acceptance.
