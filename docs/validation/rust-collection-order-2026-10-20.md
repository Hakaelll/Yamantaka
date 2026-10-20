# Collection order and icon editing

Date: 2026-10-20. Scope: native collection ordering and icon editing on schema v11. This extends [collection management](rust-collections-ui-2026-10-19.md). Full rewrite and release acceptance remain open.

## Behavior

Tools → Collections exposes Move earlier, Move later and Change icon in each collection's action menu. The icon editor stays within that collection's row. It offers Folder, Study, Reference, Archive and Star through named controls, with the saved choice selected and a Cancel action. Collection headings draw the corresponding icon with the existing stroke and palette. Unknown inherited icon names display the folder fallback without changing the stored value.

Move commands use the complete stored order of the collection's pinned or unpinned group. Search and the current page do not restrict their destination. Pinned collections remain first. A move at either group boundary does nothing. The interface keeps its current search and page after saving. The action menu scrolls within the viewport when enlarged text makes it taller than the window. No additional library bar or drag-only interaction is introduced.

The domain defines typed move directions and the five accepted icon identifiers. Both mutations use the existing asynchronous actor and guarded repository changes. They require the loaded collection revision and run within an immediate transaction. Icon changes reject unknown identifiers. A move ranks the complete pin group using stored order, case-insensitive name and ID, swaps adjacent positions and normalizes ranks atomically. This handles inherited duplicate or sparse sort orders and moves across 80-row page boundaries. Changed rows receive new revisions. A stale command, cancellation before commit or SQL failure leaves the transaction uncommitted. An executing SQLite statement itself is not interruptible.

Names, IDs, memberships, pin states and source documents remain intact. Successful changes use the existing query refresh. Failed writes retain the local error and reload current revisions for retry.

## Executed verification

The existing storage case `overrides_win_and_queries_are_bounded_and_escape_wildcards` passed offline. Its collection helper now also uses a temporary file-backed database to verify:

- Reordering tied legacy ranks while preserving pinned precedence and membership.
- Stale move and icon rejection, unknown-icon rejection and pre-cancelled movement.
- A boundary no-op and rollback of all rank/revision changes when a later row rejects an update.
- Movement from the last row of page one to page two with 82 collections.
- Order, icon and membership persistence after closing and reopening the database, with document metadata preserved and SQLite integrity valid.

No standalone test was added. The retained Rust source count remains 55. Workspace all-target offline Clippy with warnings denied and the native development build passed.

[Native screenshot evidence](../screenshots/rust-collection-order/capture-manifest.json) covers completed ordering/icon writes, movement while search hides the adjacent collection, the action menu, and the icon editor at 640×480 with 200% text. Existing compact-manager and deletion-review scenarios also run against the changed interface. The desktop form rows also stop reserving the full remaining height inside the scroll area. Capture assertions check saved icon/order, membership, query completion, visible controls and the spacing between form rows. Fixtures are synthetic. The scenarios invoke normal application adapters and inspect returned state; they do not certify physical mouse, keyboard or Narrator operation.

## Remaining acceptance

Full application restart, physical interaction, monitor DPI and Narrator remain pending. No large collection-group latency result is claimed. Tag editing, collapsible library groups and the remaining parity, migration, rendering, performance and release gates remain separate work.
