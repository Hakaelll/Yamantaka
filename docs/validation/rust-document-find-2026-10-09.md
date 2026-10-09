# Document-wide PDF search coordination

The application actor now traverses an open PDF document using the existing
native page-search tickets. A request returns up to 80 matches and a continuation
at the first omitted match. This checkpoint completes the document traversal
foundation. The reader search panel, highlight presentation, result navigation
and shortcuts remain pending.

## Traversal and bounded results

A scan owns one native ticket at a time. Completion schedules the next page
without waiting for a UI frame. The actor still handles commands and other ready
tickets between page completions. Its blocking select sleeps while native work
is pending. Search does not open another document or allocate a second native
engine.

The result batch shares existing governor-backed page payloads and copies neither
snippets nor rectangles. It retains at most 80 nonempty page payloads and exposes
at most 80 matches. A partially included page resumes at the first omitted native
ordinal. Full pages continue at their native cursor or the following page. Empty
pages are not accumulated. A continuation indicates unsearched content; it does
not promise a further match or represent a total document match count.

Progress uses one atomic scanned-page snapshot per request and notifies the UI
once per 16 completed pages. A single terminal event carries the command's
existing capacity reservation. An undrained UI therefore cannot fill the actor's
event queue with one update per PDF page. Scanned-page progress excludes a page
that still has a match continuation.

The coordinator validates result page/cursor identity. Cancellation and native
failures discard the partial batch and retain the original document/cancellation
identity in the failure event. Shutdown uses the same pending-ticket cancellation
path. Native page work limits remain explicit failures; the coordinator does not
silently skip a page or report complete results after such a failure.

## Executed evidence

Four additional integration tests cover these behaviors:

- A batch spanning two pages stops at 80 and resumes at the exact omitted match.
  Consuming the continuation yields all remaining results without duplicates.
- A scan using controlled native-ticket replies traverses 5,000 empty pages without draining UI events.
  Progress generates 312 notifications plus one terminal notification, and the
  final result is empty with no continuation. Page payloads are released.
- Cancellation and a wrong-page result after accepting an earlier page both
  reject the partial batch. A SQLite catalogue query completes while the next
  native ticket is held, and no subsequent page is scheduled after rejection.
- Real PDFium finds study-text matches on all three synthetic study pages and
  searches the 5,000-page fixture to completion for an absent phrase. Both source
  PDFs remain byte-for-byte unchanged, and Text leases return to zero after the
  result owners are dropped.

```text
cargo test --workspace --locked                               176 passed
cargo clippy --workspace --all-targets --locked -- -D warnings passed
cargo fmt --all --check                                       passed
```

These tests verify scheduling, continuation and ownership. Their execution time
is not a UI, startup, peak-memory or paired Flutter performance benchmark.
Interactive cancellation, search presentation, accessibility and physical DPI
acceptance remain pending. Logs are retained locally under ignored
`build/verification/document-find-*`.
