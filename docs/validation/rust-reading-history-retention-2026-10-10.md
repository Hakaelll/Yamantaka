# Reading history retention

Reading-history retention now removes the oldest completed sessions and their context entries in one SQLite transaction. Unfinished sessions are excluded regardless of their age. The latest completed rows are ordered by stored start time and ID, making equal-timestamp retention deterministic. The limit remains bounded to 25-1000; the imported setting is respected and defaults to 250. Invalid future preference values remain preserved rather than rewritten.

Context cleanup uses SQLite JSON operations without constructing the complete metadata map in Rust. Only keys belonging to explicitly removed session rows are deleted. Unmatched entries and unknown fields retain their JSON types, including nested objects, arrays, booleans, nulls, strings and numbers. Matching keys directly through `json_each` also handles unusual historical IDs without generating ambiguous JSON paths. Malformed/non-object root metadata produces a recovery error before any deletion. A late DELETE failure rolls back the metadata change as well.

The actor routes retention commands and returns their cancellation identity and result. Accepted session start/finish acknowledgements schedule cleanup. The shutdown persistence barrier now takes the history limit explicitly and trims only after every recovery write succeeds, before acknowledging the barrier. Empty recovery batches retain the previous behavior and do not implicitly prune history. The limit does not cap the number of unfinished sessions.

Privacy exposes an editable retention limit with a labelled Apply button. Editing the draft does not prune history. Apply atomically saves the preference and dispatches cleanup; it remains available to retry the same limit. The existing recording switch and retention input share one preference-save rollback implementation. Their explanatory copy describes removal of old completed sessions and preservation of open readings.

## Verification

Four storage regressions cover atomic pruning, unusual historical identifiers, every JSON value type, no-op retries, unfinished rows, malformed metadata, a late DELETE-trigger failure, cancellation, the upper bound and imported preference preservation. One additional actor regression checks normal/cancelled retention replies and their request identity without submitting PDF work.

The existing saturated-admission/PDF-held recovery test now starts with 300 completed historical rows and one old unfinished session. Two recovery barriers leave exactly 25 completed rows plus that unfinished session, preserve the recovered session's page/duration, remove retired context and retain the open session's unknown metadata. An independent SQLite connection verifies the result before the held native request is released or UI events are drained.

All 343 locked workspace tests pass. Formatting and warning-denied workspace/all-target Clippy pass. All 69 UI tests were rerun after the final control adjustment. Local logs are `build/verification/reading-history-retention-tests.log` and `build/verification/reading-history-retention-ui-tests.log`.

Five final native wgpu captures in `build/rust-visual/reading-history-retention-final` cover dark English, light Spanish/reduced motion, compact 640x480 with 200% text, and enabled/disabled reader recording. The review replaced an ambiguous icon-only Apply control with visible action text. The compact recording control remains visible; the retention input and explanatory text are reached by scrolling. Capture hashes and enabled/disabled SQLite recording are checked against the local manifest. The final executable SHA-256 is `ED927D8A19C37B33A2EE0279F0BD7F01AA651C0777FF79234250A5C8673E6854`.

## Remaining work

Physical keyboard editing/Apply/retry, compact scrolling and injected native retention failure acceptance remain pending. Large metadata/history latency and memory acceptance, interruption during pruning, full history presentation, goals, recents/closed documents, restoration, crash recovery and the broader continuity requirements remain open. Previously orphaned or unrelated context keys are preserved; category cleanup is a separate pending feature. Native text zoom does not certify physical DPI, Narrator or multi-monitor behavior. The full rewrite and Windows Release remain unfinished.
