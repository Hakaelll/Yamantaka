# Annotation transactions across documents

Global review needs to change notes from several documents without partially applying a selection. The domain now exposes a frozen AnnotationTarget containing both document and annotation identity, plus typed review/tag batch changes. The application owns the targets and routes a Batch operation through the existing annotation actor. A global query uses the existing optional document filter; replies retain that query scope and their request token.

SQLite applies at most 80 targets in one transaction. It validates the recorded document identity for each annotation, deduplicates repeated targets, and records affected document/page pairs. Review changes and tag addition reuse the same canonical row helpers as the reader's document-scoped commands. Tag creation/linking, consented FTS refresh and the returned governed query all occur before commit. Any failure or cancellation rolls the whole operation back. Empty or oversized global mutations are rejected.

This adds the transaction and actor foundation for the global review screen. It does not expose that screen yet. The existing reader selection, filters and bulk actions retain their document scope.

## Verification

Four new SQLite regressions cover:

- Selected notes in two documents change state while an unselected note stays pending. The pending query refresh reflects the change, duplicate targets do not duplicate links, case variants reuse one Unicode tag, and consented annotation search returns both affected documents.
- A missing late annotation or a stale document/annotation pairing rolls back review and tag changes in every document.
- Cancellation before entry, denied result memory, empty selection and 81 targets preserve the original rows and release governed memory.
- A late twelve-tag limit, failed FTS refresh or malformed unrelated result row prevents all tag changes from committing.

A new runtime integration test holds a controlled PDF engine's index-open ticket while submitting global review, tagging, cancellation and global reads. SQL commands complete while the PDF ticket remains held. Replies preserve their request token and optional document scope, cancelled work does not change accepted rows, and commands request no extra PDF work. Index cancellation then closes the held engine and shutdown releases governed annotation memory. This is a controlled engine test, not a new PDFium performance measurement.

Formatting, warning-denied workspace/all-target Clippy and the full locked workspace suite pass. Existing reader bulk review/tag, storage, search, export and editor tests remain passing. Local log: `build/verification/annotation-global-batch-tests.log`.

## Remaining scope

The global review list/detail screen, navigation to source documents, its full filtering/grouping controls, selection UI and native interaction captures remain pending. Mid-transaction interruption at the process level and broader mutation resource/latency acceptance remain pending. This checkpoint makes no claim that global review or the full rewrite is complete.
