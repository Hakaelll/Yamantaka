# Native backup export verification

Date: 2026-10-21. Scope: native JSON export from Settings → Storage. Restore confirmation, installation, and recovery controls remain pending.

## Connected behavior

Storage contains a compact Backups section with Export JSON backup. The native Windows save dialog chooses a JSON destination. An owned background thread opens the actual current SQLite path through a read-only connection and invokes the [streaming v9 exporter](rust-backup-streaming-2026-10-21.md). The preferences snapshot comes from the current application state. The library and PDF worker remain available while export runs.

The section reports progress and exposes Cancel backup. A successful result displays the filename and Show in folder; failures retain a retry action. Closing the application requests cancellation and defers closure until the worker returns. The worker owns and joins its thread. Notifications are coalesced to at most one progress update per 100 ms, plus the terminal notification.

Destination validation requires a JSON extension and an existing parent directory. It resolves that directory before checking that it lies outside the protected Rust store root, including its managed imports. Existing non-file and reparse-point destinations are rejected. Storage still writes through a synced temporary file and atomically replaces the chosen destination only on success. This protects internal library preferences from the export chooser; it does not claim resistance to arbitrary concurrent filesystem replacement by another program.

The backup source comes from the actual SQLite connection before the runtime takes ownership. In-memory demo modes disable export and explain the limitation. Interactive desktop previews use an isolated file-backed synthetic store. No guessed preview filename can select stale data.

## Evidence

Four native synthetic captures passed through `tool/capture_rust_ui.ps1`:

- Saved v9 backup, then isolated preview validation of all category hashes and 10 books.
- Cancellation in Spanish/light/reduced-motion mode, with byte-identical preservation of an existing output file.
- Refusal of a destination inside the Rust library folder.
- Saved result at 640 × 480 with application text at 200%; the export action remains visible and all controls respect their horizontal clips.

The [capture manifest](../screenshots/rust-backup-ui/capture-manifest.json) records the executable, source, and image hashes. See the [saved result](../screenshots/rust-backup-ui/backup-saved.png), [cancellation](../screenshots/rust-backup-ui/backup-cancelled-es.png), and [enlarged-text result](../screenshots/rust-backup-ui/backup-saved-small-200.png). Two visual inspection passes corrected compact layout and competing capture-only cache scrolling. The actual save picker and Explorer were not driven by these synthetic scenarios.

All five retained backup tests passed. The existing file-backed helper now opens the export source read-only, verifies that writes fail, and checks the reported path. The in-memory case reports no file path. Existing historical round trips, WAL concurrency, cancellation, file-size limits, and interrupted restoration checks remain in that suite. No standalone test case was added. Formatting, offline workspace Clippy across all targets with warnings denied, and the native build passed. Focused independent review found no production issue and confirmed the corrected in-memory demo handling.

## Remaining acceptance

Restore preview and confirmation UI, safe next-launch installation, recovery and rollback controls, real historical executable exports, close-during-export physical interaction, native picker interaction, Narrator, physical DPI, complete memory accounting, and paired performance measurements remain pending. The previous storage restore contracts are not evidence of a complete user-facing restore flow. No release readiness or measured latency improvement is claimed here.
