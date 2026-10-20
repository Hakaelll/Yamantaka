# Reviewed native library removal

Date: 2026-10-20. Scope: the native interface for the existing [storage transaction](rust-library-removal-storage-2026-10-20.md) and [runtime coordinator](rust-library-removal-runtime-2026-10-20.md). Full rewrite and release acceptance remain open.

## Connected behavior

The library selection menu, a book's context menu and Delete on a focused book open the same confirmation. It freezes the selected identifiers and states the count, removal of local reading data, preservation of original PDFs and absence of undo. Cancel receives initial focus. The compact layout keeps the consequence and both buttons visible. Cover regeneration uses the same review component with its own consequence and action.

Before removal, the interface caches reading context and asynchronously finishes selected reading sessions. A failed session save stops preparation before native removal. Once selected saves settle, it pauses their ordinary cover consumers and retires the selected active reader's presentation. The coordinator then closes native documents before attempting each SQL transaction. Existing progress and cancellation stay in the selection menu.

Actual native closures retire duplicate reader tabs. Successful database removals also discard their recently closed contexts, visible cards, cover slots, inspector cursor and cached search results. Failed records remain available for reopening. Unselected live and closed readings retain their context. Catalogue, organization, history, notes and enabled index statistics refresh after successful removal. Removing the last item on a later catalogue page returns to the last valid page.

Continuity save and load reject authoritatively absent catalogue identifiers. They retain records whose source files are missing and preserve unknown preference fields. A matching displayed source path clears the Rust last-document preference. The interface defers intermediate continuity writes during removal. A window-close request cancels remaining work and waits for the terminal outcome before normal shutdown persistence. This close-during-removal path has source review only, not physical interaction acceptance.

Reader recovery is inside Tools, including Open readings and Reopen last reading. The library no longer adds a separate recovery row below its header. The 3D book presentation and physics retain their existing composition.

## Verification

The retained `library_bulk::tests::page_selection_and_bulk_operations_keep_fixed_targets_and_partial_results` case passed offline. Its existing helper now verifies duplicate-tab retirement, pruning of removed live and closed continuity contexts through the real writer, and session-save failure followed by successful retry. No standalone test was added. The retained Rust source count remains 55.

Workspace all-target offline Clippy with warnings denied and the native development build passed. Native captures use isolated synthetic file-backed stores and generated PDF fixtures. They drive shared application adapters, real readers, session saves and acknowledged continuity persistence. They do not simulate physical mouse or keyboard input.

The final capture set covers ordinary and compact removal review, successful removal, cancellation before admission, a partial SQL failure, hidden reading recovery, last-page recovery, compact cover review and cover replacement. Successful and partial removal reopen the retained reading at zoom 1.25 and verify its saved context. The partial-failure case leaves the failed book selected for retry. Cancellation preserves all ten records. Last-page recovery removes the sole item on page two and returns to 80 records on page one.

An earlier combined capture run timed out on compact cover review after seven successful removal scenarios. A separate rerun completed both cover cases. The final complete capture run is recorded in the published manifest. [Screenshot evidence](../screenshots/rust-library-removal/capture-manifest.json) binds image and control hashes to the executable, UI sources and Cargo lockfile. Screenshots certify only these synthetic scenarios and logical layouts.

## Remaining acceptance

Physical Delete, context-menu and focus interaction, Narrator, monitor DPI, close during removal, broader import and index concurrency, large indexed-corpus latency, owned-cache reclamation and full migration acceptance remain pending. These checks do not establish whole-application performance, full parity or Windows release readiness.
