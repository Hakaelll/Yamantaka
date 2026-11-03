# Visible reader source ownership

`Action::Open` previously allowed a document already committed as visible to
replace its native handle. The old pane could then submit requests with its old
page geometry against the newly opened source. Worker epochs protect queued
requests, but cannot identify which pane originated a later request for the same
document ID.

The application actor now rejects that replacement with `ReaderAlreadyOpen`
after catalogue/removal validation and before native admission. Cancellation retains its
`Cancelled` result. Closing the reading releases its owner and permits a fresh
open. Selecting a document's existing tab still reuses that reading. To load an
external edit, close Compare if it displays that PDF, close the PDF's reading
tabs, and open it again. Returning to the library alone does not close a reading.

## Executed checks

The [result artifact](artifacts/reader-source-ownership.json) records the
synthetic before/after probes. Before the guard, opening either an unchanged or
changed visible source succeeded. Both are now rejected.

`rust/application/examples/reader_source_handoff.rs` uses real PDFium and an
isolated SQLite store with two visible readers. Only private temporary copies of
generated fixtures are edited. The corrected probe verifies that rejection
preserves the original native epoch and the exact cached raster allocation. An
external edit makes old-source text reads fail with `PdfSourceChanged`. After
closing that reader, a new open returns the replacement text. The second reader
remains usable throughout. Both readers close successfully. An ownership
rejection is excluded from document quarantine.

All eight retained application import/resumption tests passed. They include
protected import, cached encrypted covers, cover rebuilding, protected indexing,
cancellation and two-reader preemption. Their temporary directory was placed
inside `build/verification/native-test-temp`; the initial run in the host's
temporary directory encountered Windows access denial while persisting a cache
image. The existing protected-cover assertion now expects the typed
`PdfSourceChanged` result introduced by the source-coherence correction.

Workspace all-target Clippy with warnings denied and workspace formatting passed.
Three serial native captures completed with code 0. The hidden reader and the
revealed comparison match the preceding control checkpoint byte for byte after
RGBA conversion. All 42 generated source PDF hashes remain unchanged. Raw
captures stay local under `build/verification/reader-ownership`.

This checkpoint verifies actor admission and native close/reopen behavior. It
does not add automatic reload, certify every UI close/draft interaction or replace
the remaining physical accessibility, filesystem and Release performance gates.

## Complete retained workspace check

A subsequent workspace run exposed an admission-order regression: reopening a
visible document during removal returned `ReaderAlreadyOpen` before the existing
removal rejection. Catalogue and removal validation now run first. The visible
owner remains protected before any native opening request.

The corrected locked, offline workspace run passed all 56 retained tests, with
zero failures or ignored tests. Workspace formatting, all-target Clippy with
warnings denied and the native two-reader ownership probe also passed. No Rust
test functions were added. The [workspace artifact](artifacts/workspace-checkpoint-2026-11-03.json)
records the source and raw-log hashes. These checks do not establish comparative
performance, physical accessibility or final replacement-release readiness.
