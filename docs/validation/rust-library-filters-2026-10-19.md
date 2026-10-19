# Library filter recovery

Date: 2026-10-19. Scope: native library filter state and synthetic captures. The full rewrite remains incomplete.

## Change

An empty collection, tag filter or smart view could display the onboarding message instead of a filtered empty state. The empty-state reset also omitted these newer filters. The filter menu highlight, toolbar reset and empty-state reset now share one active-filter predicate and one reset operation.

Reset clears search, favorites, reading state, smart view, missing/modified flags, collection and tags. It cancels obsolete tag/collection restoration checks, returns to the first page, refreshes through the application actor and saves preferences through the existing adapters. Sort order remains unchanged. Search containing only whitespace follows the storage query's existing trimming behavior.

Library replies must match the current request token. Capture readiness now tracks completion of that query instead of requiring a nonempty page, allowing genuine zero-result states to be verified.

## Executed evidence

- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` passed.
- `cargo build -p yamantaka --offline` passed.
- `tool/capture_rust_ui.ps1` ran `filters-empty-smart` and `filters-reset-smart-es` against the frozen executable. Both processes exited successfully. The harness now rejects incorrect counts, filter state or unfinished queries for these cases.
- The empty Missing files view completed with zero results, an active smart filter and visible Reset filters controls. Native control bounds passed.
- The reset preview exercised the same reset operation used by both buttons after setting every filter category and page offset. Native assertions checked that no filter remained, the offset became zero and sort order was preserved. The actor returned all ten synthetic books. This checks the shared action, not a physical pointer click.
- Actual PNGs were inspected in dark English and light Spanish with reduced motion. Published images include synthetic provenance. No new standalone test cases were added.

Evidence: [empty filtered library](../screenshots/rust-library-filters/filters-empty-smart.png), [restored library](../screenshots/rust-library-filters/filters-reset-smart-es.png), and the adjacent control JSON and capture manifest.

## Remaining acceptance

The native empty-state screenshot covers a smart view. Separate physical interactions for empty collections/tags, a real application restart, Narrator and monitor DPI were not certified here. The full workspace test suite was not rerun for this presentation change. Library multiselection and bulk operations remain pending.
