# Comparison keyboard focus checkpoint

Comparison panes now have an explicit keyboard target. F6 focuses the other reading canvas; pointer presses and keyboard focus entering pane controls update the target. Reference popup layers retain their ownership even when they overlap another pane. A thin accent border marks the active pane, and both canvas focus targets expose labelled panel semantics.

Page arrows and Page Up/Down, document search, result traversal, zoom and close commands route to that target through the existing canonical navigation and preference controllers. Ctrl++ and Ctrl+- zoom, Ctrl+0 fits the page, Ctrl+F opens the owning search, and F3/Shift+F3 traverse its matches. Ctrl+W closes an active reference through the existing scoped close acknowledgement. Application-wide commands keep their existing routing.

Input fields retain arrow keys. Reader page commands explicitly cancel egui's spatial focus movement: without that cancellation, an arrow could turn a page in a left-hand reference and then move focus to the main reading. F6 closes transient popups and can reveal the reference when a full-width main drawer occupies a narrow window. Primary PDF-selection keyboard handling is disabled outside the main reading's active context. Existing modal shortcut guards remain in place.

## Evidence

The final native binary compiled with `cargo build -p yamantaka --bin yamantaka-rust --offline`; formatting passed. Five focused native scenarios passed through `tool/capture_rust_ui.ps1`:

- `reader-reference-shortcuts`: the capture injects actual egui key and pointer events through the app's raw-input hook. It switches to the reference with F6, turns its page, zooms, fits, opens search with Ctrl+F, types a query while using an editor arrow, submits with Enter, and traverses results with F3/Shift+F3. A click then activates the main canvas; paging and zooming there preserve the reference. The main ends on page 2 and the reference's search remains on page 1. F6 returns focus to the reference.
- `reader-reference-shortcuts-small-200`: the sequence runs at a requested 640×480 window with 200% scaling, Spanish, light theme and reduced motion. It first opens a full-width main text drawer, then uses F6 to reveal and activate the reference. F6 also returns to the main canvas from its popup.
- `reader-reference-shortcuts-close`: the panes are swapped before the sequence. Ctrl+W then closes the active reference, with exactly one native close acknowledgement, while retaining the main PDF and page.
- `reader-reference-book`: the existing independent reference search and swapped layout remain functional.
- `selection-keyboard-dark`: primary keyboard selection still reaches its existing acceptance state.

The event sequences wait for the current page's requested image and generation before deriving a zoom step. This avoids measuring the previous page's Fit scale. The [native captures and manifest](../screenshots/rust-reference-shortcuts/capture-manifest.json) record executable and source hashes. Only synthetic PDFs were opened; no Computer Use or external input automation was used. No standalone Rust tests or broad workspace suite were added or run.

Reference selection/annotation tools, shortcut customization, overlays, physical Narrator/DPI and full performance acceptance, and Release packaging remain pending. The captures establish these input paths, not full keyboard or accessibility acceptance.
