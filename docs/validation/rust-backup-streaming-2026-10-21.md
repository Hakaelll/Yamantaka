# Streaming backup export verification

Date: 2026-10-21. Scope: the storage export and bounded backup-file reader. Native backup export, restore confirmation, and recovery controls remain pending.

The previous v9 exporter accumulated all table rows in JSON values, serialized that full payload to check its size, and serialized it again as an indented file. Its size check therefore preceded the bytes actually installed. The replacement writes compact JSON through a 64 KiB file buffer, holds one database row at a time, and calculates each category hash over that category's emitted JSON array. One SQLite read transaction spans every table. V9 envelope fields, column values, setting precedence, and category hash semantics remain compatible.

Every emitted byte counts toward the existing 128 MiB limit, including the envelope, settings, manifest, and trailing newline. The exporter also refuses tables beyond the reader's existing 100,000-row limit, rather than producing a backup that the reader rejects. Cancellation is checked between rows, during output writes, and after syncing the temporary file before atomic replacement. An error or cancellation discards the temporary output and retains any existing destination. `BackupProgress` reports the table and completed row count for later background-worker integration.

The file preview opens one file handle, checks that handle's metadata, and reads in cancellable 64 KiB chunks with a running byte limit. A file growing after its initial size check cannot bypass the limit. Parsing and the isolated restore preview still materialize the bounded JSON payload and database. This checkpoint does not claim constant memory use, integration with the complete application memory governor, or measured UI latency. The exporter still retains a row, settings, and the small manifest in addition to its output buffer.

## Evidence

All five retained backup tests passed. They cover versions 1–9, populated relationships and historical column projections, category hashes, invalid relationships, safety copies, and interrupted-rename recovery. The existing tampering test now invokes an additional file-backed check:

- Cancel after a bookmark row has been emitted. The result is typed `Cancelled`, the existing destination remains byte-identical, and no temporary file remains.
- Mutate documents and bookmarks through another WAL connection after the first document row. The completed backup retains the pre-mutation document and bookmark values and passes preview hash validation.
- Export 400 synthetic tags containing 64 KiB of null text each. JSON escaping pushes the output beyond 128 MiB. Export stops before all rows finish, preserves the existing destination, and leaves no temporary file.
- Refuse a synthetic table with 100,001 rows while preserving the existing destination and cleaning temporary output.

These checks add no standalone test cases. Workspace Clippy across all targets with warnings denied, formatting, and the native application build passed. A focused independent code review found no actionable issue in hash ordering, snapshot consistency, cancellation, atomic replacement, or size accounting. No broad workspace test rerun or new visual capture was needed for this storage-only change.

## Remaining acceptance

Native background-worker admission, export and restore dialogs, full recovery interactions, real historical executable exports, complete resource accounting, and paired Flutter/Rust performance measurements remain pending. No personal library or PDF was used by these checks. This checkpoint does not establish full backup UI parity or release readiness.
