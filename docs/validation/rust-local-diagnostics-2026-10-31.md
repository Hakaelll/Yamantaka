# Local diagnostics checkpoint

Date: 2026-10-31. Scope: read-only diagnostic collection, native review, explicit copying and atomic JSON publication. This does not establish full recovery, privacy, migration or release acceptance.

## Connected behavior

Settings → Privacy opens a native diagnostic review. A dedicated worker opens the active database read-only and collects a single transactional snapshot of schema, logical page size/count, eight library record counts, a SQLite quick check and foreign-key status. The application report adds compile-time application/platform/architecture labels, renderer and existing governed memory counters/limits. Database and memory snapshots are separate. The report names the limits of memory coverage.

The report has typed fields with no document identifiers, names, paths, query/note text, passwords or raw errors. It never serializes preferences, catalogue records, native error strings or arbitrary diagnostic maps. Collection does not load PDFs or extract their text. The existing global memory ledger is sampled; this is not a process-memory benchmark.

The modal shows a summary and expandable complete JSON. Refresh keeps the last successful snapshot available after failure. Copy requires an explicit button click. Save uses the Windows JSON selector and an independent worker. Existing annotation export publication was renamed to a shared local-export operation and its caller migrated. Diagnostics reuses its same-directory temporary file, flush/sync and no-replacement publication. Registered source paths and the library folder are rejected as destinations. No automatic report transmission was added.

Cancellation sets the request token and interrupts the worker's SQLite connection. Closing the review requests cancellation. Worker destruction cancels and joins. A completed but unconsumed collection reply is discarded after cancellation; an already published save still reports success. The UI polls results and requests redraws on completion rather than running SQL, file traversal or publication during painting. A committed report remains complete if cancellation arrives after publication. No original PDF is modified.

## Executed evidence

The retained storage compatibility test `overrides_win_and_queries_are_bounded_and_escape_wildcards` passed with a diagnostics helper: one passed, five filtered, 5.38 seconds. The helper inserts a private-content canary into document metadata and a path, verifies expected numeric counts and schema/check results, and confirms the serialized database snapshot excludes it. It also checks exact JSON publication, occupied-file preservation, pre-cancelled collection/publication and SQLite integrity. No standalone test was added. An unused mutable binding reported by that run was removed without changing its behavior.

Offline native build and binary Clippy with warnings denied passed. Five synthetic native scenarios passed their state and control-bound assertions:

| Case | Observed result |
| --- | --- |
| diagnostics-saved | Collects ten synthetic documents, reviews the typed report, publishes JSON and reads back the exact payload; script checks expected counts and absence of document/path content |
| diagnostics-occupied | Keeps a pre-existing complete report byte-for-byte and presents a recoverable occupied-file message |
| diagnostics-cancelled | Cancels the background request without publishing a report snapshot |
| diagnostics-unavailable | Presents a redacted failure for a missing read-only source and retains no false success |
| diagnostics-small-200 | Shows the review in a requested 640 × 480 logical window with 200% text, light theme and reduced motion; Close remains accessible and the body scrolls |

A repeat initially encountered the report produced by the previous capture, and the application correctly refused to replace it. The capture script now recognizes its two synthetic report fixtures and removes only those exact generated outputs before repeating the scenario.

The final dark saved view, light occupied-file state and enlarged compact view were visually inspected. Captures use only isolated synthetic databases. Source files, Cargo.lock and frozen executable hashes are checked against the [capture manifest](../screenshots/rust-local-diagnostics/capture-manifest.json) before publication. Published PNGs embed origin metadata. No Computer Use was used.

## Remaining acceptance

The native scenario checks cancellation at request start; interruption during a long SQLite check, late publication races, close/reopen timing under load, real clipboard and file-selector interaction remain pending. Physical keyboard/focus, Narrator and monitor DPI acceptance are not established by these captures. The quick check is not full SQLite integrity verification. Cache/disk inventory, all Flutter performance/lifecycle diagnostic fields, safe mode, quarantine, startup recovery and the complete Windows release remain required work. No broad workspace suite was repeated for this checkpoint.
