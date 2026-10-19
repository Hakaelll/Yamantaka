# Bulk automatic covers in the native library

Date: 2026-10-20. Scope: connect the guarded [regeneration coordinator](rust-bulk-cover-runtime-2026-10-20.md) to native library selection, preserve visible images during replacement, and recover deferred uploads when governed capacity changes. This checkpoint does not establish full rewrite or release acceptance.

## User flow and ownership

Select books on the current catalogue page, open the selection menu, and choose **Regenerate covers…**. A named confirmation captures those identifiers and explains that current manual covers will be replaced without modifying PDF files. Cancel receives initial focus. Compact windows show the complete consequence and both actions; extended explanation remains available as hover text. Background library, reader-selection and global shortcuts do not run while this review owns focus.

Confirmation submits the existing bounded actor action. The selection menu shows completed/total cover outcomes and offers **Cancel remaining**, which cancels the request and wakes a parked actor. The terminal reply preserves saved/failed/unattempted counts and selects visible failures and remaining entries for another attempt. Organization writes and cover batches share the existing write reservation. Individual manual-cover decisions remain available and retain the runtime's later-choice protection.

Before submitting, the UI cancels ordinary cover consumers for the fixed selection and pauses new consumers for those identifiers, including books which become visible later. It retains existing registered textures. Successful progress resumes each completed identifier once; the terminal reply resumes every still-paused identifier even when progress was coalesced or admission failed. A resumed visible slot requests its stored choice through the ordinary cover path with a fresh generation and cancellation scope. Replacement updates its texture and source page together after successful upload. Stale ordinary results cannot replace the image.

When governed upload admission rejects a replacement, the slot retains its old texture and one tracked CPU image. It retries when the memory governor's capacity epoch changes, without scheduling an idle repaint loop. Removing a visible slot, choosing a new manual cover or starting another batch drops obsolete deferred pixels. Existing presenter retirement/fence handling continues to account for the old GPU payload until its final use completes. This evidence covers the native wgpu presenter; it does not close the broader alternate-compositor resource gate.

## Executed evidence

- Workspace formatting, all-target offline Clippy with warnings denied, and the offline native build passed.
- The retained `page_presenter::tests::atomic_upload_admission_includes_live_cpu_and_row_padding` case passed. No standalone test was added; the retained Rust source count remains 55 cases.
- The preceding runtime check passed the existing `library_bulk::tests::page_selection_and_bulk_operations_keep_fixed_targets_and_partial_results` case with real PDFium, later manual-choice conflicts, protected-reader waits, responsive queries, cancellation, original source bytes and released PDF buffers.
- Four final native capture scenarios passed. The review captures preserve three selected manual covers in dark English and light Spanish at 640×480 with text scale 2. The harness requires fully visible Cancel and Regenerate controls, and the final PNGs show the complete manual-reset consequence.
- The applied preview starts with stored manual page 1 for three synthetic books, submits the same UI command adapter, and waits for the ordinary cover-loading path to replace all three with automatic page 0 and new native texture identities. The selection clears and all paused identifiers are released.
- The pressure preview caps governed GPU capacity at its existing usage before submitting. All three replacement uploads defer while their old texture identities and source pages remain intact. Restoring the previous limit permits all three uploads; no deferred payload or paused identifier remains at capture.
- Final PNGs were inspected. The native manifest matches the frozen executable, Cargo lock and every captured UI source. Published images contain synthetic provenance; no personal PDFs, databases or executables are included.

Evidence: [native capture manifest](../screenshots/rust-bulk-covers/capture-manifest.json), with adjacent PNGs and control JSON. Reproduce with `tool/capture_rust_ui.ps1` and cases `bulk-covers-review`, `bulk-covers-review-small-200`, `bulk-covers-applied`, and `bulk-covers-pressure-es` after building the pinned runtime and synthetic PDF fixtures.

The previews drive shared review/submission/refresh adapters and assert native outcomes. They do not certify physical pointer or keyboard sequences. The GPU scenario changes the application's memory limit; it does not simulate driver exhaustion or device loss. End-to-end UI cancellation/fault stress, physical DPI/Narrator/multi-monitor checks, large-corpus latency, full workspace tests and the broader migration/parity/release gates remain open.
