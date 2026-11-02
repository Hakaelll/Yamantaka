# Imported-document rule context checkpoint

Date: 2026-10-31. Scope: passing a complete current-session import context into read-only rule simulation, frozen action review, confirmed execution, and undo. This is a rewrite checkpoint, not release acceptance.

## Baseline and implementation

Flutter's automationMatches checks importedDocumentIds for the documentImported trigger. Its connected app-shell manager supplies tag, collection, and duplicate context but leaves importedDocumentIds at its empty default. Rust's earlier simulator therefore always returned no matches for that trigger. This checkpoint connects it to actual native imports while retaining explicit review and confirmation.

The import actor now collects successful import return IDs in an ordered, deduplicated ImportedDocuments map. Every retained identity has a shared-governor memory lease. The context is bounded to 100,000 identities. Admission failure discards the complete context; it does not expose a partial subset or undo successful PDF imports. Existing registered paths and previously checkpointed records skipped by the import loop are not added. Reanalysis uses a separate path and does not publish an import batch.

A successful import update carries an immutable Arc context to the matching UI request. Stale replies cannot replace it. Starting a new import clears the previous available context. Failed/cancelled runs publish no completed batch. The context is session-local; closing the app or choosing Forget batch releases it when no reviewed preview still owns it. Resuming an import collects records committed during that resumed run, not earlier checkpointed records.

Tools offers Saved rules: batch available after a nonempty import. The manager shows the count. Simulate on a Document imported rule captures that batch; other trigger kinds retain their existing full-library semantics. The worker verifies the saved rule and opens the read snapshot, then performs indexed document-ID lookups only for the batch. Missing documents are skipped. Without a batch, it returns an empty result without traversing the catalogue. All existing cancellation, deadline, revision, memory, review and execution guards remain connected. Action review also checks that each imported match belongs to its retained batch.

A completed preview identifies its own immutable batch. Forgetting or replacing the currently available batch does not retarget an existing preview. Confirmed execution uses the existing versioned per-document receipts, rollback, history, and undo. No PDF file is modified and no action runs without confirmation.

## Executed evidence

Two existing checks were extended; no standalone test was added:

- The retained application verification chain passed: one passed, three filtered, 16.42 seconds. The import branch checks deduplication, two real matching IDs plus a missing ID, deterministic scope, zero matches without context, frozen action review and complete memory release. It also retains the existing non-import condition and full-SHA duplicate coverage.
- native_reader_preemption_resumes_the_interrupted_page_and_keeps_prior_evidence passed: one passed, seven filtered, 1.07 seconds. The actual PDFium import survives reader preemption and publishes exactly one governed ID that resolves to its committed document. Existing source-integrity, metadata and memory-release assertions remain.

Formatting, offline native build, and binary Clippy with warnings denied passed. Four final native cases use isolated synthetic stores and the actual import runtime:

| Case | Observed result |
| --- | --- |
| import-rule-preview | Imports three generated PDFs beside ten existing catalogue records; the rule previews exactly the three new identities and their extracted titles |
| import-rule-applied | Reviews and confirms that scope; three documents processed and exactly three favorites in the 13-record catalogue |
| import-rule-undone | Reverses the same execution; all three imported favorite values restored and the original ten preserved |
| import-rule-no-context | Zero matches without an import batch, with an explanatory state in a requested 640 × 480 logical viewport at 200% text |

Execution deliberately clears the completed simulation; its verifier checks the resulting idle state rather than expecting a stale preview. During visual review, an old unconditional no-batch sentence was removed, and the small no-context preview was scrolled to its close action. Final controls and durable-state assertions passed. The final preview and small no-context PNGs were inspected. Captures contain generated PDFs and synthetic catalogue records only. Source, lockfile, executable and artifact hashes were checked before publication, and PNG provenance was embedded. See the [capture manifest](../screenshots/rust-import-rule-context/capture-manifest.json).

## Remaining acceptance

Import context is not persisted across app restarts and does not reconstruct the original full batch after a resumed job. This feature provides a current-session workflow with explicit simulation and confirmation. It does not establish unattended rule execution, complete historical import-rule recovery, physical pointer/keyboard/Narrator acceptance, whole-library performance targets, or release packaging. No Computer Use or personal PDF publication was performed.

## Persistence follow-up

[Durable completed-import context](rust-import-context-restart-2026-11-02.md) records later journal persistence and native restart evidence. Earlier session-only limitations above describe this checkpoint. Reconstructing the complete original scope of a resumed import remains separate work.

## Resumed-scope follow-up

[Resumed import context](rust-resumed-import-context-2026-11-02.md) later connects known prior checkpoint members and atomic main-commit ownership proofs. Older scope statements above describe their original checkpoints; unknown legacy provenance is preserved without inventing a partial rule scope.
