# Reviewed rollback to a previous library

## Connected behavior

**Settings → Storage → Previous library** opens a native folder picker beside the active store. **Previous copy** provides the same choice in startup recovery. Selection accepts a regular sibling `pre-restore-UUID` safety folder, with a valid current-schema library and Rust preferences. Counts are reviewed before confirmation; cancellation is focused first.

The review fingerprints the complete directory tree, including unknown ordinary files and empty directories. It checks file contents, sizes and modification times, rejects reparse points, and bounds traversal to 32 levels, 200,000 entries and 32 MiB of entry names. SQLite validation uses a temporary main-file copy so that read-only validation cannot create WAL sidecars in the safety folder. Nonempty source WAL is rejected, rather than ignored. Other instances must be closed before using the copy.

Preparation rechecks the reviewed fingerprint on an owned worker. After the active application and database readers close, installation verifies the source again, saves a small atomic rollback journal, moves the current store into a fresh safety folder, and moves the selected safety folder to the active location. A final fingerprint check precedes journal removal. No selected-folder path remains after success; the old current library stays in its new safety folder. This preserves the whole store rather than reconstructing only database and preference files.

Startup resolves the rollback journal before opening a library. An interruption after saving the original returns that complete store. An interruption after installing a matching reviewed tree keeps the installed tree and its safety copy. Ambiguous combinations are retained and automatic recovery stops. JSON restoration refuses a pending rollback journal, and rollback refuses a pending JSON restore journal.

The shared review worker and installation handover now use typed JSON/rollback variants. The ordinary close path marks startup clean before exchanging stores, avoiding a later write through the replaced store path. Returning to an unsaved editor clears a queued restore handover as well as its pending-close flags. Compact review keeps the preservation explanation outside its scroller. Startup recovery uses two compact action rows and scrolls the context, keeping its failure reason visible at 200 percent text.

## Executed evidence

- The retained storage test `replacement_keeps_complete_safety_store_and_cancel_does_not_write` passes. Its existing JSON cases remain, and the rollback helper verifies reviewed counts, stale-tree rejection, pre-commit cancellation, whole-store installation, reversal through the new safety folder, unknown binary files, empty directories, continuity-file preservation and rejection of pending WAL.
- Two actual child-process terminations exercise rollback after the original directory moves and after the selected directory moves. Startup recovery is idempotent, and both complete libraries remain present with their expected book records.
- Workspace all-target Clippy with warnings denied passed. Subsequent UI changes passed focused UI Clippy and a fresh offline native build. Formatting and diff checks passed.
- [Four native synthetic captures](../screenshots/rust-store-rollback/capture-manifest.json) show Spanish regular and 640×480/200-percent rollback reviews, the existing JSON review and compact startup recovery. Action clips and the recovery context's reserved space are checked. The images were visually inspected; an initial recovery footer that displaced the context was revised.
- A separate native run starts with `--reader --rollback-apply`, completes the shared handover, and exits successfully. [Its result](artifacts/store-rollback-handover-2026-11-02.json) records ten installed books and the safety folder. Before writing that result, the executable asserts preservation of the selected source's sentinel in the active store and the old current store's sentinel in its safety folder.

The synthetic apply route invokes the same prepare and install commands as confirmation. It is not a physical folder-picker or keyboard test, and requesting the reader does not certify a particular native page was visible before closure. No personal library was restored.

## Remaining acceptance

Real installed libraries, physical picker/keyboard/Narrator/DPI checks, large-store timing, directory-sharing races, journal tampering, power-loss durability and the full unsaved-draft close matrix remain pending. Ambiguous journal resolution still requires inspection; this adds no automatic choice between changed folders. Source stores with pending WAL must be closed and checkpointed first. Main schema v11 safety copies are supported; historical Flutter schemas continue through the independent import path. Full parity, migration acceptance, resource accounting, paired performance and the Windows Release remain open.
