# Recover edits to existing notes

Date: 2026-10-31. Scope: unsaved edits to existing annotations in the primary and reference readers.

## Behavior

The default-session snapshot now retains each edited annotation's identity, pane, frozen revision, source fingerprint, page, rotation, original passage and raw edited kind, color, review state, note and tags. Draft input remains separate from committed annotation rows. No save is performed just because a draft was recovered.

Recovery reads the exact document/annotation pair on the repository worker, independently of the current 80-row note list. It verifies the frozen revision and source fields before handing the governed record to the UI. Matching sources reopen the normal editor with the draft input. Subsequent saves still use the transactional revision guard, including changes made after recovery.

Changed or unavailable annotations open a separate recovered-edit panel. It keeps the draft and original passage without attaching them to a new source. Copy draft copies the text and edited labels; Retry recovery makes a new scoped read; Discard draft explicitly removes this recovery record. The panel never overwrites or recreates an annotation automatically. Missing reference readers and disabled session restoration retain their drafts for a later matching reading.

Version 5 adds `annotation_edits`. Versions 1 through 4 remain readable, and named workspace contexts keep their separate version-one format. A snapshot upgrades only when needed and does not downgrade afterward. The total four MiB bound remains. Drafts are limited to 24 distinct document/annotation/pane identities; raw note and tag fields are bounded to 64 KiB each, and the original passage to one MiB. Invalid revision shapes, duplicate identities and unsupported records preserve the existing file. Explicit document removal clears related drafts and cancels their pending restoration.

Archive or editor ownership blocks accidental replacement by another note command. Cancelling, discarding or successfully saving a recovered edit removes that live draft from subsequent continuity snapshots. The accepted-write shutdown barrier consumes annotation replies before capturing drafts, so an edit committed just before graphics failure does not reopen as unsaved.

## Executed evidence

Offline workspace Clippy for all targets with warnings denied, formatting and the native build passed. The retained compatibility test covers version-five round trips, exact Unicode and whitespace preservation, unknown root extensions, duplicate rejection, document-removal cleanup and non-destructive rejection of malformed revision arrays. The retained annotation-history helper checks exact document-scoped editor lookup. No standalone Rust test was added.

Two native synthetic graphics-failure profiles restore existing-note edits in both panes. Their frozen fields must match after restarting through Glow. The second submits the primary edit immediately before destroying the wgpu device; the committed annotation must retain its identity and new text without reopening a duplicate draft. The reference edit remains unsaved. These native profiles edit existing page-note records; nonempty passage and geometric-mark recovery still need broader native evidence. See the [capture manifest](../screenshots/rust-stored-note-recovery/capture-manifest.json).

The restart verifier uses independent SQLite backup copies for normal startup, a missing reference, a changed annotation and disabled restoration. It verifies editor/archive ownership and exact post-exit draft persistence. The changed case updates the note directly without changing its timestamp; the newer database text must survive while the older draft appears separately. The source fixture's database, preferences and continuity bytes remain unchanged. See the [restart report](../screenshots/rust-stored-note-recovery/continuity-restart-report.json) and sibling controls. Restart PNGs remain local artifacts.

## Remaining acceptance

Selection-based drafts for new annotations still lack persistent recovery. Physical interaction with Copy, Retry and Discard, original-file replacement during recovery, large inherited records, repeated close/restore cycles and full resource/performance acceptance remain pending. Recovery input strings and shared-handle overhead still need complete governor accounting. The source check uses the catalogue fingerprint and the frozen annotation revision, not a fresh whole-file hash. This checkpoint does not certify whole-rewrite or Release acceptance.
