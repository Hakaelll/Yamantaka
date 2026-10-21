# Native Flutter import verification

Date: 2026-10-21. Scope: the connected Settings import assistant and next-launch store selection. The rewrite and complete migration acceptance remain in progress.

## Connected behavior

Settings → Storage opens an import dialog with native database and preference-file pickers. Inspection runs on an owned background worker and returns the frozen source summary. Explicit confirmation creates an independent UUID directory under the Rust root's `imports` directory through the [reviewed WAL capture contract](rust-migration-review-2026-10-21.md). Source changes invalidate the review. Cancellation retains the current store and does not install a partial copy.

After copying, Use next launch verifies the capture again and atomically writes a versioned selection file. It does not replace the running session's database or preferences. Close and reopen loads the selected database and preferences. Use original is available both before restarting and from Storage after opening an imported store. Neither choice deletes the other store. Imported libraries remain editable after activation; subsequent launches validate the store normally rather than treating ordinary reading progress as receipt corruption.

Selection accepts canonical UUID directory names only. Missing selection means the original Rust store. Invalid versions, traversal, missing stores, and reparse points fail closed. Only a genuine NotFound permits fallback; access errors and dangling selection links do not silently select another library. Native single-instance admission precedes database opening and recovery.

The worker coalesces progress notifications, supports cancellation, joins its thread, and defers application shutdown until its terminal result. Dialog shortcuts do not activate library deletion or reading commands behind the review. The original PDF files are not migration write targets.

## Evidence

The retained migration test now also checks independent managed copies, cancelled activation, next-launch selection, post-activation edits, inherited preferences, unchanged original records, return to an original store without a preference file, and malformed selection refusal. The suite remains at 55 essential tests; no standalone test case was added. All 55 passed. Formatting, offline workspace Clippy across all targets with warnings denied, and the native build passed.

Eight native synthetic scenarios passed through `tool/capture_rust_ui.ps1`: source files, source review, copied result in Spanish/light/reduced-motion mode, selected next launch, return to original, changed-source rejection, cancellation during copying, and a blocked-source error at 640 × 480 with 200% application text. The last case uses an injected closed-process decision; it does not prove detection of a running real Flutter process. Required recovery actions and horizontal control bounds passed. Screenshots were inspected, including the compact error and next-launch result. The final copy edit only clarifies the pending selection in Storage.

See the [synthetic capture manifest](../screenshots/rust-migration-ui/capture-manifest.json), [review screenshot](../screenshots/rust-migration-ui/migration-review.png), and [compact recovery screenshot](../screenshots/rust-migration-ui/migration-blocked-small-200.png). These captures contain generated study documents, not a personal library.

`tool/check_migration_startup.ps1` additionally copies the synthetic selected store into an isolated portable directory and launches the real executable without `--demo`. The imported startup loads 10 books. The original database and preference hashes remain unchanged. Selecting the original and launching again loads its single sentinel book. Both processes exit successfully. These production-path captures remain local under ignored build output because their capture records are correctly marked non-demo; they are not published as synthetic UI screenshots.

Run the startup check after generating the migration captures:

```powershell
./tool/check_migration_startup.ps1 -Executable build/verification/yamantaka-migration-ui.exe -Python <python-executable>
```

The script checks the executable hash and the owned synthetic source directory, refuses links in its input, uses a new portable directory, and removes only its own copied executable after completion. It retains small diagnostic artifacts locally.

## Remaining acceptance

Real installed and portable Flutter libraries, historical executable exports, renamed/concurrent Flutter process behavior, process-kill recovery, quarantine and rollback, native picker interaction, Narrator, physical DPI, and multiple monitors still require acceptance. This checkpoint does not import personal data, prove complete backup/restore UI parity, or authorize a replacement release.
