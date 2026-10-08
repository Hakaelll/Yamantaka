# Populated compatibility fixtures

These are synthetic inputs, generated on 2026-10-08 from the committed historical
Flutter SQL and backup column contracts. They are not exports produced by an old
Flutter executable. The Python SQLite checks validated strict types, checks and
foreign keys during creation; the Rust compatibility tests remain the authority
for the Rust implementation once executed.

`schema-v01.sql` through `schema-v11.sql` populate every represented domain table
available at that schema version. They include two documents, Unicode metadata,
reading preferences, bookmarks, sessions, annotations and their tag edges,
collections, profiles, workspaces and queue state, relationships, revision
membership, automation and its undo journal, duplicate history, and app metadata.
Where supported they also include FTS data, indexing state and a manually chosen
cover page with deliberately synthetic cache paths.

`populated-backup-v9.json` includes all 23 exported domain tables. The backup test
projects each row onto each version's exact required columns, validates the
isolated restore, exports v9, validates category hashes, and compares the entire
historical projection. It checks data values and edges as well as counts. Covers
and FTS are intentionally outside the historical JSON backup contract.

Paths name synthetic files. These fixtures do not imply that referenced PDFs or
old duplicate-safety backups are present. Native migration previews and real
historical-export comparisons remain separate acceptance work.
