# Pause unnecessary text extraction after selection failure

SelectionState previously kept requesting subsequent selected pages after a required text entry failed. Such work could not complete the selection until the failed page was retried. The admission step now pauses the selected-range tail while preserving visible-page requests, the two-pending-request limit and all successful cached text.

The decision checks only errors inside the active selected range. An error on another visible page does not block an unrelated selection. Existing Retry selection removes failed cached entries; the range becomes eligible again on the next request step. Already submitted native work is not cancelled by this change. Normal PDF raster rendering is independent and source files remain unchanged.

## Executed evidence

A demo-and-capture-only verification calls the actual private SelectionState admission path with an isolated logical 100,000-page selection. It checks zero further tail requests after an error, admission of a visible page, resumption after retry, exclusion of an unrelated error from the blocking decision, and zero admission when both request slots are occupied. This is a state-machine check, not extraction from a real 100,000-page document or a performance benchmark. No standalone Rust test was added.

The same native run opens a real synthetic two-page PDF and completes the existing keyboard-selection preview. Its control capture requires ready text geometry and a positive tracked text lease. Workspace Clippy across all targets, native debug build and formatting passed. [Status report](artifacts/selection-admission.json) · [Reader capture](../screenshots/rust-selection-admission/selection.png).

The source review also confirms that PdfPageText and SelectedText already own shared-governor leases. The capture's memory-scope text now describes those buffers accurately while excluding egui text layout, UI/font textures and opaque driver allocations. This does not add accounting for those excluded framework resources, establish process-wide memory use or prove frame-time targets.

No visual flow was redesigned. Actual PDFium failure injection, prolonged pressure/retry loops, multi-pane failure stress, physical keyboard/Narrator/DPI acceptance and complete resource traces remain pending. A failed selected page still needs recovery before copying the full passage; successful partial text is not silently substituted.
