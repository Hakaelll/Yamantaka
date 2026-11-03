# Reviewed recovery of the saved library selection

Startup previously stopped before its recovery window when `selected-library.json` was unreadable, unsupported or pointed to an unavailable managed store. A typed recovery target now distinguishes this failure from an interrupted library operation. The existing independent recovery window can review the initial native store or a selected imported capture before opening the application database or PDF engine.

Confirmation preserves the exact old selector and publishes a version-one selector for the explicit choice. Both libraries remain in place. Review uses independent SQLite/WAL inspection and frozen file trees, plus the selector's Windows identity/version and SHA-256. A changed record or chosen library rejects the write. Original-store review omits its reserved `imports` subtree, so unrelated captured libraries are neither scanned nor treated as changes to the chosen original store.

The confirmed publication writes and flushes a replacement record and a separate version-one intent before renaming the old pointer. Startup completes this intent before resolving selection. All ordinary selection reads and changes reject a pending intent, including the interval when the active selector is absent. Validated child filenames and complete hashes bind preservation and publication to the intended files. Intent cleanup deletes the exact held Windows file object. No database, preference or original-PDF write is part of selection repair.

## Executed evidence

- A native storage probe verifies stale selector and chosen-library rejection, early cancellation with exact selector readback, rejection of a foreign candidate and preservation of a 65,537-byte oversized selector. Choosing an imported capture preserves the exact old selector and all library-input hashes. Repeated startup recovery is idempotent.
- Changing an unselected imported resource does not invalidate review of the original store.
- Three actual child-process terminations occur after intent publication, after old-pointer preservation and after new-pointer installation. Before resuming, ordinary selection is rejected in every pending state. Startup then recovers the confirmed imported choice, preserves the old selector bytes and retains all database, preference, receipt and unknown-resource hashes.
- Three native UI cases exercise an unsupported selector with original-store confirmation, a valid selector pointing to a missing store with imported-store confirmation, and a malformed selector left without confirmation. Counts are two books in the original and three in the independent imported store. Both confirmed cases verify the next startup choice; cancellation leaves the selector and library inputs unchanged.
- English regular and Spanish 640×480/200% images have fully visible actions with scrolling context. The images were visually inspected. Existing JSON and pending-operation recovery are also executed as regression cases.
- Offline build, workspace formatting and all-target workspace Clippy with warnings denied pass. No standalone Rust test cases were added.

The [result artifact](artifacts/library-selection-recovery.json) records the executable hash and selected outcomes. Synthetic stores and captures remain local under `build/verification/selection-repair` and `build/demo-store/captures`.

Reproduce the storage and process-interruption assertions with `yamantaka-rust.exe --demo --verify-selection-repair=<report.json>`. UI fixtures use `--demo --selection-recovery=original|imported|cancel --capture=<image.png>`; add `--es --text-scale=2` for the compact profile. Child interruption flags require the synthetic fixture marker and do not use the installed library.

## Remaining acceptance

Physical folder-picker, keyboard, Narrator and hardware-DPI checks, power-loss durability, concurrent external writers, very large stores and combined recovery failures remain pending. The path requires an existing selector within 64 KiB and a valid candidate. Missing selectors without a pending intent retain the existing initial-store behavior. Invalid recovery intents, absent candidates and unsupported native schemas require separate recovery. A crash before intent publication may retain an unreferenced staged selector; startup never adopts that file automatically. Final replacement-release readiness remains open.
