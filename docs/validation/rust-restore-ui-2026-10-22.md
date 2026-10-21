# Native backup restoration

Checkpoint: 2026-10-22. Settings → Storage now connects backup selection, review, confirmation, background preparation, and installation after shutdown.

Choose Restore JSON backup, review the file version and record counts, then choose Restore and close. Cancel receives initial focus. Source changes invalidate the review through a SHA-256 check of the exact bytes that produced it. Preparation runs on an owned cancellable worker. The modal blocks background library shortcuts and activation handling. Cancelling preparation retains the current library.

Confirmation hands an isolated prepared database to the main process. Installation starts after the application and its joining workers have dropped. The main process retains the single-instance guard through installation, preventing a second Rust process from opening the library during the directory swap. The previous store remains in a separate safety directory. The completion message tells the user to reopen Yamantaka and identifies that directory. PDFs are not write targets.

## Recovery contract

The v2 restore journal hashes SQLite, preferences, and the restore receipt. Recovery rejects modified files and nonempty unverified WAL files. It rolls back an interrupted first rename, or validates an installed replacement while retaining the safety copy. Known staging files are checked before removal; cleanup tolerates already removed files so an interrupted cleanup can resume. Unexpected files and reparse points are retained and rejected. Journal reads are bounded to 64 KiB.

Startup recovers the root and selected managed store before requiring the selected directory to exist. Restored managed stores use their restore receipt rather than requiring a Flutter migration receipt. Legacy v1 journals can recover a missing active directory from its safety copy. If both legacy copies exist, the old journal cannot prove preference preservation; startup retains both and requires explicit recovery instead of silently accepting either one. User-facing recovery controls for that ambiguous case remain pending.

## Evidence and limits

The storage changes passed all 55 retained workspace tests before the UI connection. Existing backup cases now exercise source changes, cancellation while copying/validating/before installation, changed preferences and receipts, WAL refusal, partial cleanup retries, and managed-store recovery. No standalone test cases were added. Two owned child processes were forcibly terminated at the actual rename boundaries; recovery preserved the original database and preference hashes or the valid replacement plus its unchanged safety copy.

The production portable startup check loaded a selected synthetic library with 10 books, recovered that selected store after a simulated interrupted rename and loaded the same 10 books, then returned to the original one-book store. Original root database and preference hashes remained unchanged. `tool/check_migration_startup.ps1` reproduces that sequence.

After connecting the UI, focused Clippy and the native build passed. One [synthetic review capture](../screenshots/rust-restore-ui/restore-review.png) was inspected. A single native apply smoke prepared the backup, closed the application, installed 10 books, and retained the previous safety directory. It used the same worker, handover, shutdown, and installation code as the confirmation button. The actual Windows file picker and completion message were not driven by that smoke. The [capture manifest](../screenshots/rust-restore-ui/capture-manifest.json) records the executable and source hashes. Local smoke results remain under ignored build output.

No repeated visual matrix or additional broad suite was run after the UI connection. Physical DPI, Narrator, real historical executable exports, manual rollback selection, ambiguous-journal recovery controls, quarantine, and full memory/performance acceptance remain pending. This checkpoint does not establish complete rewrite or release readiness.
