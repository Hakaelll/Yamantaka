# Reader tab state and closure policy

The domain now owns an operational tab controller with a twelve-tab limit, independent per-tab reading context and a ten-entry closure stack. This checkpoint implements and verifies the state transitions. It does not expose a tab selector in the native application yet and does not certify PDF handle lifecycle or persisted recently-closed history.

## Existing behavior and decisions

The reference is `lib/features/workspaces/application/document_tabs_controller.dart`, with `DocumentTab` from `workspace_models.dart`. Existing document openings select their first tab unless a new tab is explicitly requested. Closing the active tab selects the adjacent surviving tab. Reordering changes cycling order. Close others and Close to right preserve pinned tabs. At capacity, opening evicts the last unpinned inactive tab; a fully occupied pinned/active set produces an error without a state change. New pinning retains the existing three-tab limit. Restoring an imported tab set can retain older pinned choices.

Each Rust tab has distinct tab/document identities, page, validated document preferences and an optional typed panel. Reusing a tab keeps its saved context. Forced duplicate-document tabs retain separate positions. Context includes zoom mode/value, rotation, margins, cover/pairing choices, theme and scroll rather than just a page number. Invalid context or duplicate tab identities fail before modifying the current set. An invalid restored set leaves the current tabs and closure stack intact; an unavailable active ID selects the first restored tab.

The specification requires at most two hot documents. The controller marks one active tab and keeps at most one other distinct document warm, selecting it in reverse tab order. Tabs sharing either document share its resource classification without consuming another document slot. Other tabs are suspended state only. `hot_documents` returns unique IDs for the future native ownership adapter; this state classification does not itself open, close, retain or release a PDF.

Closing, capacity eviction and bulk closure retain at most ten snapshots. Reopening at the twelve-tab limit fails and keeps the snapshot available. A successful reopen preserves its identity and context. Unlike Flutter's unbounded accumulation in some bulk-close paths, Rust applies the same ten-entry bound to every closure path.

## Verification

Six domain regressions exercise reuse versus explicit duplicate openings, independent reading contexts, capacity eviction, pinning/full pinned sets, adjacent selection, bulk close/reopen, the ten-entry closure bound, reordering/cycling, distinct hot-document limits and atomic rejection of invalid restoration or snapshot updates. The controller contains no UI, SQL or FFI calls.

The locked workspace passes 381 tests. After the final reorder-boundary adjustment, all 78 domain tests were rerun successfully. Formatting and warning-denied workspace/all-target Clippy pass. Evidence remains in `build/verification/reader-tabs-core-tests.log` and `build/verification/reader-tabs-core-final-domain-tests.log`.

## Remaining integration

Connect admitted native open/switch/close results to this controller without committing a failed or superseded opening. Add the desktop selector, keyboard/context-menu actions, state synchronization, suspend/reopen behavior and resource release evidence. Adapt and preserve the persisted `recently_closed_documents_v1` list, workspace rows and queue fields, then connect restoration to canonical navigation. Physical interaction, full continuity parity and the final Release gates remain pending.
