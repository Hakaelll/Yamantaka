# Reviewed migration and capture verification

Date: 2026-10-21. Scope: the storage contract for an independent Flutter import. This checkpoint does not connect a native migration wizard or activate an imported library.

## Problem and behavior

A regression check reproduced two false successes in the previous capture verifier: editing the copied preferences file, or deleting it entirely, still passed verification. Missing preferences fell back to defaults, and the receipt recorded no hash for the converted Rust preferences.

Receipt version 3 records the exact converted preferences hash, deterministic SQLite schema definitions, and logical fingerprints of every copied table, in addition to the captured database file hash and original-data preservation fingerprints. Verification requires all of them. It checks WAL-visible records in a read transaction, so an unchanged main-file hash cannot hide different records in a WAL. Fingerprints include column names, typed values, row order and column count. Source schema definitions also bind reviews and repeated imports. Root-page numbers are excluded because physical page placement can change without changing the schema. Original-source fingerprints prove preservation before regeneratable cache references are cleared. A second preservation check runs after cleanup, excluding only the two disposable search tables and the intended thumbnail columns; inherited triggers cannot silently change titles, bookmarks or other retained data during cleanup.

Version 1/2 receipts cannot prove these stronger properties. Verified retry refuses them and retains their existing directories. It neither upgrades their receipts nor overwrites those copies. Opening an existing Rust library does not depend on its migration receipt version. A newly reviewed import into another empty destination remains available through the storage API.

`review` returns a frozen `MigrationReview` with canonical source paths, the schema, per-table counts and a preference count. Its private fingerprints bind those visible facts to the source data. `capture_reviewed` rechecks the schema, records, columns and preference bytes inside the source read transaction before its WAL-aware snapshot. A changed source returns the typed `MigrationSourceChanged` error and requires another review. Flutter must be closed before inspection and around capture. Preferences are parsed from the exact byte buffer hashed, rather than reread during parsing.

Snapshot output is reserved with exclusive file creation before SQLite opens it, preventing a concurrent creator from being overwritten. SQLite backup advances in 128-page steps and checks cancellation before and after each step. Copy progress reports copied and total database pages; validation and installation have separate phases. Cancellation during copying, validation or the final installation callback leaves no imported directory. A destination appearing at the installation boundary is retained. The final Flutter/process and preference check runs after that progress callback and immediately before the final destination guard and directory rename.

Source database and preference contents are opened for reading; original PDFs are never opened by this migration code. Normal SQLite WAL access can coordinate through its sidecars. Byte-identical WAL coordination files are not promised. The independent destination is installed only after validation and a receipt write. Existing Rust destinations are never silently replaced.

## Evidence

The retained `wal_capture_includes_committed_frames_and_never_overwrites` case first failed with `(true, true)` for changed and missing copied preferences, then passed after the verifier fix. No standalone Rust test was added. The source suite remains at 55 tests.

That retained case now also invokes an isolated file-backed migration check covering all eleven source schemas; frozen review counts; changes to source schema, column names, values and preferences; an actual partial 128-page copy of a multi-page database; cancellation in all three phases; no leftover staging directory; unchanged repeated import; Flutter reopening at installation; and preservation of a destination created at the last boundary. A WAL mutation check supplies a matching current main-file hash and still expects logical verification to reject the changed records.

Independent source review covered review, capture, retry, verification, shared snapshot callers and cancellation cleanup. Its follow-up identified WAL-only schema changes and cleanup-trigger side effects. A focused reproduction returned `(true, true)` for those two incorrect successes before their guards were added; the corrected case rejects both. The WAL schema scenario confirms the main-file hash remains unchanged after the index is created. The source schema is now read inside the transaction, and logical fingerprints include column names.

Final verification passed: `cargo fmt --all --check`, offline Clippy across all workspace targets with warnings denied, all 55 retained workspace tests, and the native application build. These are storage and integration checks; this checkpoint adds no migration-screen screenshots.

## Remaining work

Native source selection, a user-facing summary and confirmation, migration worker ownership, imported-library activation, process-kill recovery, real Flutter-installation capture and historical executable exports remain pending. The existing standard process probe recognizes the Flutter executable name; renamed or concurrent application behavior needs physical acceptance. No release or complete migration claim follows from these storage checks.
