# Native shared-tag management

Date: 2026-10-21. Scope: the native controls for the [guarded shared-tag repository](rust-tag-management-storage-2026-10-21.md). The broader rewrite and release gates remain open.

## Connected behavior

Tools → Tags exposes Rename, Change color, Merge into and Delete tag through each tag's action menu. The menu's accessible name includes the tag name. Rename and color editing stay within the selected row, with explicit save and cancel controls. Names retain the 60 UTF-16-unit limit. Color editing offers named choices and a hexadecimal RGB field with a preview. Imported color values remain intact until the user saves a replacement.

Merge opens a scoped, searchable destination chooser with pages of at most 80 stored tags. The source remains fixed while search or paging changes. It is excluded from destination controls. Only rows scroll; search, pagination and cancellation stay within the dialog. Long destination names retain their complete accessible name and hover text through the existing bounded control renderer.

Choosing a destination opens a separate confirmation with both names. Deletion uses the same review pattern for one tag. Both explain the association changes, preservation of books and notes, and absence of undo. Cancel receives initial focus. The compact review keeps its actions outside the scrollable consequence area. Background reading and library shortcuts remain inactive while either dialog is open.

The controls dispatch the existing typed actor changes. Successful rename updates selected filter labels; merge replaces the source filter with the destination and removes duplicates; deletion removes the deleted filter. The changed filter set is saved immediately, then a separately scoped query rechecks its identifiers and labels. Older filter replies are cancelled. The manager reloads the edited or merged tag, while deletion recovers its normal list page.

Successful changes invalidate reader annotation records, global-note pages and cached global-search results. Conflicting annotation editing and writes block shared-tag changes. New annotation edits and bulk note writes wait while a tag change owns its request. Library bulk mutations and removal also reject conflicting admission. Ordinary reader presentation remains available without reopening its PDF.

Failed writes preserve the draft and selected filters, retain a local error, and reload current metadata revisions for explicit retry. Closing a chooser cancels its read scope. A window-close request during a submitted change cancels remaining work and waits for its terminal outcome before normal shutdown. The close-during-write path has source review only; physical acceptance remains pending.

## Verification

The native rename acceptance first failed against the preceding executable, whose tag manager lacked editing controls. It passed after connection. No standalone Rust test was added; the retained source count remains 55.

[Native evidence](../screenshots/rust-tag-management/capture-manifest.json) contains 13 synthetic scenarios: rename; compact color editing; compact merge destination selection; the second page of an 82-tag query; compact merge and delete review; committed merge and deletion; injected rename failure with retained draft; cancelled review; the actions menu; and the existing filter-restoration and compact-manager regressions.

Assertions verify the selected filter names, matching book counts and settled queries. Merge retains two book associations and one annotation association under the destination. Deletion removes the source filter and associations. Rename failure preserves the prior name, annotation membership, filters and draft. Each management scenario round-trips its filters through the normal preferences writer and decoder in an isolated file. This is a preferences round trip in one process, not a full application restart.

The capture fixtures use generated PDFs and an isolated synthetic catalogue. They invoke shared application methods, dispatch real actor commands and inspect repository replies. They do not certify physical button, keyboard or Narrator operation. Image/control hashes and final executable/UI-source hashes are recorded in the manifest. Compact native checks require visible save, cancel and destructive-action controls, including a valid merge destination.

All 55 retained workspace tests passed offline after the final UI changes. Workspace all-target offline Clippy with warnings denied, formatting and the native development build also passed. Independent code review checked request routing, stale replies, filter reconciliation and conflicting-write guards. The final native capture run passed all 13 scenarios. No latency or whole-rewrite acceptance is inferred from compilation or screenshots.

## Remaining acceptance

Physical keyboard and screen-reader behavior, monitor DPI, real application restart, close during a write, preference-write failure recovery and large-membership latency remain unverified. Collapsible groups, advanced organization, migration, rendering/resource coverage, performance comparison and release packaging remain separate acceptance work.
