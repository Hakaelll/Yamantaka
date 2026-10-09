# Reader ownership during replacement — 2026-10-09

A failed replacement opening previously moved the native pin to the attempted
document, leaving the existing reader vulnerable to background eviction. The
actor now protects the confirmed reader together with its pending replacement.
Failure restores the confirmed reader's pins. Two confirmed visible documents
reject an additional foreground document before native admission.

Native completion and UI acceptance are distinct. `Opened` retains both owners
until `ConfirmOpen` identifies the exact document and request token. Cancellation
after delivery restores the previous reader. An unrelated or stale confirmation
is an idempotent no-op. Opening revisions prevent an older completion from
restoring pins over a newer request. Shutdown cancels a delivered but unconfirmed
opening.

Later openings wait for acceptance of a non-cancelled delivered result. This
prevents a replacement from overtaking the document the UI just accepted. Imports
and index extraction also wait for the foreground ownership handoff; catalogue
queries remain responsive. Deferred imports retain their original reservations.
Readiness comes from commands and native ticket channels rather than idle polling.

Confirmation uses the bounded control reservations shared with Close, so a full
ordinary result queue cannot prevent it. The UI coalesces confirmations to the
latest delivered open and submits after draining results. Typed `QueueFull`
backpressure retains that single confirmation for a later finite retry. Opening
and rendering use independent cancellation lifetimes, so applying reader
preferences cannot cancel an accepted ownership handoff.

## Executed evidence

- `cargo test --workspace --locked`: **124 tests passed**.
- Application tests total **24**: seven lifecycle tests, six index-controller
  tests, ten SQLite actor tests and one real PDFium ownership test.
- The real native regression opens and confirms a reader, attempts a protected
  replacement without a password, then pressures both hot slots with two
  background opens. The original reader still yields text; the older background
  document is evicted. Both original PDF byte sequences remain identical.
- Controlled native tickets cover incorrect request identity, cancellation after
  delivery, older completion after a newer admission, a later opening awaiting
  acceptance, all 128 ordinary reservations being retained, deferred import with
  an intervening catalogue query, shutdown and index resume only after acceptance.
- Workspace all-target Clippy with warnings denied, formatting and scoped diff
  checks pass.

Native smoke captures additionally exercise ordinary reader opening, opening page
2 from global search and unlocking a protected document through the connected UI.
These are functional smoke runs in the ignored verification directory; the
previous published screenshots remain evidence of their own recorded checkpoints.
No visual redesign or new claim of complete reader interaction acceptance is made.

The complete rewrite, protected-file import, broader ownership stress, full
memory accounting, device recovery and remaining parity gates are still open.
