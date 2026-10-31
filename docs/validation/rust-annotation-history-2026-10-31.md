# Annotation history checkpoint

Date: 2026-10-31. Scope: transactional annotation undo/redo, focused reader controls, configurable shortcuts and command-palette actions. This checkpoint does not establish full rewrite parity or release acceptance.

## Connected behavior

The storage actor retains before/after snapshots of affected annotation rows and tag IDs. Creation, editing, deletion and same-document reader batches enter history only after their write, FTS refresh and result query commit successfully. Multi-page creation shares one entry. Unknown stored extension columns, raw geometry JSON, timestamps, origin fields and tag identities survive restoration. Undo does not delete shared tag definitions.

Undo and Redo verify the current affected rows and links against the expected snapshot inside their SQLite transaction. A later conflicting change, missing tag, cancellation, SQL failure or failed result read rolls the transaction back. The stack changes only after commit. A new edit clears that document's redo branch. Histories are document-scoped and temporary, with at most 100 entries per document and an 8 MiB combined retained-history budget. Snapshot leases participate in the existing global memory governor. Older entries expire first; oversized admission fails before saving. Attachment checks and cross-document annotation batches clear affected histories.

Notes exposes Undo/Redo with enabled states from the latest actor reply. Ctrl+Z and Ctrl+Shift+Z target the focused document canvas. The configurable registry now has 30 commands and accepts the inherited undoAnnotation/redoAnnotation identifiers. Text fields retain Ctrl+Z/Ctrl+Y editing combinations even if a global command is reassigned to them. Unsaved note editors block annotation history. The command palette exposes the same handlers and disables unavailable history directions.

Successful replies refresh both readers' note lists and invalidate visible annotation marks through the existing write-completion path. History never modifies the source PDF. Pending writes retain their existing ownership when a pane closes.

## Executed evidence

The retained storage compatibility test `overrides_win_and_queries_are_bounded_and_escape_wildcards` passed with the added annotation-history helper: one passed, five filtered, 5.37 seconds. No standalone test was added. Its history checks cover:

- Create/undo/redo with stable identity, tag restoration and FTS result changes.
- Edit/undo/redo and delete/undo, including inherited origin data, a BLOB extension and byte-exact noncanonical geometry JSON.
- An injected SQLite failure, cancellation and an external edit that Undo refuses to overwrite.
- Redo invalidation after a new edit, 105 edits retaining exactly 100 steps, and release of all governed snapshot memory when the database closes.

Offline native build and binary Clippy with warnings denied passed. The native `reader-annotation-history` scenario completed actual egui F6, Ctrl+Z and Ctrl+Shift+Z events. It reversed a creation, deletion and edit, replayed them, then branched with a new note. Main notes stayed at their original two records and main reading stayed on page 1. Reference ended with one new note, four undo steps and zero redo steps. The scenario also checks that focused text fields keep their undo events, including a reassigned global action.

The first capture reached the terminal application state but the script checked nonexistent summary fields. After correcting those fields, its visible-control check incorrectly required Redo to be enabled after branching. The final script requires enabled visible Undo and disabled Redo, and the complete scenario passed. The final Notes capture was visually inspected. No Computer Use was used.

The [capture manifest](../screenshots/rust-annotation-history/capture-manifest.json) records synthetic source, lockfile, executable and artifact hashes. Publication checks the current source and frozen binary against it and embeds image provenance. No personal library or PDF is included.

## Remaining acceptance

The native scenario exercises an independent reference document, not every same-document pane-close timing or every multi-page pointer sequence. Global review batches intentionally invalidate affected histories, as their inherited controller did not add those batches to reader undo. History-budget pressure, all inherited extension schemas, physical keyboard layouts, Narrator, monitor DPI and complete reader performance acceptance remain unproven. A workspace-wide suite was not repeated for this unit. The broader migration, trust/recovery and Windows release gates remain pending.
