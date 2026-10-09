# Connected PDF search panel

The reader Navigator now includes Find. Ctrl+F opens the query field; Enter or
the adjacent search action starts a cancelable document search. Case-sensitive
and whole-word options use the native query contract. Result rows include page
numbers and bounded text around a match. Selecting a result navigates through the
canonical reader position and preserves zoom. F3 and Shift+F3 move between
matches and wrap across document boundaries.

## Ownership and navigation

The interface has explicit idle, running, ready, stopped and failed states. Each
request carries its document and independent cancellation identity. Replies for
another document, consumer, or cancelled request cannot replace the active result.
Changing the query or options cancels old work and clears its result. Opening a
different document and shutdown release search state. Closing a running search
panel cancels its request; result navigation can run with the panel already closed.

The UI retains one batch of at most 80 matches. Next continues at the first omitted
native ordinal. Previous requests the last 80 matches strictly before the current
batch's start. A request with no preceding boundary searches to the document end,
which supports wrapping to its last match. There is no unbounded UI match cache or
cursor-history list. Backward scans retain a rolling batch on the actor, release
superseded payloads, and stop at an exclusive cursor. They currently rescan earlier
pages; they do not claim constant-time backward navigation or a measured advantage
over Flutter. Those performance checks remain outstanding.

An empty continuation range wraps once to the opposite document boundary, so
blank trailing pages cannot replace an existing search with a false no-match
state. A genuinely empty whole-document search stays empty. Search progress shows
pages examined, and the displayed match count labels its batch scope rather than
inventing a complete document total.

Visible match rectangles use canonical crop/rotation coordinates and scale with
the displayed paper. All four corners follow user rotation. The active match has
a distinct outline and fill. Matches without reliable visual geometry navigate
to their page without inventing a precise anchor. Current highlights cover the
retained batch; showing every occurrence simultaneously and an exact distinction
between absent phrases and documents without extractable text remain acceptance
work.

## Native presentation and tests

The panel inherits the reader's theme, typography, focus and motion treatments.
An inline query/action row keeps Find reachable in compact windows. Result rows
wrap contextual snippets within panel width. The compact drawer remains scrollable
at 200% text zoom. Keyboard semantics are exposed for the field and result buttons;
physical Narrator and DPI certification are not inferred from those contracts.

Two additional application integration tests cover backward rolling batches,
exclusive cursors within and between native chunks, and real PDFium navigation
back from the final synthetic study match. Four UI tests cover stale/cancelled
replies, cancellation on drop, recoverable failures, literal highlight coordinates
across all rotations, and Enter events through a real egui text editor. That event
test detected and corrected the single-line editor's focus release on Enter.

Native captures cover dark English, light Spanish, requested 640x480 at 200% text
zoom, the next match, previous-match wrapping, 90-degree rotation, and wrapping with
the panel closed. Navigation captures invoke the same handlers as the controls;
they do not certify physical mouse or keyboard interaction. Captures and their
control geometry, anonymous find state, source hashes and executable hash are
retained locally under ignored `build/rust-visual/find-final`.

The visual review inspected dark, light and compact together, corrected query
access and result context in one batch, and used one confirmation round. Subsequent
functional lifecycle and Enter fixes refreshed the runtime artifacts without
another aesthetic polishing round.

## Validation scope

All 182 workspace tests, warning-denied Clippy and formatting checks passed for
this checkpoint. Exact counts and logs are retained under ignored
`build/verification/find-ui-final-*`. Remaining gates include broader interactive
search acceptance, index/search stress and performance comparison, all outstanding
rewrite parity, physical accessibility and DPI validation, and the final Windows
Release bundle. This panel does not make the full rewrite complete.
