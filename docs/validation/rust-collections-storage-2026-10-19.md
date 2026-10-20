# Collection storage contracts

The Rust repository now exposes typed collection pages and changes on the
existing schema-v11 tables. This checkpoint prepares the native collection
manager. It does not expose collection management in the UI yet.

Follow-up: [collection order and icons](rust-collection-order-2026-10-20.md) now connects native movement and icon editing. Pending statements below describe the earlier checkpoint.

## Connected Flutter reference

`lib/storage/repositories/organization_repository.dart` and its controller define
collection creation, renaming, pinning, membership, deletion and ordering.
Collections preserve their existing identifiers, icons, stored sort order and
member document IDs. The Rust changes add no schema or backup format version.
New collection IDs are UUIDs; imported IDs are never rewritten.

Queries return at most 80 collections, their document counts, and optional
membership for one requested document. They order pinned collections first,
then stored order, name and ID. Search treats percent, underscore and backslash
as literal input. Pagination fetches no complete document membership map.

Creation trims the name, checks the same 60 UTF-16-unit limit as Flutter,
normalizes its uniqueness key with lowercase, and accepts the five supported
icons. It appends to the stored order. Renaming retains the ID, icon, pin state
and membership. Metadata edits and deletion require the loaded ID/timestamp
revision; a newer metadata revision rejects the command. Generated timestamps
advance beyond the previous parsed revision even if the clock has not advanced.

Membership sets a desired state with an idempotent insert or scoped delete.
It verifies that both the collection revision and document still exist. A
membership operation does not change the collection's metadata revision.
Deleting a collection uses the existing foreign-key semantics and keeps its
PDF documents. Every mutation runs in an immediate transaction, checks
cancellation before entry and before commit, and rolls back on failure.
Cancellation cannot interrupt an SQLite statement already executing.

## Executed verification

The retained bounded-query compatibility test now calls these public repository
operations. It verifies Unicode names, trimmed/case-normalized uniqueness,
invalid UTF-16 lengths, idempotent assignment, removal and catalogue collection
filtering. It checks renaming, stale-revision rejection, literal search,
rollback after a forced delete failure, pre-cancellation and missing-document
rejection. A catalogue of 82 collections returns pages of 80 and 2 with no shared
IDs; pinning moves a later collection to the front. Deleting a collection removes
its membership while retaining the original document's title, path and progress.
SQLite integrity remains valid.

That targeted test passes. Cargo formatting, workspace all-target Clippy with
warnings denied and the offline native development build pass. No test case was
added; the reduced Rust suite remains at 54 cases. The full suite was not rerun.
No personal library, PDF, database or screenshot is part of this checkpoint.

## Remaining work

The application actor commands, collection manager, inspector membership editor,
active filter display and preference restoration still need wiring. Reordering,
multiselection/bulk operations, tag management and smart views remain incomplete.
These storage checks do not prove native interaction, accessibility, persisted
reopen/crash behavior or large-catalogue timing. Collection parity remains partial
and the full rewrite/release goal remains open.
