# Coherent annotation traversal for export

The export service now reads every annotation for a document through one SQLite read transaction. It visits records in page, creation-time and ID order, with no 80-row or offset limit. The application service feeds each borrowed record directly to the incremental Markdown/JSON writer and returns the output only after traversal and finalization succeed.

The repository reuses the canonical full-record decoder, including tags, source identity, editorial and attachment states, origin fields and extension columns. Each row retains a reservation from the shared memory governor. The row drops after the visitor returns, so traversal does not accumulate a document-sized collection. Cancellation is checked at entry, during decoding, after each callback and before transaction completion. Visitor errors and denied memory admission release the row and end the read without a successful count.

The original paged Notes query still returns at most 80 records. Both queries now use one shared column projection, so their decoding contracts cannot drift independently.

## Verification

Three storage regressions prove:

- A real second WAL connection changes every note and tag and deletes the last row during traversal. The active snapshot still returns all 181 original rows and original tags. A later traversal sees 180 changed rows with the new tags.
- A visitor failure after row 81, cancellation during the callback and denied row admission release governed memory. A fresh traversal succeeds, and an unrelated document returns no rows.
- Each historical schema from v1 through v11 migrates and supports the ordered visitor. Existing annotation payloads inserted before migration for v2 through v11 preserve Unicode note/selection text, rotation, source fingerprint and creation time through the new visitor.

The application export regression now uses the connected service directly. Parsed JSON contains all 181 records and the final ID, with no retained source leases after traversal.

Formatting and warning-denied workspace/all-target Clippy pass. The full workspace run passed before the final historical fixture expansion, followed by another passing run of the three storage traversal tests. Local logs are `build/verification/annotation-export-snapshot-tests.log` and `build/verification/annotation-visit-history-tests.log`.

## Remaining scope

This snapshot covers annotation rows, their tags and persisted source fields. Export metadata still comes from the caller's frozen context. Actor routing, native save dialog, destination protection/overwrite confirmation, staged atomic publication, progress/error presentation and native export acceptance remain pending. The output writer can contain partial bytes on failure, so publication must occur only after the service succeeds. Large-export latency, cancellation granularity inside a single large row and complete SQLite/output resource accounting also need acceptance. No complete export UI or final release parity is claimed.
