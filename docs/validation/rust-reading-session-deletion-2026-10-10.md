# Individual completed-session deletion

The reading journal now exposes Remove session beside its restoration action. The command freezes the selected session/document identity and routes through the bounded actor. Only a matching completed session can be deleted; an unfinished session or a different document scope is left untouched. Repeating a completed deletion is a no-op.

SQLite removes the session and its saved context in one transaction. Context pruning is shared with history retention, preserves unrelated entries and JSON types, and matches historical IDs as keys without constructing JSON paths. Malformed root metadata produces a recovery error. Cancellation before commit, a failing delete trigger, or an ignored delete rolls back both row and context. Retention now checks its actual deletion count too, preventing an ignored row from losing its context.

The interface allows one pending deletion, cancels superseded reads and waits for the exact scoped reply before refreshing. Changing selection or leaving the destination does not alter or cancel the admitted write. A failure releases the write slot for retry and keeps the resident page; a failure after departure appears in the application error surface. Successful refresh returns to the prior page when its last resident session was removed. A success message appears only for an effective deletion. Loading controls are disabled while the write is pending. Restore/Remove actions share a wrapping row so both fit in the compact 200% composition.

## Verification

Four SQLite regressions cover unusual quoted/Unicode IDs, wrong-document and unfinished-session protection, repeated deletion, preservation of all unrelated JSON types, absent context metadata, cancellation, malformed metadata, late abort/ignore triggers and retention rollback when a delete is ignored. One actor regression checks cancelled, wrong-scope, successful and repeated deletion replies while a native PDF opening is held, with independent SQLite reads and no additional PDF requests. Two presentation-state regressions check frozen write identity, unrelated tokens/scopes, departure without cancellation, stale-read cancellation and retry after failure.

All 365 locked workspace tests pass. All 76 executable tests were rerun after the final compact-control/status adjustment. Formatting and warning-denied workspace/all-target Clippy pass. Evidence is `build/verification/reading-session-deletion-tests.log` and `build/verification/reading-session-deletion-ui-final-tests.log`.

Five native DX12 captures cover dark English, light Spanish, compact 200% text and successful deletion in both locales/themes. Deletion captures use separate file-backed synthetic stores. An independent read-only SQLite connection verified 180 completed sessions, absence of `synthetic-session-180` and its context, and preservation of the unfinished `synthetic-live` row. The captured page contains 80 resident entries, no pending read/write, and the successful outcome. The preview invokes the same deletion command as the visible control; physical input is not simulated.

Final manifest: `build/rust-visual/history-delete/capture-manifest.json`. Executable SHA-256: `87016EC758E9E58DE8385DC40D2F7B802E574186FB5006255AC7DE4C8560E7F3`. The executable, lockfile, all recorded UI sources and five PNG/control pairs were independently checked. All five processes exited successfully. Generated evidence and synthetic stores stay outside the public checkpoint. Capture stores have separate instance identities from the demonstration window being reviewed.

## Remaining acceptance

Physical activation/focus/scrolling, native failure/retry presentation, deletion while leaving/reentering, concurrent external changes and last-row pagination interaction remain pending. Category cleanup, goals, recents/closed documents, startup/crash recovery, workspaces, broader restoration acceptance and the complete Release gates remain open.
