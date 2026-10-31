# Confirmed native rule execution checkpoint

Date: 2026-10-31. Scope: applying a reviewed native plan, persistent receipts, cancellation/failure rollback, history, and guarded undo. Full automation parity and Release acceptance remain open.

## Connected behavior

Tools → Saved rules → Simulate → Review actions → Confirm execution applies all six supported action kinds on an independent worker. The saved-rule revision, destination revisions, and each document's complete v11 row signature are checked before their relevant writes. Exact-duplicate actions reread the full source and compare its source revision and SHA-256 immediately before its document transaction. The current source remains open against concurrent Windows writes through that transaction. Original PDFs are never written.

Tag/collection membership is added only when absent. Repeated actions share a new destination identity and do not duplicate memberships. Workspace and reading-queue actions append suspended entries with the original page and preferred reading mode, preserve existing entries, and enforce the inherited 1,000-entry limit. An empty active selection chooses the first added entry. Queue changes advance workspace revision timestamps. Finished state sets the last stored page, progress one, and the opening time; pending resets page and progress. Favorite uses the interpreted reviewed value.

Each document change commits in the same short SQLite transaction as its undo receipt and root progress update. Destination creation and its receipt are atomic as well. Native versioned root/item records use the existing v11 operation_journal table; no new principal tables or user_version changes are introduced. Two recreatable indexes support bounded history queries and reverse receipt lookup. This avoids repeatedly scanning previously reverted receipts and does not rewrite an ever-growing undo JSON array for every document.

Cancel operation and application failure stop further writes and attempt to revert the committed prefix. The application limit is checked between documents, and duplicate hashing checks it between chunks. Rollback can take longer than that application limit. Native window closure requests cancellation and waits for acknowledgement. The UI continues rendering while the worker runs; idle execution does not request continuous repainting. This architecture is not a measured latency or memory-RSS acceptance result.

## Undo and recovery

The collapsible Execution history reads up to 80 native root records at a time. Review undo retains a root revision, and Confirm undo verifies a SHA-256 snapshot of the root row in addition to its timestamp. It rejects a changed journal even if its timestamp was left unchanged.

Undo proceeds in reverse receipt order. It checks the catalogued source identity, current progress/favorite values, and signatures of added membership/queue rows before reverting each document. Each document reversal and its receipt status commit together. Existing memberships are untouched. A new destination is deleted only when its definition still matches and it has no remaining document/annotation/workspace use. Workspace revision timestamps advance; a removed active queue entry clears only its own selection.

Conflicts stop that attempt and keep the remaining receipts recoverable. Other changes to that document transaction roll back together on conflict. A failed or interrupted record with remaining receipts retains rollback capability. Undo cancellation keeps the already-reverted prefix and exposes the remaining work for review. Processed counts include documents later reverted and do not count changes still present.

Reopening the store can list a leftover running record as interrupted and recover its committed receipts. No in-memory execution plan is automatically reapplied. Receipts are versioned native payloads; inherited Flutter journal rows remain preserved and are not interpreted by this executor.

Controlled plan, per-document receipt, receipt parsing, result, and history allocations use the shared memory governor. Native SQLite and driver allocations remain outside that ledger. Error codes persisted in roots are fixed codes rather than source paths or SQL exception details.

## Executed evidence

The retained application test `file_comparison::tests::background_comparison_rejects_partial_hash_collision_and_confirms_full_copy` now also drives the real execution worker and independent SQLite store through the existing review chain. The focused offline run passed. It covers complete application and undo, an actual favorite change and restoration, queue reading mode and active-selection preservation, source-byte preservation through the enclosing chain, cancellation after one committed document, a stale reviewed document, a SQL failure after a committed prefix, subsequent reading-progress conflict preservation and retry, catalogued-source conflict, root changes without timestamp updates, stale undo revisions, persisted interrupted-record recovery through a reopened database connection, and SQLite integrity. No standalone tests were added.

The final binary built offline. Formatting and binary Clippy passed with warnings denied. Native scenarios cover:

- `rule-execution-completed`: ten synthetic documents and seven declared actions spanning all six kinds. The refreshed catalogue has finished progress, one completed root is visible, and Review undo remains available. Spanish dark theme.
- `rule-execution-undone`: the same execution is undone through the normal runtime and committed journal. The refreshed catalogue regains unfinished progress and the root is no longer reversible. English light theme.
- `rule-execution-undone-small-200`: the undone record remains readable at a requested 640 × 480 window, 200% text scale, reduced motion, and Spanish light theme.
- `rule-execution-failed`: a synthetic SQL trigger rejects the second queue insertion. The first committed document and created destinations are reverted, and the root reports failure without remaining rollback work.

Native sequences invoke controllers programmatically using the real runtime and isolated file-backed stores. The [capture manifest](../screenshots/rust-rule-execution/capture-manifest.json) records executable, source, Cargo.lock, image, and control hashes. Captures contain synthetic data only. No Computer Use or personal library was used, and no broad workspace suite was run.

## Remaining acceptance

The reopened-connection recovery check seeds an interrupted status; it is not a killed-process or power-loss test. Physical close/cancel, termination at commit boundaries, undo interruption, large histories/queues, low-budget recovery UI, active-reader/workspace interactions, FTS interaction coverage, imported Flutter journal recovery, and import-triggered rule context remain pending. Undo uses current-value/signature postconditions; identical-value edits or removal/recreation that return to the same inherited row representation are not distinguished by an independent edit-history generation. Receipt tampering is not authenticated cryptographically. Paired performance measurements, Narrator/DPI, and final Release packaging remain outstanding.

This checkpoint connects the native workflow. It does not certify all recovery, compatibility, performance, accessibility, or whole-rewrite requirements.
