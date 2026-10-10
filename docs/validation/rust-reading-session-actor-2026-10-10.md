# Reading session actor routing

The application actor now accepts owned session start/finish commands and dispatches them to the existing transactional SQLite repository. Each reply retains the session ID, document ID, operation, cancellation identity and result. Cancellation and storage failures use this scoped reply rather than an unscoped acknowledgement, so the reader can distinguish an unsuccessful start from an unsuccessful completion.

Session persistence runs on the use-case worker. It does not perform SQL on the UI thread, submit PDF work, wait for a pending PDF ticket or create another worker. Commands and replies use the actor's existing bounded admission reservations. The new repository requirement is explicit in the actor's generic contract; adapters cannot silently report unsupported persistence as success. The presentation event dispatcher surfaces failures and suppresses cancellation errors. Reader lifecycle matching and automatic recording are still pending.

## Verification

One integration regression uses a temporary file-backed SQLite database, an independent SQLite read connection and a deliberately held native PDF open. Before releasing that open, it verifies:

- A cancelled start returns the same request identity and creates no row.
- Two identical starts return scoped success and create only one row.
- Two identical finishes persist start/end pages, active seconds and reader context.
- A conflicting completed duration returns a scoped validation failure and preserves the previous duration.
- The independent connection reads the committed page, duration, scroll and panel values.
- Session commands submit no additional native PDF requests.

The existing storage regressions continue to verify transaction rollback, context extension preservation, validation and historical schema compatibility. Formatting, warning-denied workspace/all-target Clippy and all 331 locked workspace tests pass. The complete test output is local in `build/verification/reading-session-actor-tests.log`. SQLite was added as a test dependency using the already locked workspace package; no package versions changed.

## Remaining work

This checkpoint does not activate recording in the reader. Consent/privacy settings, lifecycle start/finalization and retry ownership, monotonic activity integration, shutdown completion, history/context queries, trim/cleanup, goals, recently closed documents, restoration and native acceptance remain open. A queued command is not a shutdown durability guarantee: the lifecycle integration must await or safely preserve outstanding completions before stopping the actor. The complete rewrite and Windows Release remain unfinished.
