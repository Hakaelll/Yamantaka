# Atomic library removal storage

Date: 2026-10-20. Scope: the local database operation required by reviewed library removal. The native selection menu does not invoke this operation yet. Reader retirement, actor admission, confirmation, cancellation/results and continuity reconciliation must be connected before presenting it as a completed feature. This checkpoint does not establish release readiness.

Follow-up: [the native removal interface](rust-library-removal-ui-2026-10-20.md) now connects confirmation, session saving, reader/continuity reconciliation and partial outcomes. Statements about the interface being pending below describe this earlier checkpoint. Physical and broader acceptance remain open.

## Connected Flutter behavior

Follow-up: [the runtime coordinator](rust-library-removal-runtime-2026-10-20.md) now implements close-before-delete admission, partial outcomes and stale-opening rejection. The report below records the preceding storage checkpoint; the user-facing action remains incomplete.

`LibraryBulkOperationService.execute(removeFromLibrary)` invokes cover cleanup and then `DocumentRepository.remove`, which deletes the document record. Foreign keys remove document-owned rows. The app shell closes the current reading when needed, refreshes organization/library state and clears last-document presentation settings. The original PDF is never deleted. Individual removal also asks for confirmation.

The Rust repository operation preserves the SQL removal semantics and strengthens index consistency. `LibraryRepository::remove_from_library` validates one identifier, checks cancellation, begins an immediate transaction, deletes that document's cached FTS rows and deletes exactly one document. It checks cancellation again before committing. A missing document, SQL failure or rejected commit leaves the transaction uncommitted. FTS5 has no document foreign key, so its cleanup explicitly participates in the same transaction even when indexing is disabled or the document was excluded.

The existing schema v11 foreign keys remove document preferences, notes, bookmarks, sessions, annotations and their tag memberships, document tags, collection membership, workspace tabs, cover decisions, enrichment, topics, search state, relations in either direction and revision membership. A revision group's current-document reference becomes null. Shared tags, collections, workspaces, profiles, rules, operation journals, merge history and opaque metadata are preserved. The schema and backup versions do not change.

This SQL adapter performs no filesystem deletion and does not trust stored thumbnail paths as deletion authority. PDF files and unrelated files referenced as thumbnails remain untouched. Reclaiming owned derived caches, auxiliary hash/import records and reconciling Rust reader-continuity files belong to subsequent orchestration; this checkpoint does not claim they are cleaned. Opaque metadata remains unchanged, including legacy keys which can contain document references.

## Executed evidence

- `cargo test -p yamantaka-storage --test compatibility overrides_win_and_queries_are_bounded_and_escape_wildcards --offline -- --exact` passed one retained case. The new helper reuses the populated historical v11 fixture rather than adding another standalone test. The retained Rust source count remains 55.
- The helper opens an isolated file-backed database and uses a separate SQL connection to inspect every logical user table before and after operations. Invalid, oversized, missing and pre-cancelled requests preserve the rows. A missing-ID request also retains a pre-existing orphan FTS row through rollback.
- Injected root-document and dependent-bookmark deletion failures restore every inspected row, including FTS entries deleted earlier in the transaction. This verifies transaction rollback after both root and cascade failures.
- Successful removal deletes the selected record and its dependent data, removes both relation directions, nulls the revision-group current pointer and preserves the other document's note, bookmark, annotation/provenance, index revision and indexed text. Shared entities and opaque JSON rows compare equal to their original values.
- SQLite integrity passed after removal. A v9 backup exported from that database passed restore preview with the expected two remaining documents and one remaining annotation.
- Two documents refer to the same synthetic PDF path. Its bytes and an unowned file deliberately placed in stored thumbnail paths remain equal to their original bytes after all operations.

Formatting, all-target offline Clippy with warnings denied and the offline native build are checked for this checkpoint. UI source/layout is unchanged, so no screenshot batch is required. Full workspace tests, indexed-corpus deletion latency and physical interaction checks have not been executed here.

## Remaining integration

Add explicit removal confirmation, retire selected native readers safely, cancel obsolete document consumers, preserve coherent sessions and continuity, and report partial cancellation/failures before refreshing selection and organization views. Coordinate with in-flight import/index work and verify late results cannot restore removed state. The storage method's contract requires callers to retire readers first; there is no user-facing removal command until that orchestration is implemented and verified.
