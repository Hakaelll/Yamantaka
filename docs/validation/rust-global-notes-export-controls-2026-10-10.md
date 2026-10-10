# Global notes export controls

Library notes now exposes Markdown and JSON save controls through the existing native save selector. The screen explicitly states that global review export includes its current filtered page, capped at 80 notes. The applied filters and loaded offset become an owned Review request. Reader export continues to include every annotation in its document.

Both destinations use the shared typed export state, cancel control and recovery messages. Results require the expected Document or Review scope and matching token. Published success survives cancellation arriving before its reply. Moving between screens does not cancel accepted export work; the state identifies a global export or an earlier document when its context differs. Shutdown cancels outstanding work through the existing lifecycle.

Global controls are disabled during annotation reads/mutations, with no records or while a reader draft is open. Checking readiness does not allocate a new query each frame. Only starting an export snapshots its filters. No SQL or PDFium calls are added to UI components.

## Verification

Formatting and warning-denied workspace/all-target Clippy pass. All 67 UI tests pass, including a new scoped Review-state regression that rejects document/stale replies and preserves a published success after cancellation. Local log: `build/verification/global-notes-export-ui-tests.log`.

Four native synthetic captures at `build/rust-visual/global-notes-export` verify dark global JSON, light Spanish global Markdown, compact text zoom and unchanged reader JSON export. The three published artifacts contain respectively 80, 80 and 181 annotations. Global JSON includes two nested documents. Executable, Cargo lockfile, UI source, PNG, control and export-file hashes were checked against the manifest. Executable SHA-256: `FAEA6DB2DF5C36B8DF9F51528C08E611070B3CB574AF20A0215E90B88DB27564`.

Captures supply an isolated synthetic destination instead of interacting with the save dialog. They execute the real request, worker, snapshot, formatter and safe publication path. Actual native selector interaction, compact keyboard/scrolling, Narrator, monitor DPI, concurrent mutation stress and process interruption acceptance remain pending. The full rewrite and Release gates remain open.
