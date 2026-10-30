# Reference passage selection checkpoint

The reference now supports passage selection and copy through the same SelectionState controller and geometry presentation as the main reading. Its I-beam tool activates selection; selecting it again opens Copy, Clear and Done. Small panes place the tool inside Reference options. The reference does not add a permanent selection toolbar. During comparison, main selection actions move to Reading options and retain the main passage. This removes the shared selection toolbar that left no PDF canvas below a compact reference header.

Selection rendering now uses the pane's UI scope for widget identities. Focus bounds intersect the visible paper clip, preventing a large PDF's offscreen center from assigning focus to another pane. Each state retains its own visible focus IDs. These changes allow two views of the same document and page to select independently.

The selection widget retains horizontal-arrow and Escape input through egui's focus filter. Escape clears its passage without releasing focus, so Ctrl+A still selects the page afterward. Page/history commands recognize document selection focus while text fields retain their editing keys. Copy and Escape route once to the active pane under the existing modal guards.

ReaderPane now routes page-text replies to the exact owning selection or text consumer by document, page and cancellation token. Enabling selection adopts an existing matching page-text request or payload. Both text panels reuse selection text, and reference close clears its consumers before native closure. Requests retain the existing limit of two pending pages per selection state and the shared memory-governed PDF text/copy implementation.

Small comparison canvases now cap each presentation margin at one eighth of its viewport axis. Fit calculations, destination viewports and content placement use those same margins without rewriting stored preferences. The main scroll area also honors its actual pane height instead of imposing egui's default minimum. These changes prevent a tiny Fit page from falling outside the visible canvas when toolbars and 200% scaling reduce the available space.

## Evidence

The final native binary compiled offline and formatting passed. Five focused scenarios ran through tool/capture_rust_ui.ps1:

- reader-reference-selection swaps the panes, selects the main PDF's first character Y, then selects M from page 2 of a distinct reference. Its copy command returns that reference character. Escape clears only the reference range; Ctrl+A and another copy return Mixed landscape page. Alt-arrow history traverses away and back while preserving the main selection. The reference copy popup is captured with enabled, visible controls.
- reader-reference-selection-shared shows the same PDF and page in both panes. It asserts distinct widget IDs, preserves the main Y range and copies the reference's full first-page passage independently.
- reader-reference-selection-small-200 repeats the flow in Spanish, light theme, reduced motion, a requested 640×480 window and 200% scaling. Recorded bounds require Copy, Clear and Done to remain fully visible.
- reader-reference-selection-close closes the distinct reference with exactly one scoped native close acknowledgement, then waits for the rebuilt source widget before requesting test focus and copying the retained main Y selection.
- selection-keyboard-dark retains the original primary keyboard-selection acceptance path.

The comparison sequence waits for the comparison layout before starting its first pointer gesture. It also transfers a pending page-text request into selection and traverses history while preserving the selected passage. Real egui pointer, key and Copy events drive the sequence. The synthetic capture path records the copy command's document and text instead of writing the Windows clipboard. This verifies routing and payload; physical clipboard acceptance is not claimed. No Computer Use or personal PDFs were used.

The [native captures and manifest](../screenshots/rust-reference-selection/capture-manifest.json) record source and executable hashes across all Rust crates. Bounded native child logs retain debug-only synthetic selection progress when a capture fails; the harness joins a timed-out child before reporting failure. One essential geometry test, small_fit_page_has_visible_geometry_on_both_axes, was added and passed. It checks explicit fitted rectangles on narrow and short viewports and retains ordinary-window margin behavior. No broad workspace suite was run. Reference annotation tools, full cross-page/rotation/geometry acceptance, shortcut customization, overlays, physical keyboard/Narrator/DPI, performance acceptance and Release packaging remain pending.
