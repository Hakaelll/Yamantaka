# Rust catalogue query — stage 2

Date: 2026-10-08. Scope: domain query contract and SQLite repository; UI integration is separate.

## Implemented contract

- `LibraryReadingState`: all, not started (`<= 0`), in progress (`> 0 && < 1`), completed (`>= 1`). These match the Flutter library query, including progress `0.995` remaining in progress. Smart collections use a different threshold and are not implemented here.
- Filters combine with AND: escaped text, favorites, missing state, reading state, authors, topics, tag IDs and a manual collection ID. Within authors/topics/tag IDs, any selected value matches. Membership filters use correlated EXISTS, so several matching assignments never multiply results or counts.
- Effective title/author keep the existing Rust precedence: override, detected metadata, source metadata, then filename/empty author. Author filters trim/lowercase selection values and compare normalized effective authors. Text also searches filename, library notes and persisted topic keys, matching the fields searched by the original SQLite repository. `%`, `_` and `\` remain literal escaped LIKE characters. No FTS index or FTS query was added.
- Six enum-selected sorts: last opened, title, author, added, progress, page count. Titles/authors default ascending; other sorts default descending. Explicit ascending/descending changes the primary key only; every mode ends in `id ASC` for deterministic pagination. The existing Flutter modes (recent, title, added, progress) keep their direction and stable ID tie rule. Author/page count and explicit direction extend that contract.
- Empty/default queries retain recent order (`last_opened_at DESC, id ASC`) and include missing documents. A zero page size means 80; requested sizes cap at 80. Count uses the identical predicate builder and ignores order, offset and page size. Offset conversion saturates instead of wrapping to a negative SQLite offset.
- All filter values, limits and offsets are SQL parameters. Order expressions and directions are selected from enums; membership placeholder lists contain only generated `?` markers.

## Validation

Added `rust/storage/tests/library_queries.rs` with nine tests covering every sort/direction, stable ties across pages, combined filters/count consistency, progress boundaries, effective overrides/source fallback, any-match memberships without duplicates, all text fields, escaped metacharacters, injection-shaped values, missing-state combinations, the 80-row cap and oversized offsets.

The nine query tests and storage regression suite passed in the integrated
workspace. Formatting and Clippy passed. A fresh Release measurement after query
integration ran five passes and 1,000 samples per corpus size; worst query+count
p95 was 2.451/6.546/24.782 ms for 5k/25k/100k documents. The 20-query pattern does
not cover every added facet and sort combination; broad performance parity is
still pending. See `rust-catalog-benchmark-stage2-2026-10-08.json` for raw samples.

## Remaining scope

This is partial query parity, not complete library parity. Smart collections, facet/count summaries, grouping, multiselection and bulk operations are outside this change. Organization tags are distinct from enrichment topics: tags filter by persisted organization ID, topics by persisted topic key. No document-type filter was invented because the current document schema and Flutter LibraryQuery have no document-type field; the import pipeline currently handles PDFs. SQLite LOWER/LIKE retain SQLite's existing ASCII case-folding semantics, without introducing Unicode collation or accent removal. Source metadata fallback is preserved from the existing Rust repository even though the Flutter SQL predicate uses only override/detected metadata.

Only `rust/domain/src/lib.rs`, `rust/storage/src/database.rs`, the new test file and this note are changed for this stage. The actor remains the UI access boundary; the database is not exposed to UI code. No schema migration, dependency or Document field was needed.
