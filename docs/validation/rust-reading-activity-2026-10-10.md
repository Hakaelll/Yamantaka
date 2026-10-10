# Monotonic reading activity

The connected Flutter reader starts a session after opening a document, records time while visible with activity in the preceding 30 seconds, and caps persisted active seconds at 24 hours. Its controller uses a periodic one-second timer. Session history and restoration are not yet connected in Rust.

The Rust domain now provides a monotonic activity accumulator for that visibility and inactivity policy. The caller supplies elapsed time from a session origin, the new visibility state and whether an interaction occurred. It credits only the visible part of the prior interval within the activity window. Late input starts a new window without crediting the intervening idle period. Returning to the reader also starts a fresh window. Fractional seconds are retained internally and the reported total uses whole seconds, capped at 24 hours.

The accumulator has no timer, clock access, thread or repaint request. Reader lifecycle and input events can observe it, and session finishing can settle the final interval. This avoids requiring periodic UI updates solely to maintain an activity counter. It measures elapsed qualifying time rather than reproducing the phase-dependent rounding of Flutter's periodic samples.

## Verification

Four new domain regressions cover long idle gaps followed by input, hidden intervals and return to the reader, fractional/repeated/regressing observations, and the 24-hour limit after a long active session. Formatting and warning-denied workspace/all-target Clippy pass. All 326 locked workspace tests pass. The result is recorded locally in `build/verification/reading-activity-tests.log`.

## Remaining work

This is the timing policy for the continuity implementation. Session IDs, atomic persistence and legacy context storage, history, goals, recently closed documents, restoration, reader lifecycle/input integration and native UI acceptance remain pending. No connected recording or idle-performance certification is claimed by this checkpoint.
