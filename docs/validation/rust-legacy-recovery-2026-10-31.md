# Flutter receipt recovery checkpoint

Date: 2026-10-31. Scope: reviewed recovery of the five persisted Flutter automation undo kinds, and undo of that recovery. This is a partial rewrite checkpoint, not release acceptance.

## Behavior and safeguards

Tools → Saved rules → Execution history now offers Review recovery for inherited journals with undo capability. An independent read-only worker prepares a governed, immutable proposal. It parses the original JSON in insertion order and reverses its receipt sequence, following the connected Flutter rollback behavior. The supported kinds are removeTag, removeCollection, removeQueue, restoreFavorite, and restoreProgress. Explicit unchanged receipts are skipped. Unknown fields or kinds, duplicate keys, invalid progress/mode/theme values, stale revisions, and insufficient memory reject preparation. The original JSON is never rewritten. Missing documents are reviewed as no-ops and are not recreated.

Old receipts lack complete postconditions. The review therefore displays current and proposed values and requires explicit confirmation. It cannot establish that an old action caused the current value, or recover actions that were never persisted. Current document, membership, destination, and journal revisions are checked before writes. Queue removal preserves unrelated entries, repairs active/split selection, and retains unknown split properties.

Recovery uses a new native root with compatibility marker yamantaka-legacy-recovery-v1. Every document mutation, full inverse receipt, and root progress update share one short transaction. Removed rows retain every SQL column and value, including queue notes, zoom, pins, and unknown compatible columns. Final completion and the inherited journal's recovered state commit together. Original PDF files are not written.

Cancellation or failure attempts to undo the committed prefix. A native journal survives process interruption and offers reviewed undo after reopening. Undo checks postconditions, restores removed rows without replacing existing rows, restores captured reading/favorite values and queue selection, and finally restores the inherited journal's capability. Conflicting later edits remain untouched and leave remaining receipts recoverable. Workspace comparisons exclude the revision timestamp while retaining all other header fields and the complete ordered queue.

Reading progress now follows the existing Flutter calculation: zero for zero pages, one for one page, otherwise pageIndex / (pageCount - 1), with clamping. The library's Continue reading shortcut prefers the active visible document, including its first page at zero percent.

## Executed evidence

The existing application verification chain was extended; no standalone test was added. The retained file_comparison::tests::background_comparison_rejects_partial_hash_collision_and_confirms_full_copy test passed: one passed, three filtered, 19.60 seconds. Its receipt-recovery branch uses independent file-backed v11 stores and actual plan/execution workers. It covers all five receipt kinds, unchanged receipts, full row and split restoration, stale document rejection, failure after an earlier commit, preservation of conflicting later queue edits, retry, budget admission/release, malformed and unknown payload preservation, reading progress boundaries, and SQLite integrity.

The same retained test verifies cancellation after an observed commit and launches an owned child test process against an isolated 1,000-document fixture. After observing a durable running root with committed work, the parent kills and joins only that child. Reopening finds an interrupted recovery; native undo restores the committed prefix and leaves the inherited journal available for review. This proves that exercised process-interruption path, not power-loss durability or every possible commit boundary.

Offline native build and binary Clippy with warnings denied passed. Five native capture cases passed their durable-state assertions and control-bound checks:

| Case | Observed result |
| --- | --- |
| inherited-recovery-review-small-200 | Two documents, three destinations, one skipped receipt; original history unchanged; confirmation and close visible at 640 × 480 logical size and 200% text |
| inherited-recovery-completed | New reversible native recovery, two documents processed, inherited journal recovered |
| inherited-recovery-undone | Recovery rolled back and inherited capability restored |
| inherited-recovery-unknown | Review rejected; original three history records preserved |
| inherited-recovery-failed | Injected SQL failure after an earlier commit; prefix reversed and original inherited capability retained |

The small review and completed-history PNGs were inspected. Captures use isolated synthetic stores, English/Spanish and light/dark variants. Their source, Cargo.lock, executable, and artifact hashes were checked before publication. Published PNGs include provenance metadata. See [capture manifest](../screenshots/rust-legacy-recovery/capture-manifest.json).

## Remaining acceptance

Unknown historical receipt formats remain preserved and unavailable for automatic recovery. Linked-rule retries still start a fresh reviewed simulation rather than replaying an inherited offset. Import-triggered rule execution and full original-feature parity remain pending. These checks do not establish physical monitor DPI, Narrator, real installed Flutter migration, whole-library performance targets, power-loss durability, or Windows release packaging. Native receipt postcondition hashes do not authenticate a database against deliberate tampering or distinguish identical-value ABA changes.
