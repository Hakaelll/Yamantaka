# Explicit cover-cache rebuilding

## Connected behavior

Details → More actions → Regenerate cover starts a dedicated application job. Choose cover, title/author editing and Explorer also use that bounded menu, keeping the inspector's primary row readable. Missing files and safe mode disable the rebuild action.

The job captures the catalogue source and cover revision, then acquires the existing background full-file hash/read guard before opening a private native owner. It checks the catalogue fingerprint and a recorded full digest when one exists. That guard stays alive through source-revision checks, capture, confirmed closure and final validation. The selected-page operation validates the native source before and after rendering and requires successful local cache persistence. It never falls back to opening another PDF handle without authorization.

Existing cover-page decisions use a single selected-page render. Automatic detection remains available for a decision that needs refreshing. Before completion the job rechecks catalogue/cover state and verifies cached dimensions and pixels against the native capture. Selected-page rebuilding does not rewrite the decision or catalogue metadata. Automatic detection uses the existing guarded cover-completion contract.

The shared one-shot password broker now identifies import, metadata analysis and cover rebuilding separately. Dialog action labels match the operation. Passwords remain owned zeroizing values without serialization or cloning. The old import-only broker API was migrated and removed. A cancelled challenge wakes the job and native cleanup completes before its result. A completed cache write can remain after cancellation.

The actor admits one explicit rebuild at a time. Two admitted readings produce a recoverable capacity result with a close-and-retry action rather than starting a third private owner. Existing renderer priorities and the two-hot-document ceiling remain in place. Full-file hashing uses the existing owned worker and governed buffer.

## Executed evidence

Offline workspace Clippy for all targets with warnings denied, the focused retained real-PDFium test and the native application build passed. Later UI-only menu edits passed focused UI Clippy and rebuilds. No standalone Rust test case was added.

The retained import helper now also removes the disk image cache and regenerates a manually selected second page of an existing protected PDF. It retries an incorrect credential, compares rebuilt dimensions and every pixel with the corresponding unencrypted reference page, and verifies the original cover revision, book count, manual title/author overrides, opening/source timestamps and an existing annotation payload. Separate cases cancel while waiting, reject a changed cover selection and report capacity with two pinned readers; both pinned readers remain usable. The surrounding helper still checks unchanged source bytes.

Two synthetic native captures passed: the inspector action menu with the complete rebuild control inside its clip/viewport bounds, and the protected-PDF rebuild dialog at 640×480 with 200% application text. The latter checks operation-specific actions and bottom clearance. Both were visually inspected. Source, executable and lockfile hashes match the [capture manifest](../screenshots/rust-cover-rebuild/capture-manifest.json). The earlier direct inspector row truncated the new action; the final menu replaces that layout.

The backend cases invoke the same typed request used by the inspector button. The dialog capture exercises the real password challenge but does not submit a physical keystroke or prove a complete application-restart matrix.

## Remaining acceptance

Full-text indexing of protected PDFs, physical keyboard/Narrator/DPI interaction, large and damaged PDF corpora, process-kill recovery, repeated ownership races, injected disk failures and complete memory/performance acceptance remain pending. Older libraries without a recorded full digest still use their inherited partial fingerprint for catalogue comparison. Check file can establish their full-content baseline. This checkpoint does not complete the rewrite or authorize a replacement Release.
