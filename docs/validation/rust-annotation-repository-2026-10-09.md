# Annotation repository checkpoint

The SQLite repository now reads and changes annotations through a UI/FFI-independent domain contract. This checkpoint implements storage operations. The application actor and visual editor do not yet call them.

## Stored data and transactions

Queries return at most 80 independently owned records. They combine document, page, kind, color, review state, tag, literal text and update-time filters. Document queries default to page order. Global queries default to recent updates. Offsets retain the legacy 100,000 limit.

Creation requires a governed selection draft. A proposed page-note fallback needs explicit confirmation before the repository accepts it. Changes support note/kind/color edits, deletion, editorial review batches, tag batches and fingerprint attachment checks. Attachment status and editorial review remain independent. Notes retain the 4,000 UTF-16-unit limit, annotations support twelve tags, and review/tag batches accept at most 80 IDs.

Snapshot restoration preserves the annotation ID, document, page, creation/update timestamps, selected text, context, normalized rectangles, rotation, fingerprint, attachment status, review state, tags and origin fields. Additional SQLite columns retain their NULL, integer, real, text or blob values. Restoration includes these columns in the INSERT itself, so an insertion trigger can inspect their original values. Column names must exist in the actual schema, cannot replace known identity fields, and are quoted as identifiers. Values use SQL parameters.

All writes, tag joins, consented annotation FTS updates and the returned page share one transaction. A cancelled request, wrong-document ID, invalid record, search-update failure or denied response reservation rolls the operation back. Snapshot saves preserve timestamps; direct edits generate a new UTC update timestamp. Search changes respect opt-in and exclusions and leave active PDF indexing progress and revision unchanged. The main schema stays at version 11. PDF originals are never write targets.

## Memory and malformed data

The shared Text budget reserves record strings, geometry, tag lists, extension values and result arrays before allocation. Records and pages release their leases on drop. SQL construction and geometry serialization use temporary reservations. Reservations are conservative payload accounting, not measurements of SQLite's internal allocations or process RSS.

Typed decoding rejects unknown enum values, invalid normalized rectangles and unknown rectangle fields. A rejected read leaves the original row unchanged. Automatic quarantine and recovery presentation remain pending. Cancellation is checked between operations, rows and before commit; this checkpoint does not add a SQLite statement interruption hook.

## Verification

Seven synthetic SQLite integration tests cover:

- Create, edit, tags, review, deletion and exact snapshot restoration, including all five SQLite extension value types and an insertion trigger that requires an extension value.
- Explicit page-note confirmation, cancellation, ownership checks and rollback when preparing the returned page fails.
- Search consent, exclusions, injected FTS failure and independence from an active PDF indexing session.
- Migration from each schema version 1 through 11, legacy annotation preservation, reopening and exact stored values.
- The 80-row page limit, capped offsets, combined filters, literal wildcard characters, time boundaries and rollback after memory admission failure.
- Malformed stored geometry/enums and cross-document batch rollback without normalization of the original rows.
- Version 9 backup restoration and separate attachment/editorial review states.

`cargo fmt --all --check`, warning-denied workspace/all-target Clippy and `cargo test --workspace --locked --quiet` pass. The local workspace log is `build/verification/annotation-repository-tests.log`. This backend checkpoint changes no UI presentation and claims no new native visual acceptance.

## Remaining integration

Actor commands, stale-result rejection, the annotation editor, fallback confirmation UI, painted marks, mutation history, global review and Markdown/JSON export remain pending. Broader backup-version acceptance, interrupted-write recovery, large-library latency, clipboard, physical DPI, Narrator and final release gates remain open in the parity matrix.
