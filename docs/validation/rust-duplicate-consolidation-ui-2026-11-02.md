# Native duplicate consolidation

The Identical copies view now connects reviewed consolidation to the guarded storage operation. It preserves both source PDFs and combines catalogue data only after explicit confirmation.

## Connected behavior

The user chooses the kept record and reviews one incoming copy. The modal exposes title, author and library-note conflicts, affected record counts and overlap precedence. Its footer remains visible at 640 × 480 with text at 200%; fields scroll independently. Cancel receives initial focus. Original records remain separate when the review is dismissed.

Preflight requires both affected readings to be closed, pending changes and sessions to settle, and affected recovery drafts to be resolved. Confirmation rejects competing background work. The actor pauses normal command admission and indexing during the transaction, closes affected native PDF handles through asynchronous tickets, and runs hashing, backup and consolidation on a joined worker. Closing the application waits for cancellation or a committed result. A receipt arriving after cancellation still reconciles the actual committed state.

The transaction writes merge history and its native operation journal together. Continuity loading resolves retired identities only through matching native completed history and journal records. Open and recently closed contexts retain tab identity, page and zoom. Ambiguous or malformed native recovery evidence fails closed; unrelated drafts retain their earlier behavior. Affected drafts are not silently reassigned.

The UI refreshes catalogue results, selection and covers after completion. Immediate Undo opens the exact pre-merge backup with its recorded SHA-256 through the existing restore workflow. This is a whole-library restore, with explicit review and application shutdown, rather than selective record undo.

## Executed evidence

- All four retained application tests passed. The existing duplicate verification includes changed-source rejection, post-review record changes, injected transaction failure, backup round trip, native journal identity, continuity alias recovery and malformed-journal rejection.
- Workspace Clippy, all targets, passed with warnings denied. The native debug executable built successfully and formatting passed.
- Five native synthetic cases passed capture assertions: reviewed choices, compact enlarged-text review, completed merge, dismissed review and Undo backup review.
- A separate read-only check inspected the actual completed capture catalogue, committed journal, backup bytes and original PDFs: two remaining records, selected incoming title persisted, kept source identity unchanged, original PDF hashes unchanged and valid foreign keys. [Status report](artifacts/duplicate-consolidation-native.json).
- [Review](../screenshots/rust-duplicate-consolidation/duplicates-merge-review.png), [compact](../screenshots/rust-duplicate-consolidation/duplicates-merge-small.png), [completed](../screenshots/rust-duplicate-consolidation/duplicates-merge-completed.png), [cancelled before application](../screenshots/rust-duplicate-consolidation/duplicates-merge-cancelled.png), [Undo review](../screenshots/rust-duplicate-consolidation/duplicates-merge-undo.png).

The native manifest describes original capture bytes. Published PNGs add synthetic provenance without changing pixels; companion origin files record both hashes. The earlier [read-only discovery checkpoint](rust-duplicate-discovery-2026-11-02.md) provides the prior UI state.

## Remaining acceptance

The Undo capture verifies review of the three-record backup, not installation through the native confirmation button. Dedicated durable merge-history browsing after restart, native forced termination during merge, physical keyboard/Narrator/DPI interaction and large-library performance acceptance remain pending. The cancelled capture dismisses review before application; it does not prove mid-transaction cancellation. Storage failure and cancellation checks have narrower scope than native interaction acceptance. No final rewrite or Release acceptance is claimed.

## Durable-history follow-up

[Consolidation history](rust-duplicate-history-2026-11-02.md) connects later paged native history and hash-checked backup review. The evidence above describes its original session-only checkpoint.

## Installation and restart follow-up

[Native backup installation](rust-duplicate-restore-2026-11-02.md) now records the later actual restore handover, table comparison, PDF hashes and separate native process restart. Earlier scope and remaining physical interaction requirements above retain their original meaning.
