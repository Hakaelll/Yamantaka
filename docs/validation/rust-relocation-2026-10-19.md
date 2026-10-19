# Missing-source relocation

A missing document now exposes **Locate PDF…** in Details. Use **Check file** to
mark a missing source first. The native picker selects a candidate; selection
alone does not update the catalogue. The inspector presents the file name,
verification result, **Update location** and **Cancel**. The full path is available
on hover. The gallery geometry and global chrome are unchanged.

## Identity and confirmation

The application distinguishes two source histories. If a source-bound full
reference exists, the selected candidate must match that complete SHA-256, saved
size and legacy fingerprint. A different tail after the first 64 KiB is rejected,
even when the legacy fingerprint matches. The review describes a match against
the saved full reference, not a retroactive proof of the original import.

For an inherited document with no full reference, the saved size and partial
fingerprint must match. The review states that complete equality is unverified
and requires explicit confirmation that this is the same book. That historical
limitation cannot be recovered by hashing only the new candidate. It is never
presented as an exact duplicate or used to merge records. Confirmation saves a
full reference for future checks of the selected source.

Inspect and Confirm each perform a fresh full read outside the repository actor,
using one governed 64 KiB buffer and the shared guarded reader. The review does
not retain a file lock while waiting for a decision. Confirmation checks the
reviewed canonical path, timestamp, size, fingerprint and full digest again. Its
read-only Windows handle denies concurrent writes/deletion through persistence.
Cancellation and shutdown cancel and join the owned worker.

The shared domain plan contains the expected catalogue/reference snapshot, the
candidate's source fields and full digest. At most one relocation job runs in the
actor. Current or opening readers prevent admission and cancel pending work;
the inspector also requires this document's reading tabs to be closed. Leaving
the inspector, selecting another book or opening a reading discards pending review.

## Persistence and derived data

An immediate main transaction compares the complete expected source/reference
snapshot before writing. It rejects a candidate path assigned to another document,
including ordinary/verbatim Windows path representations. Unicode paths are
persisted without lossy conversion. Arbitrary hard-link aliases and network
identity equivalence are not certified by this checkpoint.

The write changes only path, normalized path, file name, physical date and missing
state. Document ID, fingerprint, page count, manual title/author, favorite,
progress, annotation geometry/content, bookmarks, cover selection, sessions and
relationships are retained. The original PDF is never an output target. Main
schema v11 remains unchanged. The full reference uses the existing separate
versioned store and retains the previous key if the main transaction fails.

Derived FTS rows for the document are cleared in the same main transaction. An
existing index state receives a new revision and becomes pending; an excluded
state stays excluded. No index state is created for a document that had none, and
relocation does not enable indexing or start it automatically. Late old indexing
sessions fail their revision check. Native source evidence remains bound to its
source stamp; reanalysis can regenerate stale evidence while keeping overrides.

## Verification

The existing comparison/file-check test was extended. No test cases were added;
the minimal suite remains at 53. Both retained comparison tests passed. The
relocation sequence covers a non-ASCII path, a changed tail despite an equal
partial fingerprint, inspection without mutation, change after review, a registered
path collision, a simulated main write interruption, cancellation, stale review
rejection, successful confirmation and persistence across database reopening.

It checks preserved identity, fingerprint, latest manual title, author, favorite,
page position, bookmark, note and manual cover page. It verifies index invalidation,
retained exclusion, no file writes and governed-buffer release. Workspace Clippy
with all targets and warnings denied, formatting and the locked offline native
build are checked for this checkpoint. The complete workspace suite was not rerun.

[Native synthetic captures](../screenshots/rust-relocation) cover full-reference
review, successful confirmation in Spanish light mode with reduced motion,
mismatch rejection and a legacy review in a 640×480 logical viewport with 200%
text. At that enlarged size the review exceeds the default inspection tray: vertical scrolling or resizing exposes the whole warning and actions; the small capture shows the action section rather than certifying simultaneous visibility. They drive the real command/reply route against an owned synthetic file
copy. Native picker interaction and physical keyboard/mouse accessibility are
not automated by these captures. Only provenance-bound synthetic images and
redacted control records are published.

## Remaining acceptance

Physical picker/input and Narrator acceptance, monitor DPI, network and hard-link
aliases, mapped writes, forced-process/power-loss faults, concurrent native-reader
stress and measured large-file latency remain pending. Relocation changes a
catalogue reference; it does not move files, merge duplicate documents, validate
PDF syntax or complete the whole Rust rewrite.
