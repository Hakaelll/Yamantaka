# Native rule simulation checkpoint

Date: 2026-10-31. Scope: matching previews for saved local rules. This checkpoint does not establish complete automation parity or release acceptance.

## Connected behavior

Tools → Saved rules offers Simulate for supported definitions. A dedicated worker opens an independent read-only SQLite connection, verifies the saved rule revision, and retains a read transaction across the scan. Library changes committed after the snapshot starts do not change later pages. The UI receives progress without running SQL or file hashing on its thread. Only an active scan requests periodic repainting.

The scan reads ordered batches of up to 80 documents and retains governed match records. Conditions support extension, effective author, source folder, tag, collection, stored missing-file state, exact duplicate, and imported-document context. Manual simulation has no import batch and therefore returns no imported-document matches. Import-triggered execution remains pending.

Source-folder matching normalizes separators and case and respects directory boundaries. This deliberately excludes similarly named sibling folders that the Flutter prefix check could include. Tag and collection conditions resolve the original ID first, falling back to a normalized name only when that ID does not exist. Author overrides take precedence.

Exact duplicates require fresh full SHA-256 reads. Each verified source must match its recorded size, modification time, and legacy fingerprint. The old fingerprint alone cannot establish equality. Results retain the observed source revision and full digest. Unreadable, missing, or changed sources are excluded and reported as unverified. A completed preview is an observation, not permission to apply actions later; execution must revalidate the reviewed data and source proofs.

Cancel simulation discards incomplete matches. Scans also discard partial results when they reach 100,000 records or 30 seconds. Checks run between records and file chunks; blocking operating-system reads are not made interruptible by this deadline. Match, page, bound, and hash-buffer allocations use the memory governor. Native SQLite and operating-system allocations are outside this accounting.

Closing the native window during a pending simulation requests cancellation and defers closure until the worker reply. Leaving the manager cancels its request and discards results belonging to that request. Confirmed action execution, execution cancellation, frozen reviewed plans, journals, recovery, and undo remain required work.

## Executed evidence

The retained application test `file_comparison::tests::background_comparison_rejects_partial_hash_collision_and_confirms_full_copy` now also drives the real simulation worker and read-only storage adapter. Its focused offline run passed, with no new standalone tests. It checks all eight condition kinds, folder boundaries, case handling, author overrides, ID priority and name fallback, matching order, source proofs, stale rule rejection, pre-start cancellation, missing-source reporting, memory release, and original file bytes. A WAL snapshot remains coherent after another connection changes a title, missing flag, and rule. An observer's SQLite data version remains unchanged across every simulation, detecting committed writes from another connection.

Three source files share the same legacy partial fingerprint and size. Two have identical full content; the third differs after the first 64 KiB. Both the application check and native duplicate scenario return only the first two.

The binary built offline. `cargo fmt --all --check` and binary Clippy with warnings denied passed. Four native scenarios completed:

- `rule-simulation-extension`: ten synthetic books matched, Spanish dark theme.
- `rule-simulation-duplicates`: three scanned, two matches, no unverified sources, English light theme.
- `rule-simulation-cancel`: an actual scan of a synthetic library with 100,010 documents is cancelled after progress begins; no partial match list is retained.
- `rule-simulation-small-200`: ten matches, Spanish light theme, reduced motion, requested 640 × 480 window, 200% text scale. The sequence scrolls to Close preview and verifies its control bounds.

The [capture manifest](../screenshots/rust-rule-simulation/capture-manifest.json) records executable, source, image, and control hashes. Native sequences invoke controller actions programmatically through the real runtime and isolated SQLite stores. These captures do not prove physical pointer/keyboard behavior, Narrator, monitor DPI, multi-monitor handling, performance budgets, or full Flutter migration. No Computer Use, personal library, broad workspace suite, or release packaging was used for this checkpoint.

See the [saved-rule guide](../user-guide/saved-rules.md) for current user-facing behavior. Full confirmed application, restart recovery, undo, import integration, and final rewrite acceptance remain pending.
