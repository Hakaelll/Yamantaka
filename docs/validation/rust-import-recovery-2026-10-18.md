# Explicit recovery of interrupted PDF imports

Import choices and confirmed work now survive a restart of a file-backed Rust
library. Tools exposes **Pending imports**, with Resume and Discard for each job.
Startup only lists pending work. It does not scan folders or open PDFs until the
reader chooses Resume. Cancel keeps the job; Discard removes only its recovery
records, preserving the books already saved.

## Persistence and ordering

The main library remains schema v11. Recovery uses a separate SQLite sidecar
named `<database filename>.imports.sqlite3`, with its own schema version 1. The
repository owns one lazily opened connection. In-memory demo libraries retain the
same behavior only for that process; the recovery captures use isolated actual
SQLite files.

The actor persists import intent before native admission or deferral. A job stores
either explicit absolute file paths or a selected folder root. Each successful
main-library commit is followed by a recovery checkpoint. A checkpoint records
its document identity, normalized path, legacy fingerprint and page count.
Resume skips it only while the corresponding catalogue record still matches.
Completed file selections yield after at most 64 skips so queued reader commands
can run. Folder traversal retains its one-path demand and native close barrier.

The main commit and auxiliary checkpoint are separate transactions. If the
process stops between them, replay sends the file through the normal importer;
its canonical-path identity preserves the existing document and user decisions.
A job is removed only after successful exhaustion. Errors or cancellation retain
its source and checkpoints. Resume/discard admission respects active ownership;
a cancelled native import finishes closing before recovery exposes its snapshot.

The close barrier persists queued intents even before they begin. Runtime disposal
also drains those intents before stopping owned native work. A close-flush failure
reports a redacted native error instead of claiming that every choice was saved.

## Bounds and compatibility

There may be at most 80 pending jobs. Explicit selections allow up to 100,000 paths
and 4 MiB of path text; their serialized payload is bounded at 10 MiB. Folder jobs
store only their root, so they do not materialize a complete tree in memory.
Summaries and checkpoint fields have bounded reads. SQL uses parameters and the
existing WAL/full-synchronous configuration.

Unknown recovery versions and unrecognized schemas are preserved and rejected.
Opening the main library does not depend on opening the recovery store. Recovery
errors stay in Tools; importing new files requires saving their intent successfully.
The sidecar is operational state and is not added to the inherited v9 backup
format. Broader backup/recovery packaging remains part of final acceptance.

A checkpoint describes work already committed. It does not revalidate the physical
PDF after external changes. Missing-file detection and source revalidation remain
separate pending work. Resuming a folder scans its current ordinary subtree;
files subsequently added there may be included, while links stay excluded.

## Verification

The existing folder-import test was extended without adding test cases. It now
reopens actual main/sidecar files and verifies:

- One confirmed document remains paused on startup; explicit Resume imports only
  the other two and retains its identifier.
- Replaying the gap before a recovery checkpoint preserves title overrides,
  favorite state, reading position and all three document identities.
- Discard keeps saved documents; cancelled admission retains a resumable intent.
- The immediate-close flush persists a request that has not entered native work.
- An unsupported sidecar version is rejected without changing its hash, while the
  main v11 library still opens and retains its documents.
- Original PDF hashes and tracked native-resource release remain intact.

All eight retained native import/reanalysis preemption regressions passed, along
with the retained historical-schema check covering versions 1 through 11. Workspace
formatting, Clippy with all targets and warnings denied, and the locked native
build passed. The minimal suite stays at 53 tests; this checkpoint ran the ten
related tests rather than the full suite.

The [native captures](../screenshots/rust-import-recovery) exercise a reopened
pending job, its explicit resume, its discard and the pending menu with 200% text.
The first resumed capture exposed a reused process-ID collision with an old
synthetic database. Capture and isolated-preview stores now include a fresh UUID
in addition to the process ID. The harness opens the ordinary submenu through
egui menu state. It uses only
synthetic PDFs and actual repository commands. Control records distinguish the
pending ten-book library, the resumed eleven-book library and the discarded
ten-book library. Manifests bind the images to source and executable hashes.

Forced process termination, power-loss and disk-fault injection, native pointer
and keyboard operation, physical DPI, Narrator and large-queue performance remain
unverified. This checkpoint does not establish full file-management parity or
completion of the rewrite.
