# Bulk library organization

Date: 2026-10-20. Scope: document tags, collection membership and archive, extending the native page selection and favorite operations. The rewrite and release acceptance remain incomplete.

## Connected behavior

The selection menu supports favorite/unfavorite, assigning an existing tag, adding to or removing from an existing collection, and archiving. A typed `library_bulk` job replaces the favorite-only job; its callers and scoped reply path were migrated together. One document mutation runs per actor iteration. The immutable target snapshot, 80-book bound, deduplication, cancellation and saved/failed/unattempted results are shared across actions.

Tag writes use the existing atomic membership/FTS adapter. Index access comes from the actor's current indexing policy. Annotation tag memberships and manual document metadata are preserved. Collection actions carry the chosen collection revision; a subsequent rename or deletion produces failures rather than silently using a stale revision. Completed writes remain committed on cancellation, and visible failed/unattempted targets remain selected for retry. Individual tag/collection editing controls share the active bulk write lock, preventing a later UI membership edit from being overwritten by an unfinished bulk item. Organization managers and enabled search statistics refresh after relevant successful replies.

Archive follows the connected Flutter service: ensure a collection named Archivado, then add the document. The storage adapter now creates or reuses that collection and adds each membership in one transaction. Existing IDs and collection customization are preserved, repeated archive is idempotent, missing documents do not create empty collections, and a failed membership rolls back a newly created collection. Main schema v11 and backup formats are unchanged.

The target chooser captures the selected identifiers when opened. Independent request tokens govern paged reads; searching or closing cancels an obsolete read. Pending/error results cannot submit a stale displayed destination. Search, empty state, retry and pagination are connected through the application actor, with at most 80 destination rows per page. The dialog blocks background shortcuts while it owns focus.

A fixed header/search and footer keep pagination and Cancel visible while the result list scrolls. Long destination labels use a bounded variant of the existing custom control; complete labels remain in accessibility metadata and hover text. Ordinary controls keep their existing text/icon policies. Selection interactions, menu rendering and preview code were separated into focused modules.

## Executed checks

- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` passed.
- `cargo build -p yamantaka --offline` passed.
- `cargo test -p yamantaka-application --lib library_bulk::tests::page_selection_and_bulk_operations_keep_fixed_targets_and_partial_results --offline -- --exact` ran one passing retained regression. It covers literal selection results, fixed targets, partial/pre-cancelled favorites, missing/duplicate targets, tag assignment with preserved annotation use, collection add/remove, stale revisions, repeated archive identity/counts, forced membership failure rollback, and preserved manual title/progress.
- `cargo test -p yamantaka-storage --test compatibility overrides_win_and_queries_are_bounded_and_escape_wildcards --offline -- --exact` ran one passing existing compatibility case, including the connected organization adapters. The retained source count remains 55 cases; no standalone case was added.
- Ten native scenarios passed, including the existing compact tag and collection managers. Tag/collection pickers resolved the 81st destination on page two. Destination, Previous and Cancel controls stayed within recorded bounds in the normal and 640×480/text-scale-2 chooser captures, including a long accented label. The harness explicitly requires visible destination, Previous and Cancel controls.
- Native applied previews re-read organization metadata after the scoped write and then query the library through the corresponding filter. Tag assignment, collection addition and archive returned exactly the three selected IDs. Removal from a four-book collection returned only the unselected book. The tag readback retained its existing annotation membership. The favorite regression capture still returned three favorites through the generalized actor path.
- Final PNGs were inspected. The capture manifest matches the frozen executable and every captured UI source. Published screenshots include synthetic provenance; personal libraries, PDFs and executables are excluded.

Evidence: [native manifest](../screenshots/rust-bulk-organization/capture-manifest.json), with adjacent PNGs and control JSON. The source-pinned previews exercise shared selection, chooser query, submission and readback adapters, not physical pointer/key sequences.

## Remaining acceptance

Bulk cover regeneration and removal from the library remain to be implemented. Physical keyboard/Narrator/DPI validation, selector typing/cancellation and late-result interaction stress, per-action cancellation/fault stress, and large existing-index tag-update latency remain pending. The full workspace suite and full performance comparison were not rerun for this checkpoint. A stable release is not claimed.
