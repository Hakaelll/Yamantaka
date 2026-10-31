# Recover unsaved page notes

Date: 2026-10-31. Scope: new page-note editors in the primary and reference readers.

## Persistence and restoration

The default continuity snapshot now includes page-note drafts in the same atomic replacement as reading positions and comparison settings. It stores document identity, the catalogued fingerprint, pane role, page, rotation, color, raw note text and raw tag input. Drafts remain separate from committed annotations. Input is not trimmed or treated as a valid annotation before the user saves it.

Version 3 adds `page_notes`. Versions 1 and 2 remain readable; sessions without drafts retain their existing version until they first need version 3. Once upgraded, subsequent saves retain version 3. Workspace embedded reading contexts keep version 1 and reject default-session draft fields. Unsupported records and invalid rotations preserve the existing snapshot. The total four MiB bound remains unchanged; the draft contract permits at most 24 distinct document/pane pairs and limits each note and tag field to 64 KiB.

After a reader opens, a retained draft reopens its Notes editor only when the document identity, catalogued fingerprint and page range match. A missing reference or disabled session restoration retains the draft on disk. It can be recovered when the matching reader later becomes available. Explicit document removal removes its draft context. Saving or cancelling a restored editor removes that live draft from subsequent snapshots.

The existing background continuity writer captures edits after its idle interval. Graphics recovery also captures the latest editor state during checked shutdown. After the accepted-write barrier, shutdown consumes annotation replies before taking that snapshot. A successful note creation therefore remains a committed annotation instead of reopening as a duplicate draft. A failed creation retains its editor input.

## Evidence

Offline workspace Clippy for all targets with warnings denied passed. The retained historical-schema compatibility test passed with version-three round trips, exact Unicode/whitespace/tag preservation, duplicate-editor rejection, document-removal reconciliation and non-destructive rejection of invalid rotations. No standalone Rust test was added. The native application build and focused UI Clippy passed after adding the recovery scenarios.

Two native synthetic profiles destroy the wgpu device after opening primary and reference page-note drafts. The fresh Glow process must restore every saved draft field. In the second profile, a primary note save is submitted immediately before the fault, without consuming its reply in the normal UI loop. Recovery must expose exactly one committed note and no primary draft, while retaining the reference draft. The [capture manifest](../screenshots/rust-page-note-recovery/capture-manifest.json) records the captured build and source hashes.

The restart verifier also clones the synthetic two-draft store with SQLite's backup API. Ordinary restart restores both editors; marking the reference missing restores the primary editor and retains the reference draft; disabling restoration leaves both drafts retained without opening a reader. The verifier checks the post-exit JSON and confirms that the source fixture's database, preferences and continuity bytes are unchanged. See the [restart report](../screenshots/rust-page-note-recovery/continuity-restart-report.json) and sibling controls files. Restart PNGs remain local artifacts.

## Limits

Selection-based annotation drafts, edits to existing annotations, metadata drafts and other editors do not yet have persistent recovery. Physical device loss, forced termination during the idle interval or atomic replacement, failed storage writes and full revision-change interaction remain pending. The catalogued fingerprint check does not prove fresh whole-file SHA-256 equivalence. These fixtures do not certify full graphics-failure recovery or whole-rewrite acceptance.
