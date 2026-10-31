# Inherited reader commands checkpoint

Date: 2026-10-31. Scope: six additional inherited keyboard commands, focused page jump, transactional bookmark toggling and separate chrome/immersive states. This is not full command-palette parity or release acceptance.

## Connected behavior

The configurable registry now includes goToPage, documentNavigator, togglePanel, toggleBookmark, toggleChrome and toggleImmersive using their existing Flutter IDs and defaults. Together with the previous commands, 27 native actions are configurable. The command-palette assignment remains preserved and unconnected.

Ctrl+G opens a page-jump modal scoped to the active reader's document identity and native page count. It displays one-based PDF page numbers, validates the range, retains focus after an invalid Enter, and uses the existing canonical navigation only on confirmation. Cancellation or leaving the reader releases the draft. Confirmation checks that the targeted document is still active in the same pane. The small layout scrolls within the modal rather than overflowing the window. The field has an accessibility label.

Ctrl+J opens the focused pane's Pages navigator. Ctrl+Shift+J closes an open panel or opens it. Main and reference paths reuse their existing drawer/popup controllers. The bookmark command uses a typed Toggle mutation, not the currently displayed bookmark page. Its add/remove operation, affected-page FTS refresh and returned page share one SQLite transaction. Current row labels on other pages remain unchanged. A stale read can be cancelled before a toggle; an already pending write keeps ownership. Successful fresh results clear the stale flag and a toggled editing target clears its old draft.

Ctrl+H changes chrome visibility independently of immersive mode and retains the main panel. Ctrl+Shift+I and native F8 share the immersive-state transition: entering closes panels and transient popups, stops their queries, and hides headers in both comparison readers; leaving shows controls again. Ctrl+H can show controls while remaining immersive. The floating exit restores the normal reader. Original PDFs are unchanged.

The application rechecks modal and surface availability between commands from one input frame. A page key queued after opening Ctrl+G therefore cannot move the PDF behind its new modal. Reader-only commands also stop after an earlier command changes the surface.

## Executed evidence

The existing storage compatibility check overrides_win_and_queries_are_bounded_and_escape_wildcards passed: one passed, five filtered, 4.54 seconds. Its added bookmark helper uses an independent v11 store with 101 existing named bookmarks. It toggles page 200 outside the visible 80 rows, verifies add/remove totals and unchanged visible labels, checks cancellation and an injected insert failure, and runs SQLite integrity. No standalone test was added.

Formatting, offline native build and binary Clippy with warnings denied passed. Three final native capture scenarios passed their state/control-bound assertions:

| Case | Observed result |
| --- | --- |
| legacy-reader-commands | Opens/closes the main navigator, adds/removes its bookmark, switches to reference, opens Ctrl+G, blocks Page Down both in the opening input frame and while modal, types invalid 0 and then valid 2 with Enter, bookmarks reference page 2, toggles chrome while preserving the main panel, opens the reference navigator, and enters immersive mode with both panels closed |
| legacy-reader-jump-small-200 | Stops at the reference page-jump review; both documents remain on page 1; Go to page and Cancel remain visible at a requested 640 × 480 logical viewport, 200% text and reduced motion |
| reader-reference-shortcuts | Existing default-key comparison sequence still preserves independent page, zoom, search and focus state |

The first capture timed out because the synthetic Ctrl+A event omitted Windows' command modifier, so it did not select the field contents. The preview now normalizes command from ctrl and emits phase diagnostics in its isolated capture log. The subsequent complete sequence passed with real egui Text/Enter events. The final modal and immersive comparison PNGs were visually inspected. No Computer Use was used.

Captures use isolated synthetic stores and generated acceptance PDFs. Source, Cargo.lock, executable and artifact hashes were verified before publication. PNGs embed origin metadata. See the [capture manifest](../screenshots/rust-reader-commands/capture-manifest.json).

## Remaining acceptance

The command palette remains required work. Physical keyboard layouts, pointer/keyboard acceptance beyond the exercised scenarios, Narrator and monitor DPI, stale-document replacement during the dialog, cursor auto-hide parity and complete reader performance acceptance remain pending. The tests establish their exercised current-document paths, not every timing boundary or original Flutter behavior. No release bundle or personal PDF was published for this checkpoint.
