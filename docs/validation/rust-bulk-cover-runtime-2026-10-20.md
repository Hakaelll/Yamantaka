# Bulk cover regeneration runtime

Date: 2026-10-20. Scope: actor/native coordination for automatic cover regeneration. At this runtime checkpoint the UI did not invoke this action. Selection-menu wiring, explicit confirmation and live pixel replacement were pending; this checkpoint is not a completed user-facing bulk-cover feature or a replacement release.

Follow-up: [the native UI checkpoint](rust-bulk-cover-ui-2026-10-20.md) connects the action and verifies retained-image replacement, including governed GPU-capacity recovery. The evidence below describes the preceding runtime checkpoint.

## Implemented contract

`Action::RegenerateCovers` accepts at most 80 catalogue identifiers and a request cancellation scope. It shares the existing bulk identifier validation/deduplication. Admission captures every available document source and exact stored cover revision before any native work starts. Unavailable entries become per-item failures. These SQL-only snapshots execute within one actor command, so another application command cannot change a later item's baseline during capture.

One existing governed `CoverJob` runs at a time. Its new snapshot constructor uses the captured revision at admission and completion, rather than recapturing a newer manual choice when that queued book eventually starts. A later choice therefore wins both while waiting for native capacity and while detection is in flight. Persistence uses the established automatic-cover compare-and-replace adapter after successful detection. Existing source checks and cache writing remain in that path; the coordinator does not delete old choices or cached images before success.

The actor integrates native ticket readiness, capacity revisions, indexer slot yielding and waiting-cover counts. A parked job sleeps until a native result, capacity change or command/control wake. Ordinary metadata queries continue while two reader handles are protected. A catalogue SQL bulk operation and a cover-regeneration batch cannot own the bulk reservation concurrently.

Progress snapshots contain only identifiers/count outcomes and the request scope, not bitmap collections. Successful detected pixels are released after the existing persistence/cache path completes. Terminal replies use the shared scoped bulk outcome with saved, failed and unattempted identifiers. Cancellation preserves completed writes, stops subsequent admissions and waits for an active native ticket to reach its terminal state. Shutdown drops/cancels the active job before the engine is released.

The connected Flutter bulk action restores automatic selection, including a previous manual choice. The pending UI must therefore require an explicit explanation/confirmation for that reset, retain the current displayed image until replacement succeeds, and preserve decisions made after the batch starts.

## Executed evidence

Prerequisites are the pinned PDFium binary and generated fixtures documented in the main README. The native regression uses `textual.pdf`, `automatic-cover.pdf` and `front-matter-logo.pdf` from the ignored fixture directory.

- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` passed.
- `cargo build -p yamantaka --offline` passed.
- `cargo test -p yamantaka-application --lib library_bulk::tests::page_selection_and_bulk_operations_keep_fixed_targets_and_partial_results --offline -- --exact` ran one passing retained case. No standalone case was added; the source count remains 55.
- Real PDFium kept two protected readers readable through native text requests while a third-source cover request waited. The job was not ready at the unchanged capacity revision, and the actual runtime returned a normal three-document library query while its batch remained parked.
- A manual decision changed while waiting kept its exact decision snapshot unchanged; another manual decision installed after native admission also survived completion. Both conflicted resets were reported as failures, while another document received a detected cover.
- A cancelled two-book batch retained its first completed reset and reported the second ID as unattempted. Cancellation before the first native admission made no write. Duplicate IDs were processed once and missing entries were reported separately.
- The runtime published a finished scoped cancellation reply and progress snapshot. After runtime/worker shutdown, tracked PDF-buffer usage was zero. Original textual fixture bytes matched their saved pre-run bytes. Manual document title and reading progress were preserved.

The existing shared bulk case still executes its SQL/tag/collection/archive checks before this native helper. UI source and layout did not change, so no new screenshot batch was generated for this runtime-only checkpoint. The full workspace test suite was not rerun.

## Remaining work

Connect the selection-menu action and confirmation to this runtime, consume scoped progress, cancel correctly from the UI, and replace library pixels while retaining old images until new uploads succeed. Verify failed/cancelled refresh recovery, GPU pressure, reader/import interleavings and user interaction with real native captures. Live source-change invalidation and broader performance/physical acceptance remain open in the full rewrite matrix.
