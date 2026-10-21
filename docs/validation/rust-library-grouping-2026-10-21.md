# Native collapsible library groups

Date: 2026-10-21. Scope: grouping the current catalogue page in the native library. The full rewrite and release gates remain open.

## Connected behavior

The existing sorting menu includes Group by: author, collection, tag, reading state, or no grouping. The choice is saved in the independent Rust preferences. Reset filters also removes grouping. There is no additional permanent toolbar.

Both the physical cover grid and document list use collapsible group headings. Heading counts describe the current page of at most 80 books. Groups appear in first-encounter order, and each group preserves the current document sort. Unknown authors, books without collections or tags, and the three reading states have English and Spanish labels. Named groups and missing-value groups retain distinct internal identities, even when their displayed labels coincide.

Collection grouping chooses the first membership in the complete pinned/order/name/ID sequence; tag grouping chooses name/ID order. The actor reads those labels for the current document IDs through one bounded SQL query. A reverse collection-membership index avoids scanning all memberships for each book. No full-catalogue membership map is loaded by the UI.

Folding removes hidden books from range selection, Select all, keyboard navigation and the Delete shortcut's fallback. Hidden inspector selection and deferred keyboard focus are cleared. Up/Down follows actual grid rows across group boundaries, including incomplete rows. Already submitted bulk commands retain their original immutable targets. Folded state is temporary and retained only for groups on the current page; the grouping choice survives restart.

The grid reuses the existing cover renderer, physics, source imagery and context actions. Cover headroom remains in full grids; miniature layouts use tighter heading spacing. Group headings wrap long names, expose expand/collapse labels and counts through widget semantics, and paint an independent keyboard-focus outline. The sorting menu scrolls within the available window height.

## Verification

The retained source suite remains at 55 tests. The existing storage compatibility case now checks pinned primary collections, case-insensitive primary tags, missing documents, absent memberships, cancellation, empty input and the 80-ID boundary. Runtime update consumers were migrated together with the new membership payload.

Native synthetic captures exercise author grouping, its first group folded, Spanish/light collection grouping, grouped tag lists, and Spanish/light reading-state grouping in a small window with 200% text. Their acceptance checks verify no lost or duplicate page documents, stable within-group order, additive/range selection, folding and vertical row navigation. An initial preview failed because its simulated first click did not establish a selection anchor; the corrected Ctrl-click setup passed. This was a verification-fixture error.

Independent source review found two issues: hidden inspector selection could reach Delete, and flattened arrow offsets skipped columns at group boundaries. Both were fixed and rechecked. Review did not exercise physical accessibility.

Final verification passed: `cargo fmt --all --check`, offline workspace Clippy across all targets with warnings denied, all 55 retained workspace tests, and the native application build. All five final captures passed their acceptance checks and horizontal control bounds; manifest hashes match the executable, Cargo.lock and current UI sources. Two bounded visual passes checked the normal views and tightened miniature heading spacing. At 200% text in a small window, the library remains scrollable; the first viewport does not fit a complete book row. See the [capture manifest](../screenshots/rust-library-grouping/capture-manifest.json).

## Remaining acceptance

Physical mouse and keyboard interaction, Narrator, monitor DPI transitions and large-catalogue timing remain unverified. Native snapshots and synthetic assertions do not certify those gates. This checkpoint does not complete the rewrite or authorize its release.
