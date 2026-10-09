# Connected reader bookmarks — 2026-10-09

The reader's Navigator control now opens a panel with Bookmarks and Text tabs.
Bookmarks are loaded from the active document through the application actor.
The panel supports saving the current page, editing a label, removing an entry
and navigating to its page through the existing canonical reader navigation.
The repository returns pages of at most 80 entries; previous/next controls expose
remaining entries. Entries outside the actual open PDF's page count remain
visible for editing or removal, with navigation disabled and an explanation.

Saving a page uses a new `Ensure` operation. An existing document/page bookmark
retains its ID, label and both timestamps exactly. This prevents an unnamed save
from erasing a previously named passage, including when it is outside the current
list page. Explicit label editing uses the existing transactional rename command.

Request identity and document identity guard asynchronous snapshots. Changing
documents cancels a pending read but does not cancel an accepted write. A stale
response cannot replace the current document's list. Failures keep label drafts
available for retry; saving or removing a different bookmark preserves an open
draft. Text input and write controls are disabled while an edit is in flight.
Submission failures leave the panel recoverable through Reload. A write failure
reported after leaving its document is also surfaced by the application's error
notification rather than being silently hidden.

Wide windows use a resizable panel. Below 620 logical pixels the same content
becomes a full-width scrollable drawer, with a Close control; choosing a passage
closes that drawer to reveal the reader at its destination. Text extraction works
in both arrangements. This extends the existing palette, typography, controls
and finite hover/focus feedback without introducing a new visual identity.

Four UI state regressions verify stale results, independent write lifetime,
draft preservation after failure and draft preservation after an unrelated
successful write. A new SQLite regression verifies idempotent page saving with
exact preservation of a named bookmark. Together with the preceding repository
and actor tests:

```text
cargo test --workspace --locked                          137 passed
cargo clippy --workspace --all-targets --locked -- -D warnings  passed
cargo fmt --all -- --check                               passed
```

Native visual review used three isolated synthetic cases: dark English, light
Spanish and a 640×480 requested window with enlarged text. The latter capture is
800×600 physical pixels at 2.5 pixels per logical point, corresponding to a
320×240 logical viewport. Its entries are reached by scrolling; this is not a
claim that the entire list fits above the fold. The first review found an
icon-only save action and excessive compact heading space. One correction batch
added the visible page-specific action label and prioritized it in the compact
drawer; the second and final review confirmed all three cases.

The capture script records image/control hashes and source hashes in the ignored
`build/rust-visual/bookmarks-confirmation` output. The subsequent change to
late-write error notification changes no layout; those captures evidence the
panel presentation, not that later error path. Synthetic PDFs, databases,
executables, local captures and build logs are not published by this checkpoint.

Remaining scope includes immediate bookmark FTS refresh, wider interactive
acceptance, document outline and thumbnail navigation, in-document search,
links, per-tab navigation contexts and other rewrite acceptance gates. This
checkpoint does not complete the full navigator or authorize a release claim.
