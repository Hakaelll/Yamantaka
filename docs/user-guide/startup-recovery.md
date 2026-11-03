# Recover a library that cannot open

If Yamantaka cannot read the selected Rust library or its saved preferences, it opens a recovery window before starting the PDF engine. Unsupported preferences versions also use this path. Saved preferences are validated before opening the database; Yamantaka does not silently replace unreadable settings with defaults.

Choose **Choose JSON backup…** to select an existing Yamantaka backup. Review the file name and book, annotation and bookmark counts. Reviewing a copy does not install it. You can choose another file or select **Exit without restoring**.

Choose **Restore and close** to confirm. Yamantaka revalidates the reviewed backup, closes the recovery window and installs a staged library. The entire old Rust store remains in a separate `pre-restore-<identity>` folder, including unreadable preferences or a damaged database. The completion notice identifies that safety folder. Reopen Yamantaka to use the restored library. Original PDF files are not changed.

Closing the window during review or preparation cancels that work. Once confirmed preparation hands over to installation, the existing transactional restore journal and rollback rules apply. If installation fails, Yamantaka reports the failure rather than claiming success. Keep the old store, backup and restore journal for recovery.

The recovery window uses Spanish by default because saved language settings may be unreadable. Launch with `--en` for English. `--software` selects the existing Glow renderer if the GPU cannot run the default DX12 path. These options do not repair a damaged store.

If automatic recovery cannot confirm an interrupted library operation, the recovery window instead offers **Review current** and **Previous copy…**. Review the chosen library's book, annotation and bookmark counts. **Confirm and close** keeps that choice: the current library stays in place, or the previous folder becomes active and the complete current folder becomes a new safety copy. The pending operation record is preserved under a unique `yamantaka-recovery-preserved-<identity>.json` name beside the library. Its contents do not determine the choice. Reopen Yamantaka after confirmation.

Leaving this window without confirmation does not resolve the operation. A changed library or journal requires another review. Validation reads independent database and WAL copies; it does not create SQLite sidecars in the original folders. A complete, valid WAL is included in the reviewed counts. Damaged, truncated or stale WAL frames stop this review rather than silently presenting an older main-file view. Retain those files for separate recovery.

This path requires an existing current folder and exactly one pending operation record no larger than 64 KiB. Multiple pending operations, missing folders, unsupported historical schemas and damaged imported-store selection records still need separate recovery. The independent Flutter import supports historical databases. For an unavailable startup marker, see [safe startup and record recovery](safe-startup.md). Without a usable recovery source, exit and retain the files.

See the [interrupted-operation checkpoint](../validation/rust-pending-operation-recovery-2026-11-03.md) for the exercised preservation and WAL cases.
