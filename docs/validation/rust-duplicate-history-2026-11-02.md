# Durable consolidation history

Tools → Identical copies now exposes an optional history section. It reads completed native consolidation receipts from disk, newest first, with pages limited to 80 records. Each entry identifies the current kept title and a locally formatted date; retired titles use a neutral fallback. Reviewing an entry validates the backup against the SHA-256 recorded in its journal before entering the existing whole-library restore review.

The query joins merge history to its matching completed native journal and validates version, record identities, backup path and digest agreement. Unsupported inherited merge records do not gain invented native recovery authority. String sizes, page offset and memory are bounded. The read runs on the joined background consolidation worker and does not close reader handles or lock normal command admission. It does not perform a restore automatically.

The retained duplicate application test reopens the database through a new store and checks the committed history identity and backup digest, plus an empty offset page. No new standalone test was added. The first attempted test used the default temporary directory and failed with Windows access denied in an existing file-check fixture; the run with the workspace test directory passed.

Native synthetic cases discard the in-memory receipt, load committed history through a new database connection, then capture the history list or open its backup review. This verifies disk-backed recovery without a session receipt; it is not a full process restart. Both cases assert one history entry, two current records and no in-memory merge receipt. Backup review asserts three books and no queued installation.

Executed: focused retained application test, workspace Clippy across all targets with warnings denied, native debug build, formatting, and two native captures. The earlier [consolidation checkpoint](rust-duplicate-consolidation-ui-2026-11-02.md) records baseline behavior.

[History list](../screenshots/rust-duplicate-history/duplicates-merge-history.png) · [Backup review](../screenshots/rust-duplicate-history/duplicates-merge-history-undo.png)

The manifest records native image bytes; published PNGs add synthetic provenance without changing pixels, with both hashes in origin files. Full process restart interaction, physical keyboard/Narrator/DPI acceptance, large history performance and native restore installation remain pending. Restoring any entry replaces the entire library with that snapshot, including later library edits; the interface states this consequence before confirmation.

## Installation and restart follow-up

[Native backup installation](rust-duplicate-restore-2026-11-02.md) now records the later actual restore handover, table comparison, PDF hashes and separate native process restart. Earlier scope and remaining physical interaction requirements above retain their original meaning.
