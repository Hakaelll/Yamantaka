# Bookmark persistence and actor validation — 2026-10-09

This checkpoint implements bookmark storage and application commands. The reader
bookmark panel is not connected yet; this is persistence evidence, not completed
user-facing bookmark parity.

The repository reads the existing bookmark table without changing its schema.
Reads preserve opaque IDs, nullable labels and raw timestamp strings. Saving an
existing document/page pair updates its label and modification time while
preserving its ID and creation time. New labels are trimmed; whitespace-only
labels become null. New writes accept at most 4096 Unicode characters, while
legacy labels are read without truncation.

Rename and removal require both the bookmark ID and its document ID. A missing
document or a bookmark belonging to another document returns a typed failure.
The write and its returned list snapshot share one transaction. Cancellation or
snapshot decoding failure rolls back the write, avoiding a reported failure
after an already committed mutation.

Lists contain at most 80 entries, ordered by page and ID. Count and rows share a
read transaction. Offsets align to the page size and clamp to the last available
page, including after deletion and for the largest platform offset. Persisted
page counts can be stale, so storage does not reject a bookmark solely because
its page exceeds catalogue metadata. The connected reader must validate actual
PDF bounds before saving or navigating.

Seven SQLite tests cover:

- Legacy identity and creation-time preservation, Unicode labels, null labels
  and document-scoped editing.
- A 161-entry catalogue, deterministic pagination and deletion of the final row.
- Invalid labels, missing documents, cross-document changes and cancelled reads
  and writes without unintended changes.
- An unrepresentable legacy page value: snapshot decoding fails and a concurrent
  attempted save rolls back, preserving the original row for recovery.
- Writes through historical schemas 1–11 without changing their schema version.
- Exact bookmark values after database reopening and v9 export/restore preview.
- Saving a canonical page when the stored document page count is stale.

An application integration test holds a native indexing opening in flight while
real SQLite bookmark commands save, rename, query and remove entries. It verifies
request-token identity, scoped cancellation failures, catalogue responsiveness
and clean indexing cancellation. Bookmark work runs on the application actor
without waiting for that native operation.

Validation for this checkpoint:

```text
cargo test --workspace --locked                          132 passed
cargo clippy --workspace --all-targets --locked -- -D warnings  passed
cargo fmt --all -- --check                               passed
```

The application tests account for 25 of the workspace tests. This checkpoint
adds seven storage tests and one application integration test. It does not add
new interface captures. Bookmark editing, canonical jumps, current-page state
and the unified reader navigator remain pending. Immediate FTS refresh following
an interactive bookmark mutation is also pending; the existing index refresh
path includes bookmark entries. No release acceptance claim follows from these
tests.
