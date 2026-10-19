# Shared library tag storage

The Rust repository now exposes bounded tag queries, normalized tag creation
and idempotent document assignment/removal. It reuses the tag identity helper
already used by annotation edits. Existing tag IDs, names and colors remain in
place; creating a matching normalized name returns that existing tag.
This storage checkpoint is not yet connected to a document-tag manager in the UI.

## Membership and search consistency

Tag pages return at most 80 rows, separate document/annotation counts, color and
optional membership for one document. Exact-ID lookup supports saved selections
outside the first page. Search escapes literal percent, underscore and backslash.
New library tag names retain Flutter's 60 UTF-16-unit limit. Existing annotation
tag behavior and its wider legacy input limit are unchanged.

Document membership changes run inside an immediate transaction and verify both
IDs. Repeated assignment/removal does not rewrite the search cache. Removing a
document tag never removes its annotation memberships or the shared tag row.
Cancellation is checked before work and before commit; an SQLite call already
in progress cannot be forcibly interrupted by this token.

A changed membership updates metadata tags, cached page-text tag columns and tag
search entries inside that same transaction. The active PDF indexing revision
and cursor are preserved. Annotation tag entries retain their own semantics.
Excluded documents receive no search entries. If global indexing is disabled,
an uncached document receives no new index rows; an already materialized cache
is maintained so later searches do not expose obsolete tags. This maintenance
reads no PDF and starts no indexing worker. A search-cache failure rolls back
the document membership change.

Three covering indexes support reverse document/annotation tag membership and
name ordering. They are added idempotently within the existing schema migration
transaction for versions with organization tables. Main schema version 11,
backup contracts and imported row values are unchanged. No speedup or complete
large-catalogue latency budget is claimed without measurement.

## Executed verification

The retained catalogue-query case now checks reuse of a shared legacy tag ID and
color, independent membership counts, normalized duplicate creation, UTF-16 name
limits, literal search, idempotent assignment and the catalogue tag predicate.
It builds a synthetic three-page search index, changes document tags and checks
page/tag search results while preserving a tagged annotation. Existing cached
rows stay current with indexing disabled; an uncached disabled document creates
no rows. Exclusions, cancellation and absent IDs are checked. A missing FTS table
forces rollback after the membership change starts. An 84-tag dataset returns
pages of 80 and 4 with no duplicate IDs, including exact lookup past page one.

All six retained compatibility cases pass. The populated historical-data case
also passes across every v1–v11 fixture, retaining every original column value.
Cargo formatting, workspace all-target Clippy with warnings denied and the
offline native development build pass. The reduced suite remains at 54 cases;
the full workspace suite is not rerun. These fixtures are synthetic and no
personal database, document or screenshot is published.

## Remaining work

Application actor commands, the document-tag manager, active filter controls and
preference restoration still need wiring. Tag renaming, merging, deletion, color
editing, multi-document operations and their broader search/journal acceptance
remain incomplete. Full restart/crash, permission faults and large-index update
timing remain pending. The rewrite and final Windows release remain unfinished.
