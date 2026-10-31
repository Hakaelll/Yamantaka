# Reject stale annotation edits

Date: 2026-10-31. Scope: individual edits to existing annotations in either reader.

## Change

Individual edit commands now require the revision of the annotation the user opened. The storage adapter reads that exact document/annotation pair and checks its revision inside the same write transaction, before allocating the undo snapshot or changing any row. Missing or changed annotations return the typed `StaleAnnotation` failure. All existing edit callers use this contract; there is no optional unchecked edit path.

The SHA-256 revision covers annotation identity, values, tags, timestamps, origin fields and inherited columns. Inherited scalar values include their type, and binary data is hashed without converting it to text. Floating inherited columns use their exact bit representation. The catalogue title is excluded because it is presentation data rather than annotation content.

The record calculates its revision once when the storage worker constructs it. Immutable `AnnotationRecord` ownership retains that revision with the governed allocation. The UI copies or compares the resulting 32 bytes; it does not serialize and hash the note body during an interaction. Validation releases its temporary record before capturing undo history. These changes reduce redundant work by construction, but this checkpoint includes no measured speedup claim.

After rejection, the editor keeps the uncommitted text and blocks Save. A subsequent list reload compares the editor's frozen source with the new record and keeps that block when they differ. Cancelling the editor allows a fresh query and a new edit. The original PDF is unchanged.

## Executed evidence

Formatting, offline workspace Clippy for all targets with warnings denied, the retained historical-schema compatibility test and the native build passed. The retained annotation-history helper verifies accepted edits, rejected stale edits, unchanged undo/redo counts, unchanged FTS results after rejection, inherited binary-column changes and edits made directly in SQL without changing the timestamp. Changing only the catalogue title leaves the annotation revision unchanged. Its existing undo/redo and 100-entry history-bound checks still pass. No standalone Rust test was added.

Two native synthetic profiles passed. The conflict controller freezes an older editor, commits a newer note, submits the older edit and reloads the list. The saved note must remain `Newer annotation text`; `Uncommitted older draft` must remain in the editor with Save disabled. The shared-reference marks profile exercises the migrated edit contract through ordinary actor writes and confirms shared annotation refresh. The [capture manifest](../screenshots/rust-annotation-edit-revisions/capture-manifest.json) records source, executable and artifact hashes.

## Remaining acceptance

This is concurrency protection, not persistent recovery of existing-note drafts. Such drafts and selection-based annotation drafts still need durable recovery. Physical multi-window interaction, clipboard use during conflict review, large inherited-record performance and the wider migration/Release gates remain pending. The guard applies to the individual Edit contract; legacy whole-record restore, batch review and other explicit mutation contracts retain their separate semantics.
