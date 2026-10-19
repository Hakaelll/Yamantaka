# Library multiselection and bulk favorites

Date: 2026-10-20. This checkpoint connects page selection and favorite/unfavorite. It does not complete all library bulk actions or the rewrite.

## Implemented behavior

The domain selection set is separate from the inspector cursor and keyboard focus. Like the connected Flutter library, it retains only identifiers on the current bounded page. Ctrl-click toggles a book; Shift-click selects an inclusive range; Ctrl+Shift adds a range. Selection mode makes ordinary clicks toggle membership. Select this page and Ctrl+A select at most 80 books. Escape clears selection outside text input, menus and modal dialogs.

Focused covers and rows support Space to toggle, arrows and Home/End to move focus, and Shift with those navigation keys to extend selection. Pending focus requests scroll their destination into view, including an offscreen list row. Double-click/Enter opening is suppressed in selection mode and while selection modifiers are held; explicit Read actions remain available. Physical keyboard and Narrator certification remain pending.

The existing filter row contains the selection menu. Narrow windows use a checkbox icon with the complete accessible label. The popup scrolls within the viewport. When books are selected, favorite actions appear first; cancellation appears first during a write. Covers retain native PDF imagery, volume and pointer physics. Selection uses the existing projected outline and marker, while keyboard focus remains independent.

Bulk favorite/unfavorite takes an immutable identifier snapshot. The application actor permits one such job at a time and performs one SQL update per iteration, retaining its response reservation until a terminal reply. The request validates its 80-item bound and identifiers, and deduplicates targets. Missing rows are reported as failures instead of false successes. Cancellation stops subsequent items and preserves completed writes. The reply distinguishes saved, failed and unattempted targets; failed/unattempted books still on the visible page remain selected for retry. The Result menu exposes counts. Individual favorite controls and manual selection changes are disabled during the operation.

Only catalogue favorite flags are written. PDFium and original PDF files are not used by this operation. The existing SQL adapters still own persistence; presentation code does not execute SQL.

## Executed checks

- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` passed.
- `cargo build -p yamantaka --offline` passed.
- `cargo test -p yamantaka-application --lib library_favorites::tests::page_selection_and_bulk_favorites_keep_fixed_targets_and_partial_results --offline -- --exact` ran one passing regression. It checks literal range/additive selection, page retention, select/clear, fixed targets after selection changes, real SQLite partial cancellation, missing targets, duplicate targets, unfavorite, unchanged manual title/progress/position, the request bound and cancellation before the first write.
- `cargo test -p yamantaka-storage --test compatibility overrides_win_and_queries_are_bounded_and_escape_wildcards --offline -- --exact` ran one passing existing compatibility case.
- Five native scenarios passed: `library-selection-covers`, `library-selection-list-es`, `library-selection-small-200`, `library-selection-menu-small-200` and `library-selection-favorites`. The first four retained exactly three selected books. The bulk scenario returned three favorite flags through the normal actor query and cleared successful selections. Native assertions also compared each displayed book against the fixed target set.
- At 640×480 with text scale 2, the closed selection control and both open-menu favorite actions passed recorded control bounds checks. The remaining selection options scroll. This text-scale check is not physical monitor DPI certification.
- Final PNGs were inspected. The source manifest matches every captured UI source and the frozen executable. Publication embeds synthetic provenance; no personal library, PDF or binary is published.
- Capture inventory names are now checked for uniqueness before launching a process. A deliberately duplicated temporary inventory was rejected. This prevents a new library scenario from overwriting an older PDF text-selection scenario.

The native selection previews call the same modifier adapter and write action as the controls. They do not certify actual physical mouse/key sequences. The retained source suite has 55 test cases, with one new focused regression for this stateful write; the full workspace suite was not rerun here.

Evidence is under [rust-library-selection](../screenshots/rust-library-selection/capture-manifest.json), with adjacent PNGs and recorded controls.

## Remaining work

Bulk tag assignment, collection add/remove, archive, cover regeneration and removal still require implementation and acceptance. Broader keyboard/context-menu flows, physical accessibility, large-catalogue interaction timing, shutdown/fault stress and the full release gates remain pending. No stable release is claimed by this checkpoint.
