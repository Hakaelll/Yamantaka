# Coherent annotation review snapshots

The storage repository now captures a filtered review page and its document metadata in one SQLite read transaction. The resulting domain snapshot is immutable and owns at most 80 governed annotation records plus one metadata record per referenced document. Catalogue title and author projection follows manual corrections before stored/extracted values. Missing document metadata is represented explicitly; it is not replaced with invented persisted values.

The snapshot reserves its metadata vector and string bytes before copying them, then accounts for their actual capacities. Cancellation or decoding/admission failure drops both the annotation page and metadata leases. The independent read-only export connection supports the same operation without borrowing the actor's connection.

The application writer now accepts this snapshot directly. It groups borrowed records using fixed stack arrays and a governed descriptor vector, then calls the compatible review formatter. No annotation bodies are cloned. The review export remains limited to the filtered page; reader export retains complete-document traversal.

## Verification

Three new SQLite regressions cover:

- An 181-note fixture returns 80 records and metadata from both documents, preserves manual title/author and progress, applies a text filter, and returns the final 21 records at offset 160.
- A file-backed WAL library supplies the snapshot through an independent read-only connection. A subsequent writer transaction changes document metadata and notes. The returned snapshot retains the old values while the next snapshot sees the new values. This tests immutable capture and independent readers, not a stress measurement of simultaneous reads and writes.
- Cancellation, invalid UTF-8 metadata and metadata-budget denial release every lease. The denial fixture first proves the annotation page itself fits the budget, then fails while adding a large author value. Correcting malformed metadata permits another successful snapshot.

A new application regression exports the captured page after the source library changes. JSON retains the captured manual title, author, progress and note text, with exactly two records in the first document and 78 in the second. Borrowed grouping/output memory is released after writing, and dropping the snapshot releases the remaining text memory.

Formatting and warning-denied workspace/all-target Clippy pass. All 320 locked workspace tests pass. The result is recorded locally in `build/verification/annotation-review-snapshot-tests.log`.

## Remaining integration

The storage-to-writer path is verified. The global export worker request, safe file publication integration, native save controls, cancellation/recovery UI and capture evidence are still pending. Large real-metadata latency, concurrent mutation stress and physical interaction acceptance remain open. No Release or full-rewrite completion is claimed.
