# Atomic reading session storage

The domain now defines borrowed start/finish commands and the legacy camelCase reader-session context. SQLite keeps the existing `reading_sessions` table and `reading_session_contexts_v1` metadata key. New writes normalize validated timestamps to UTC, retain start/end pages and cap active seconds at 24 hours.

Starting the same command again preserves its row. An occupied ID with a different document, start time or page is rejected. Finishing validates both session and document identity, chronological order and finite reader context. Repeating the same finish row values is accepted; conflicting completed row values are rejected. New IDs accept bounded ASCII alphanumeric, hyphen and underscore identifiers, including the legacy SHA-256 shape and UUIDs, so metadata JSON paths remain unambiguous. Existing historical rows are not rewritten to this policy.

Session completion and context storage occur in one transaction. SQLite JSON operations merge known context values into the existing object while preserving other session entries and unknown fields. The implementation does not load the entire metadata map into a Rust JSON tree. Invalid existing JSON or a non-object context produces a recovery error instead of silently replacing the user's metadata. A late metadata failure rolls the session row back as well.

## Verification

Four new storage regressions cover:

- Retryable start/finish, UTC storage, canonical context fields, unrelated metadata and nested unknown-field preservation, and rejection of conflicting session identities/completed values.
- Malformed metadata preserved without completing the session; an injected metadata trigger failure after the session UPDATE rolls back the entire transaction, and removing the trigger allows recovery.
- Cancellation before start, nonfinite zoom, backwards end time, wrong document identity and the 24-hour persisted limit.
- Each historical schema v1-v11, populated with a completed legacy session before migration. A new session can be recorded afterward, and an independent SQLite snapshot verifies every legacy row field remains unchanged.

Formatting and warning-denied workspace/all-target Clippy pass. All 330 locked workspace tests pass. The result is recorded locally in `build/verification/reading-session-storage-tests.log`.

## Pending integration

The repository does not yet record sessions from reader events. Application actor routing, lifecycle finalization, history/context loading, trim/cleanup, goals, recently closed documents, restoration and native UI acceptance remain pending. Process-level interruption and broader resource/latency acceptance remain open. The full rewrite and Release gates are incomplete.
