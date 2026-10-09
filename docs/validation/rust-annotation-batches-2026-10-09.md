# Atomic multi-page annotation preparation

The domain, repository and application actor now support a governed batch of annotation drafts for a passage spanning several pages. The current reader editor still submits one-page drafts; its multi-page connection remains the next integration step.

Batch preparation requires every selected page in canonical order. It rejects missing, repeated or reordered snapshots and invalid character boundaries. Each nonempty page segment retains its own selected Unicode text, context, geometry, page index and rotation. Whitespace-only segments and an empty exclusive end-page boundary produce no annotation. An entirely empty passage is rejected.

Each draft applies the existing geometry policy independently. Reliable segments retain their requested kind; unreliable segments propose a page comment with the selected text and no rectangles. The batch exposes whether any segment needs page-note confirmation. The eventual editor must present that fallback across the affected pages before saving. The same entered note and tags apply to each created record.

The draft array and each owned draft use Text-budget leases. Array capacity is reserved before allocation. Failure after preparing an earlier page releases all partial drafts, and a completed batch can outlive the source text snapshots. The actor retains it through Arc and borrows it into the repository operation.

Single-page and batch creation share one insertion path and transaction context. Batch records, tag joins, consented FTS updates and preparation of the returned page all commit together. A failure on a later page, unconfirmed fallback, cancelled request or denied response reservation rolls back the whole operation. Records have individual IDs and a shared creation timestamp. Returned queries retain their 80-row pagination; that response limit does not truncate the batch being saved.

## Verification

Three new storage integration tests cover ordered Unicode segments, per-page geometry fallback, invalid source order and character boundaries, an empty exclusive end-page boundary, draft-array admission and memory release. They also check rollback after a trigger rejects the second insertion, rollback of created tags/search entries, successful shared timestamps and search results for both pages, cancellation and denied response memory after writes.

A fourth test submits the owned batch through the application actor using actual SQLite storage. It checks both page records and selected text, tags, request routing, independent Arc lifetime and release without native PDF calls.

Formatting, warning-denied workspace/all-target Clippy and workspace tests pass. The local full log is `build/verification/annotation-batches-tests.log`. This checkpoint has no presentation changes or new native captures.

## Remaining integration

The editor still needs to freeze every selected page, preview the complete passage, submit the batch and present affected-page fallback confirmation. Multi-page pointer/autoscroll acceptance, native batch creation, annotation history and complete paint coverage remain pending, together with tag/global-review/export interfaces and the other release gates in the parity matrix.
