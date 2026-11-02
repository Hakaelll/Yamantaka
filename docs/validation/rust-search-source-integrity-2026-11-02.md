# Full-source identity for resumable indexing

## Connected behavior

Indexing now computes the complete SHA-256 before native admission. It reuses the existing owned hash worker and its governed 64 KiB buffer. The actor observes completion through its blocking select. The hash phase does not reserve a native PDF slot; a verified source waits if reader/import admission has priority.

The read guard stays alive across native opening, password prompts, text extraction and closure. Native source revision replies must match the guarded file. The guard is checked before page persistence and completion, and the repository rejects late writes if the catalogue source record changes. Shutdown transfers the guard to the close ticket so it survives until that reply is consumed.

The index repository now requires a captured file snapshot and complete digest. Its auxiliary `index_sources` row binds the source key and digest to the exact committed index revision. A resumed cursor is accepted only when that proof matches. Older or cleaned caches restart that document at page zero; a differing catalogue full-hash baseline is rejected before opening, without replacing earlier index pages.

The existing independent file-verification cache migrates transactionally from v1 to v2, preserving file hashes. The main library schema remains v11. One proof row is retained per indexed document. Proofs contain identifiers, hashes and revisions, without passwords, extracted text or raw paths. Clearing the index deletes the proofs while preserving exclusions. Existing document-removal cleanup removes proofs from an already opened verification cache.

Proof writes follow the main index commit. A crash or failure in that gap leaves a revision mismatch, which forces a restart. It cannot authorize a cursor from a different committed revision. Main and auxiliary stores do not pretend to share a transaction.

## Executed evidence

The existing retained native preemption test now includes these cases, without adding a standalone test:

- A v1 auxiliary database containing a sentinel file hash migrates to v2 and preserves that hash.
- A valid page cursor survives closing and reopening the database. A superseded session cannot append a page.
- Equal legacy identity with a differing complete digest restarts at zero. Missing and stale revision proofs also restart at zero.
- An injected auxiliary insert failure occurs after the main page commit. A subsequent start rejects the unproven cursor and clears its cached page before reindexing.
- A catalogue path change rejects both a late page and a stale admission snapshot, preserving the previous page text.
- Index clearing removes auxiliary cursor proofs.
- Real protected PDF indexing denies source writes during password prompts. A shutdown close ticket is allowed to become ready without consuming it; writes remain denied until the ready ticket is consumed, then succeed.
- A separate PDF copy is changed beyond its first 64 KiB while keeping its size, modification time and legacy fingerprint. Actual full-file hashing rejects it against the existing full baseline before native opening or a password prompt. Previously indexed text remains intact.
- Existing password retry, cancellation, owner-loss, reader-slot yield, exact SDK text comparison and runtime FTS lookup cases continue to pass.

The compatibility run also exposed an older annotation fixture that called a snapshot-owning read inside a manually opened savepoint. The fixture now changes and restores its temporary title between normal read calls, preserving the check that catalogue titles do not affect annotation revisions. Production annotation code was unchanged.

The focused real-PDFium test, retained catalogue/metadata compatibility test, workspace all-target Clippy with warnings denied, offline native build and formatting checks were executed. This change does not modify UI composition; the earlier unlock captures remain evidence for their own recorded executable and source hashes.

## Remaining acceptance

Forced process termination in the commit/proof gap was modeled with a failing proof write, not executed as a process-kill matrix. Repeated ownership races, shutdown admission failures, corrupt auxiliary stores, large PDF corpora, physical accessibility checks, full memory accounting and paired Flutter/Rust performance measurements remain pending. Legacy catalogue entries without a trusted full digest still cannot prove historical byte identity. This checkpoint does not complete the rewrite or authorize a Windows Release.
