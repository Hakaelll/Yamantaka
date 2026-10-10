# Governed reading history queries

The domain now exposes immutable reading-history pages with at most 80 completed sessions, a total count, normalized offset and optional document filter. Records contain the original session/document identities, effective title and path, missing-file state, timestamps, pages, duration and only that session's context object. Unknown context extension fields remain intact in the raw JSON object for the later restoration adapter.

SQLite count, metadata validation and page reads share one transaction. Manual titles take priority over extracted/source titles, with the file name as fallback. Unfinished sessions are excluded. Start time and ID provide deterministic ordering. The SQL materializes only the requested session page before joining context keys, and matches keys directly rather than constructing paths from historical IDs. Missing or non-object individual contexts return no context, matching the legacy history behavior. Malformed/non-object root metadata, duplicate session-context identifiers and invalid session timestamps produce recovery errors instead of returning a partial page.

The shared memory governor reserves CPU text capacity for record descriptors and each borrowed SQLite string before copying it. Final construction accounts for actual vector/string capacities. The lease lives with the immutable page, including when several clients hold its Arc, and is released only after the final owner drops it. Unrelated metadata objects are not copied into Rust. SQLite's opaque internal allocations remain outside this accounting; no broader process-memory bound is claimed.

Owned history requests now route through the bounded application actor. Replies retain the query and cancellation identity, returning an Arc page or a scoped failure. They do not submit native PDF work or wait for a PDF ticket. This checkpoint does not yet expose a history destination in the interface.

## Verification

Four SQLite regressions verify:

- 181 completed sessions plus an unfinished session, 80-row paging, the final 21 rows, document filtering, manual/source-title priority, missing-file state, extreme offsets and lease release.
- Unusual historical IDs, Unicode and nested unknown context fields. A 400 KB unrelated object does not enter the Rust page budget; making the selected context large causes clean budget denial.
- Cancellation, a late oversized document title and invalid UTF-8 after an earlier valid row, all leaving no live history allocation or partial result.
- Malformed metadata, duplicate context keys and a bad timestamp, all failing without a partial page and releasing their leases.

One actor regression holds native PDF opening while reading an 80-row history page. It verifies scope/query/request identity, continued catalogue writes, retention of memory until the last Arc drops, cancellation and budget-denial replies, and absence of extra native requests.

All 348 locked workspace tests pass. The four storage regressions were rerun after strengthening selected-context and invalid-UTF-8 coverage. Formatting and warning-denied workspace/all-target Clippy pass. Local evidence is `build/verification/reading-history-query-tests.log` and `build/verification/reading-history-query-edge-tests.log`.

## Remaining integration

History presentation, selection/source navigation, context interpretation and restoration remain pending. Unknown future context values require explicit adapter handling before changing reader preferences; this query does not silently replace them with known defaults. Larger history/metadata latency and memory acceptance, cancellation during long SQLite work, live concurrent-change acceptance, physical interaction and native history captures remain open. Goals, recents/closed documents, workspaces, crash recovery and the complete Windows Release gates remain unfinished.
