# Guarded full-content references during import

Every newly inserted native import now supplies a full SHA-256 reference bound
to the source revision used for its PDF metadata. File selection, folder import,
arguments, activation and resumed import jobs share the same actor flow. No new
progress bar or modal is added. Reanalysis of an existing document retains its
previous metadata workflow rather than silently replacing a content reference.

## Admission and source ownership

Registered absolute paths are looked up through indexed normalized keys, including
ordinary and verbatim Windows representations. A known path reuses its document
and recovery checkpoint without starting a hash or reopening PDFium. Manual
metadata and an existing reference remain unchanged. This is path idempotency,
not content deduplication or approval of a changed source. Ambiguous registered
aliases are rejected; unsupported non-Unicode path values are rejected before
lossy text conversion. Unresolved aliases still reach the final canonical lookup.

A new candidate enters a cooperative hash-admission state. At most one importer
hash worker is active across import streams; other streams wait without creating
more hashing threads. A dedicated worker reads the complete regular file through
the shared guarded reader and one governed 64 KiB buffer. It uses a bounded reply
channel and a cancellation token. Pending work registers actual channel readiness
with the actor instead of adding an idle polling loop.

The read-only Windows handle denies new write/delete access. It is acquired before
PDFium opens the candidate and remains owned by the import state during native
opening, page analysis, reader preemption and reopening. The native source stamp
must match the guarded path, size, timestamp and first-64-KiB digest. The same guard
is checked again after confirmed native close, immediately before persistence;
its complete digest accompanies the native revision. Cancellation and failure
release it, and owned workers are joined. The original PDF is never an output.

Hash preparation owns no PDFium slot. Foreground reader work and cover admission
keep their native ownership rules. Reading every byte adds source I/O for a new
candidate; this checkpoint does not claim a measured import speedup or prove
reader-contention latency. The known-path shortcut avoids that work on ordinary
repeated imports and is checked by the absence of new native open requests.

## Storage and interruption

The repository import signature now requires a `FullSourceHash` and cancellation.
Its native revision must equal the analyzed document's revision. The previous
unreferenced import signature is removed; every caller is migrated. Test fixtures
use a shared helper that computes a real digest from quiescent files. Production
uses the held guard rather than that synchronous fixture helper.

Storage retains main schema v11 and inserts the new document and metadata inside
one main transaction. Its derived complete reference is written to the existing
version-1 file-hash sidecar before the main commit. A sidecar error prevents the
main insert from committing. After a reported main failure, an absent new UUID's
reference is removed; an existing main row's reference is never removed by this
cleanup. A repeated canonical path returns the saved ID without resetting its
metadata or digest.

The two WAL databases do not provide an atomic cross-database power-loss commit.
A process death between the auxiliary write and main commit can leave an
unreachable reference with no document row. Global orphan pruning and destructive
power-loss acceptance remain pending. Once a main row commits, a recovery replay
can checkpoint that same row without overwriting its reference or manual choices.
The full-reference sidecar remains derived data outside v9 backups.

## Verification

The retained eight native reader/import/reanalysis tests pass. New-import source
replacement attempts now fail with Windows sharing violation 32 while analysis
is parked, including loss of the initial native owner. The original source,
page evidence and active readers remain intact. Cancellation, repeated reader
preemption, reopened metadata and reanalysis failure behavior stay covered.
Successful imports expose the expected complete digest before any explicit
Check file operation. A repeated import preserves the import revision and issues
no new native open request.

The retained folder/recovery test also passes with an extended native helper. It
checks the guard before native opening, a source-bound digest mismatch, Unicode
and ordinary/verbatim lookup, rejection of an unsupported path encoding,
reference persistence across reopening, retained manual values and write access
after the guard is released. A deferred foreign-key failure occurs at main
commit after the auxiliary digest write; no document or orphan digest remains.
An unknown auxiliary schema version is preserved byte for byte and no document
commits. Governed allocations return to zero after ownership ends.

Four retained metadata-reanalysis tests and two native metadata-source tests pass
after migrating their import calls. In total this checkpoint exercises fifteen
related tests and adds no test cases; the minimal suite remains at 53. Formatting,
all-target workspace Clippy with warnings denied and the locked offline native
build are checked. The full workspace suite and performance gates were not rerun.

[Native synthetic captures](../screenshots/rust-import-hash) import three generated
PDFs and then run the normal Check file command on a newly added document. The
library contains 13 documents and the reply is `available`, not `baseline`: its
reference came from import. Dark, Spanish light/reduced motion and a small viewport
with 200% text are captured. No reference is injected into the harness database.
The PNGs retain their PDF cover text; raw pixel inspection avoids mistaking a
reduced image preview for missing document content.

## Remaining scope

Automatic duplicate candidate discovery/resolution, replacement-version review,
full ownership hardening of every reader/reanalysis path, aggregate cache pruning,
mapped writes, unusual filesystem aliases, forced-process/power-loss faults and
physical picker/accessibility acceptance remain pending. Large-corpus timing and
paired Flutter/Rust measurements still require their own runs. This is an import
correctness checkpoint, not the completed rewrite or Windows release.
