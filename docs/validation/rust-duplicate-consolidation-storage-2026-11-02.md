# Guarded duplicate consolidation storage

## Scope

This checkpoint implements the storage operation and real file-proof guards. The current **Identical copies** interface still discovers, reviews and opens groups. Confirmation, editor/reader retirement, live session flushing, continuity remapping, manager refresh and the merge/undo interface remain pending. No merge is started by discovery or by opening this view.

The domain defines a selection from a verified full-hash group, per-field metadata choices, an immutable review revision, affected-table counts and a backup receipt. An independent store prepares and applies the operation. The application supplies a source guard that rehashes both selected files, checks their exact native source revisions and holds read handles through commit. Under Windows those handles deny writes and deletion.

## Preservation rules

The kept record retains its ID, path and source identity. Resolved title and author values become explicit manual corrections. Empty fields can be filled from the other copy; different nonempty titles, authors and library notes appear as conflicts. Favorites are combined, maximum progress and its page are retained, and the latest stored open date wins.

Bookmarks retain their IDs when they can move; duplicate pages collapse to the kept bookmark and fill an absent label. The kept reader preferences win when present, otherwise all stored preference columns transfer, including cover and initial-side policy. Reading-session IDs and every history field remain unchanged except the owning document. Historical NULL end dates are preserved. Annotation IDs, geometry, review state, text and tag associations survive, with original owner/fingerprint provenance recorded.

Tags, collections and revision memberships are combined without duplicate keys. Workspace tabs move with their IDs and queue context; collisions keep the existing tab and repair an active incoming tab reference. Equivalent relations collapse and direct relations between the two copies disappear as self-relations. Remaining relation IDs and anchors are retained. Revision-group current-document pointers follow the kept record.

Manual cover selection wins over automatic detection. Thumbnail paths are cleared so the chosen page is rendered for the kept record. Topic memberships are combined and manual topics remain manual. Missing enrichment and reading goals are filled from the incoming record; the kept values otherwise take precedence. Unknown goal fields and existing session-context metadata are preserved. Search text and index state are invalidated, and an explicit indexing exclusion on either record remains excluded on the kept record.

## Transaction and recovery

Review hashes typed values and row boundaries from affected document, preference, note, cover, annotation, membership, workspace, relation, revision, search-state and relevant metadata rows. Preparation and pre-apply checks use coherent read transactions. A changed catalogue source or dependent record requires another review.

A unique complete v9 JSON backup is synced and published before the write transaction starts. It includes the existing manual-cover and preference extensions. The backup is then held read-only and hashed. Apply rechecks the review revision inside a write transaction, performs the transfer, validates SQLite and foreign keys, checks the live source guards, and commits the merge-history row with the data changes. Failure or cancellation before commit rolls back SQL and retains any completed backup. Replaying a completed plan fails because its incoming record has been removed.

The receipt identifies the kept and incoming records, merge history, backup path and full backup SHA-256. The backup can be prepared with the existing reviewed JSON restoration path. This is a whole-library JSON restore, not a selective undo of later changes. The merge/undo interface has not yet been connected.

## Executed verification

The retained application test now runs a populated synthetic store through production discovery, source freezing, review, pre-backup, merge and backup preparation. It checks metadata choices, progress/favorites, bookmark collision, full preference/session/annotation rows, annotation tags, manual covers and the resulting render plan, topics, goals including unknown fields, workspace active-tab repair, relation collapse, revision pointers, FTS invalidation and preservation of indexing opt-out.

It rejects a changed dependent note and a changed file tail even when size, partial fingerprint and native modification date are unchanged. A Windows write attempt fails while the source guard is live. An injected SQL failure leaves the entire exported state unchanged. The pre-merge backup restores the exact previous exported categories and settings. Replay is rejected, unrelated organization/rule/journal records remain equal, and all tracked memory is released. No standalone test was added.

Formatting, retained application library tests, all-target workspace Clippy with warnings denied and the offline native build were executed. This is storage and source-proof evidence, not a live GUI consolidation test or process-kill acceptance.

## Remaining acceptance

The next integration must retire affected live readers and writers, retain or resolve unsaved editors, flush live sessions/preferences, reconcile continuity and in-memory histories, and accept a committed write receipt even if cancellation arrives after commit. Historical unfinished sessions must not be guessed to be live. A confirmation view must expose conflicts and collision precedence, preserve the recovery path and make whole-library undo consequences clear.

Physical interaction, forced-process interruption, broader malformed data and recovery races, large-catalogue timing, full memory accounting and final Release acceptance remain pending. PDFs are not embedded in the JSON backup and original PDF contents are never written by the consolidation operation.

## Connected UI follow-up

[Native consolidation UI](rust-duplicate-consolidation-ui-2026-11-02.md) records the later confirmation flow, atomic native journal, continuity recovery and immediate backup review. The earlier evidence above retains its original storage-only scope.
