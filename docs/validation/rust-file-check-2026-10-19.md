# Source verification in document details

Details now includes **Check file**. Its cancellable background job distinguishes
an available source, changed contents, a missing path and an unverifiable saved
date. Access and sharing failures report an error rather than marking a document
missing. Results stay in the existing scrollable inspector.

## Content references and persistence

A first reference is accepted only when the current size, legacy fingerprint and
saved modification date match. Local Dart-style dates are supported when they
resolve unambiguously; microsecond precision tolerates sub-microsecond filesystem
rounding. An unparseable or ambiguous saved date produces an unverifiable result.
The first reference records the current full SHA-256. It does not retroactively
prove the bytes of a historical file that had no complete hash.

Later checks compare all bytes against that reference. A matching digest permits
refreshing the physical modification date and clearing the missing flag. A changed
source is reported without replacing its identity, metadata, page count or reading
data. Only a confirmed not-found result sets `is_missing`. Missing results are
checked again before their guarded write, so a file restored during the check is
not incorrectly marked absent.

References live in `<database filename>.file-hashes.sqlite3`, independently of
main schema v11 and the import-recovery store. This derived store has its own
version 1 and uses source keys bound to path, legacy fingerprint, size, page count
and recorded date. Missing state is excluded from that key, allowing the previous
reference to verify a restored source. Unknown cache versions are preserved and
rejected; malformed digest reads are bounded.

At most the current and immediately previous source keys are retained per document.
The previous key survives an interrupted main transaction. Both reference insertion
and older-key pruning share an auxiliary transaction; a cancelled main write leaves
the prior reference available. Deleted-document pruning and a dedicated hash-cache
cleanup control remain pending. These references are not added to legacy v9 backups.

## Work and source ownership

One file-check worker may run at a time. It uses the same guarded file reader as
explicit two-file comparison and one governed 64 KiB buffer. Windows read handles
deny concurrent write/delete access while hashing and applying the result. Paths,
sizes and dates are checked again, and late repository writes require the original
file snapshot and reference. Cancellation is checked before the guarded write and
before its main commit. Shutdown cancels and joins owned work.

The shared catalogue source type now serves both metadata-page evidence and file
verification. It replaces the metadata-only file type without a compatibility
alias. Source-date normalization can expire older analysis evidence; reanalysis
regenerates that evidence while retaining manual values. Manual cover selection
continues to use its saved page. No PDF source is an output target.

## Verification

The existing comparison test was extended without adding test cases. Its file-check
flow verifies persisted references across reopening, repeated timestamp-only
changes, a maximum of two stored keys, changed bytes after the first 64 KiB with
the same size/date, missing/restored paths, wrong path types, stale catalogue
rejection, a path restored before the missing reply, cancellation, unchanged source
bytes and release of governed buffers. It checks preserved title/author overrides,
favorite state and reading position.

Both retained comparison tests and both native metadata-source tests passed.
Workspace formatting, Clippy with all targets and warnings denied, and the locked
native build passed. The minimal suite stays at 53 tests; this checkpoint ran four
related tests rather than the full suite.

Four [native captures](../screenshots/rust-file-check) drive the real command and
reply route: first reference, missing source in Spanish light mode, changed source
and restored availability with 200% text. Each reports its outcome in the control
record. The harness uses an owned synthetic copy and scrolls the inspector to the
result; normal inspection remains user-scrollable. Images bind to executable and
source hashes. Personal PDFs, databases and full hashes are not published.

## Remaining scope

Checks are explicit and per document. Bulk library scans, replacement-version
review, relocation, automatic full hashing during import, aggregate cache budgets,
GUI hash-cache cleanup, ACL/network/mapped-write fault scenarios, physical DPI,
Narrator and measured large-file I/O latency remain pending. Hashing does not
certify PDF syntax or replace native PDF rendering checks. No finished Rust release
or complete file-management parity is claimed.
