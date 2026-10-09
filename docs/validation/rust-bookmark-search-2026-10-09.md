# Bookmark search updates

Bookmark changes now update their search entry in the same SQLite transaction.
The application actor passes its current `IndexAccess` to `change_bookmark`.
That single contract handles saving, renaming, removal, and snapshot delivery.
All callers use the explicit access value. No compatibility method remains.

With access enabled, storage replaces the entry for the affected document and
page. Removal deletes that entry. The change does not refresh document metadata,
extract PDF text, or change a running index session's revision. Bookmarks can
appear in search before their PDF has been indexed. The bulk refresh preserves
that behavior and keeps excluded documents out of bookmark entries.

With access disabled, storage makes no FTS changes. Previously retained entries
remain until the user erases the index, as with the existing privacy controls.
The actor rejects searches while access is disabled. An enabled edit to an
excluded document does not reintroduce its bookmark text into the index.

An FTS failure, cancellation, or returned-snapshot failure rolls back the
bookmark transaction. The reader cannot receive a successful bookmark snapshot
for a half-applied edit.

Four SQLite regressions cover these outcomes:

- Immediate Unicode label search, idempotent saving, rename, removal, and bulk
  refresh before any PDF index state exists.
- Disabled access, exclusion, cancellation, and explicit reinclusion.
- An unavailable FTS table that rolls back an enabled edit while disabled edits
  continue to work.
- A running PDF index session that remains valid after a bookmark change, with
  another document's bookmark and existing page text still searchable.

The actor integration test also searches a renamed bookmark while native PDF
opening is held in flight. It verifies the exact page result and the absence of
that result after removal. Native work remains pending throughout those reads.

Validation on 2026-10-09:

```text
cargo test --workspace --locked                          141 passed
cargo clippy --workspace --all-targets --locked -- -D warnings  passed
cargo fmt --all -- --check                               passed
```

The requested skills informed the review. Ponytail favored the existing SQLite
transaction. pstack's Migrate Callers Then Delete Legacy APIs removed the second
mutation method. Test Behavior, Not Implementation kept the checks focused on
search results and persisted values. The Thermo-Nuclear Code Quality Review
checked the ownership boundary, duplicated paths, and atomicity. The review
found and removed the unnecessary method before publication.

The filtered FTS deletion is not a constant-time lookup. The legacy FTS schema
marks document and page identifiers `UNINDEXED`, so SQLite can scan retained
entries to find the affected row. This checkpoint does not establish latency at
the required large-catalogue sizes. Those performance gates and the broader
rewrite acceptance remain pending. No interface changes or new visual captures
are part of this checkpoint.
