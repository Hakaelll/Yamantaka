# Native smart library views

The library's State menu now includes the eight smart views from Flutter:
in progress, not started, completed, annotated, bookmarked, missing, recently
opened and recently added. The same menu remains inside the compact filter
popover on narrow windows. Both menus scroll within the viewport, including
at 200% text. The active view names the existing control; no new permanent bar
or navigation rail is introduced.

No smart view clears the separate reading-state, collection, text, favorite or
file-state predicates. They combine through AND in the existing catalogue query.
No smart view, Reset filters and All books remove the smart-view selection. Its
stable key is saved in the existing versioned atomic preferences and recognized
keys are restored on construction. Unknown keys fall back to no active view.

## Query semantics

The enum selects static SQL fragments and every data value remains bound.
Annotation and bookmark views use EXISTS, so several records on one PDF return
one document. The smart progress views preserve Flutter's 0.995 completion
threshold. The separate reading-state filter retains its existing threshold of
1.0; these are distinct persisted concepts in the connected Flutter query.

Recent views use the last 14 days. One reference instant belongs to each
LibraryQuery and is shared by its page and count. Each refresh obtains a new
instant. SQLite compares parsed dates with the bound UTC cutoff, including
stored UTC offsets, rather than comparing differently formatted timestamp
strings lexicographically. Empty and unparseable dates do not match. Future
timestamps remain eligible, as with Flutter's lower-bound-only predicate.
Reference times outside the supported range return a recoverable query error.

The existing 80-document page cap and derived count remain in force. No full
membership map, search index, background watcher or PDFium call is needed for
these views. Large-catalogue date-filter latency has not been measured here.

## Executed verification

The retained bounded-query compatibility case now exercises all eight views
against literal expected document IDs and counts. It checks values immediately
below and at 0.995, separate 1.0 reading completion, duplicate annotation/bookmark
records, exact 14-day inclusion, one-second exclusion, equivalent UTC offsets,
empty/invalid dates and future dates. One-row paging returns two distinct
bookmarked documents. Favorite, author, text and collection predicates combine
with the smart view. Enum keys round-trip and unknown keys are rejected.

The targeted storage case passes. Cargo formatting, workspace all-target Clippy
with warnings denied and the offline native development build pass. No test
case is added; the reduced suite remains at 54 cases. The full workspace suite
and performance benchmarks were not rerun.

[Three native synthetic captures](../screenshots/rust-smart-views) verify an
annotated view, a Spanish light/reduced-motion recent view and the compact menu
at 200% text. Two marks and two bookmarks belong to one synthetic document;
recent-date fixtures distinguish it from the older documents. Each result has
one document. The compact capture scrolls the actual menus to the selected view;
its control rectangle lies inside its clip. The result counter uses the correct
singular form in both languages. Capture source and executable hashes match.
Only synthetic fixtures and screenshots are published.

## Remaining acceptance

Physical mouse/keyboard/Narrator and monitor/DPI checks, full preference restart,
large catalogue timing and paired Flutter/Rust performance remain pending. Tag
management, multi-document operations, grouping, collection reordering and the
rest of the rewrite/release scope are still incomplete. This checkpoint does not
claim final library parity or release readiness.
