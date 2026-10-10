# Reading session lifecycle and persistence barrier

The application now owns a bounded reading-session lifecycle independently of visible documents and PDF cancellation. It measures activity using the existing monotonic policy, permits only one active reading session, and retains completed sessions until their finish acknowledgements succeed. Completion freezes its timestamp, page, duration and context before the next document starts.

Closing while the start acknowledgement is pending does not discard the finish. Replies must match the session, document, operation and cancellation identity. Failed admission, cancellation and storage errors retain the original payload and require explicit retry. A new retry receives a new token, so late replies cannot acknowledge it. Invalid end timestamps or context leave the live session intact. At most 32 uncompleted sessions are retained; reaching that limit rejects another start instead of deleting pending data.

Recovery collection produces immutable start/finish pairs only after all active activity has been finalized. These pairs can be replayed when an acknowledgement was lost, using the existing idempotent storage operations. The runtime exposes a blocking persistence barrier for shutdown/recovery through its bounded control channel. It runs SQL on the use-case worker and acknowledges completion directly, independently of the ordinary UI event reservations. It does not wait for PDF tickets or require another worker. A failure reports the first error while still attempting the remaining commands; this is not an atomic transaction across several sessions. The caller must retain the original lifecycle for retry when any command fails.

## Verification

Four lifecycle regressions cover:

- Closing before start acknowledgement, opening the next session, preserving the old closure, excluding hidden intervals and keeping the two durations independent.
- Failed queue admission, cancellation, explicit retry, immutable closure values, rejection of wrong document/operation replies and rejection of old tokens.
- Invalid context/backwards wall time preserving live activity, refusal of overlapping active sessions, and full bounded retention without eviction.
- A real SQLite context trigger failure, recovery replay twice, explicit retry and successful final acknowledgement without duplication.

A fifth integration regression saturates ordinary actor admission with a native PDF open held. Two persistence barriers finish before any UI events are drained or that native request is released. An independent SQLite connection confirms one completed row with the expected start/end pages and the 30-second idle-limited duration. No additional native work is submitted.

Formatting, warning-denied workspace/all-target Clippy and all 336 locked workspace tests pass. The complete test output is local in `build/verification/reading-session-lifecycle-tests.log`. The final review moved the session identity rule into the domain so lifecycle and SQLite cannot diverge.

## Remaining integration

The reader does not yet instantiate this lifecycle. Connecting recording consent, real input/visibility events, live reader context, navigation/finalization, retry controls and the exit barrier remains pending. Crash recovery storage and process-interruption acceptance are also pending; an in-memory retained payload is not crash durability. Session history, trim/cleanup, goals, recently closed documents and restoration remain open. No native capture was generated for this application-layer checkpoint. The complete rewrite and Windows Release gates remain unfinished.
