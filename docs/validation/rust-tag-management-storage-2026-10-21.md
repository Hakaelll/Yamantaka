# Guarded shared-tag changes

Date: 2026-10-21. Scope: repository and existing actor contracts for tag rename, color, merge and deletion. The native tag manager does not expose these actions yet. Confirmation, destination selection and reconciliation of active filters and annotation views remain required before user-facing tag management is complete.

Follow-up: [native tag management](rust-tag-management-ui-2026-10-21.md) now connects the controls, confirmation and selected-filter/view reconciliation. Interface-pending statements below describe this preceding storage checkpoint.

## Data behavior

Tag queries now return the stored metadata revision. Rename, color and deletion require that loaded revision; merge checks both source and destination. A stale or missing tag rejects the change. Rename retains the identifier and validates the existing 60 UTF-16-unit name limit and normalized uniqueness. Colors accept the supported names or a six-digit hexadecimal RGB value. Existing imported colors remain unchanged unless explicitly edited.

Merge retains the destination identifier, name and color. It copies document and annotation associations with duplicate suppression, deletes the source through the existing foreign-key cascades, and advances the destination revision. Merging a tag into itself does nothing after revision validation. Deletion removes associations without deleting documents or annotations.

All mutations use the existing immediate transaction. Cancellation is checked before admission, between merge steps, during affected-page refresh and before commit. SQL, revision or cache failures roll back the complete change. An executing SQLite statement itself is not interruptible. Schema v11 and backup formats remain unchanged.

For rename, merge and deletion, a transaction-local SQLite temporary table captures the affected document and annotation pages before associations change. A second temporary table records eligible cached keys. Fixed SQL statements refresh the affected entries as a batch, without a Rust membership list or a full FTS scan for every page. Disabled-index eligibility uses one outer FTS scan and indexed snapshot-key lookups. Existing document metadata, page-text tag fields and annotation search entries refresh in the same transaction. With indexing disabled, annotation pages refresh only when their annotation cache already exists. Excluded documents receive no new search entries. Color changes do not alter searchable text or initiate indexing.

Renaming also exposes an inherited identity case: a tag may retain the deterministic identifier derived from its former name. Shared tag creation now checks whether that identifier is occupied and uses a new UUID when necessary. This permits reuse of the former name without rewriting existing identifiers. Both document-tag creation and annotation-tag creation use the same function.

## Verification

The retained compatibility case `overrides_win_and_queries_are_bounded_and_escape_wildcards` exercises the actual repository and FTS5 database. Its existing helper covers:

- Rename with cached page-text and annotation search refresh while indexing is disabled.
- Reuse of the former name with a distinct identifier, preserving the renamed tag.
- Rejection of stale revisions, duplicate or invalid names, invalid colors and pre-cancelled merges.
- Rollback after a search table becomes unavailable, followed by another operation on the same connection.
- Rollback of inserted destination associations when a source association rejects cascading deletion.
- Union of overlapping document and annotation associations, preservation of destination metadata, self-merge and deletion.
- Removal of stale search terms while source documents and annotation records remain present, with SQLite integrity valid.
- Unindexed and excluded document/annotation fixtures retain their cache row counts after tag renaming.

No standalone test was added. The retained Rust source count remains 55. All 55 retained workspace tests passed offline after the batch rewrite, including native PDFium and DX12 checks. Formatting, workspace all-target offline Clippy with warnings denied and the native development build also passed. Independent review executed the batch SQL against mixed cached, uncached, annotation-only and excluded cases. SQLite query plans confirmed the outer FTS scan and indexed snapshot lookups. This establishes query structure and behavior, not an end-to-end latency result.

## Remaining acceptance

Native editing controls, explicit merge/delete review, bounded destination search, selected-filter replacement, annotation-view invalidation and physical interaction remain pending. No large-membership latency, interruption during an SQLite statement, full migration or release acceptance is claimed. The repository capability alone does not complete the tag-management parity row.
