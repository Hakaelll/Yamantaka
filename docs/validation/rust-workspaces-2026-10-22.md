# Saved workspace checkpoint

Tools → Workspaces now connects named reading snapshots to the existing v11 SQLite tables. It lists imported Flutter spaces and supports creation, replacement from the current readings, renaming, duplication, setting the default flag, confirmed deletion and opening. Lists use pages of 80. Names are trimmed and limited to 80 UTF-16 units. Duplicate documents within a saved workspace are rejected with an explanation because the inherited schema allows one entry per document.

Writes run on the existing application actor in SQLite transactions. Revision guards reject stale operations. Creation, replacement and duplication assign unique stored tab IDs and remap the active reading and split-pane secondary reference. Other workspace columns and unknown split-object properties remain stored. A namespaced, versioned context inside split JSON retains native per-reading preferences and queue state without changing schema v11 or backup v9. Native context is used only while its inherited reader rows still agree; edits from an older client therefore do not revive stale context.

Opening uses fresh live reading IDs, existing PDF admission and deferred retirement of outgoing documents. It preserves the current reader on admission or native opening failure. Empty saved sets use the existing last-reader close path and retire remaining handles afterward. Closing the manager cancels an unaccepted load. Keyboard shortcuts and queued external activations stay blocked while the manager is open.

Validation was confined to this feature:

- One retained compatibility test now checks duplicate/update split-reference remapping, preservation of imported view metadata and rejection of a stale write. It passed; no additional standalone test or broad test run was added.
- One native synthetic scenario exercises create, replace, rename, duplicate, default flag, delete and actual PDF opening through the application actor. Assertions check queue notes, active reading, rotation, reading theme and margins. A separate read-only connection reopens the file-backed database and checks the final saved set. The same scenario produced the final screenshot after the reference fix.
- The Windows development build passed. The published [capture](../screenshots/rust-workspaces/workspaces.png) contains synthetic documents only.

This is partial workspace parity. Applying saved library filters, focus and split/comparison state, automatic default opening, active-workspace queue persistence, deletion undo and physical interaction acceptance remain pending. The current manager saves and explicitly restores reading snapshots. It does not establish complete restart or advanced workspace acceptance.
