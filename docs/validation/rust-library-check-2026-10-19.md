# Library-wide source verification

**Tools → Check library files → Check now** scans the registered documents through
the existing full-file verifier. Its submenu reports available, missing, changed,
unverifiable and failed items. A completed scan can contain errors: completion
means the traversal finished, not that every source is valid. **Cancel check**
keeps results already committed. The submenu shows cancellation finishing until
the owned worker stops; no global progress strip or automatic startup scan is added.

**Show missing** and **Show changed** open the corresponding catalogue filters.
The shortcuts reset text, favorite and reading-state predicates so affected books
are not accidentally hidden. **State → Modified files only** is independently
composable with the existing filters and sort options. Its preference persists in
the Rust preference namespace and both Reset filters paths clear it.

## Traversal and scheduling

A domain repository contract obtains the initial highest document ID and retrieves
ordered pages of at most 80 records. Queries use the primary-key cursor rather
than a growing offset. The application checks page size, order and cursor bounds.
It retains one page and admits at most one hashing job at a time. Each cooperative
actor step either accepts one result or admits one document; failures cannot loop
through the entire catalogue in a single step. While a hash is pending, the actor
waits on its real readiness channel and still services other commands.

The initial upper ID bounds the scan. This is not a long database snapshot: newly
inserted IDs above that bound are excluded, while concurrent additions ahead of
the cursor but below the bound can be observed. Removed or replaced queued rows
fail their source checks. Additions behind the cursor require another explicit
scan. The result counts describe attempted items, including failures, rather than
promising a fixed percentage of an unchanging catalogue.

The batch reuses FileCheckJob with its governed 64 KiB buffer, full SHA-256 stream,
held Windows read handle, source/reference snapshot and late guarded write. It
does not use PDFium or hold a database transaction while reading a file. The
[initial-reference limitation](rust-file-check-2026-10-19.md) still applies to
inherited records that have no historical complete hash. This scan neither
consolidates duplicate documents nor accepts a replacement version.

Progress is one scoped shared snapshot. Only a terminal reply uses the UI event
queue; each file does not allocate another queued event. The UI refreshes the
catalogue on completion or cancellation. Individual verification and relocation
are excluded while a library scan runs. Comparison and reading retain their
existing scheduling. Measured contention with active readers remains acceptance
work, not a performance claim from this implementation.

Cancellation before admission and cancellation after completed work produce a
cancelled terminal result. Cancellation joins any owned hashing worker. Shutdown
cancels the batch and releases its page and worker. A later explicit scan starts
again safely; there is no persisted scan journal or automatic resumption.

## Saved state

The existing schema-v11 `is_modified` and `last_validated_at` fields now record
confirmed observations from both individual and bulk checks. Changed contents
set the modified flag without replacing the fingerprint, dimensions or reading
data. A missing observation sets the missing flag. These observations retain
other previously known source flags. A verified available source clears both
flags and records its physical timestamp. Errors and unverifiable saved dates do
not invent an availability result or overwrite those flags.

Successful explicit relocation also clears the source flags and records the
validation date after its reviewed candidate passes confirmation. The existing
separate full-reference store still retains current/previous keys across main
write failure. No source PDF is an output target. IDs, manual metadata, progress,
annotations, bookmarks, covers and relationships are not replaced by this scan.

## Verification

The retained comparison/file-check verification was extended without adding test
cases. Both affected tests pass; the suite remains at 53. The new helper uses a
file-backed 85-document library and checks an empty scan, the 80-row boundary,
cancellation after one committed available source, a subsequent complete pass,
a deletion after the first page was queued and a later ID outside the initial
upper bound. It expects 80 available, one missing, one changed, one unverifiable
and two failed outcomes, including the removed record and a directory at a stored
PDF path.

It verifies modified/missing query results, clearing a prior modified flag,
validation timestamps, preserved manual title/author/favorite/position, unchanged
bytes, unmarked access/type failure and released governed memory. This is a
correctness fixture, not a throughput benchmark. Workspace formatting, all-target
Clippy with warnings denied and the locked offline native build are checked.
The complete workspace suite was not rerun.

[Native synthetic captures](../screenshots/rust-library-check) drive the real
actor for a ten-document mixed result, the modified filter in Spanish light mode
with reduced motion, cancellation before work and a small viewport at 200% text.
The mixed outcome is six available, one missing, one changed, one unverifiable
and one failed. Small menus scroll when all counts and actions cannot fit at once.
Only generated PDF copies, provenance-bound images and redacted controls are
published. Physical pointer/keyboard interaction and Narrator are not certified
by these captures.

## Remaining acceptance

Authorized folder watches, source replacement review, automatic full hashing at
import, cache-wide cleanup, network/ACL/mapped-write faults, power-loss recovery,
physical DPI/accessibility and concurrent-reader stress remain pending. The
5,000/25,000/100,000-document and paired Flutter/Rust timing gates still require
their own measurements. This checkpoint does not complete the rewrite or its
Windows release.
