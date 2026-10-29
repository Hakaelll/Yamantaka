# Reference navigator checkpoint

The reference header now has one navigation popup with Contents, Pages and Find sections. Pages uses the same virtualized thumbnail view as the main reader, with separate state, cancellation tokens and render routing. Selecting a thumbnail uses canonical reference navigation and respects the selected synchronization options.

The view retains at most 16 thumbnail slots and limits each image side to 256 physical pixels. Switching sections, closing the popup or closing the reference cancels pending thumbnail work and releases its images. Render replies must match the complete requested key and cancellation identity before upload.

Small panes group the book chooser, zoom and linking tools under Reference options. The repeated document heading is omitted there. The reference canvas also bounds its scroll area to the actual available height: the previous minimum scroll height could discard the canonical offset and leave a small pane showing only page margins.

## Evidence

The final native binary compiled with `cargo build -p yamantaka --bin yamantaka-rust --offline`. Five focused scenarios passed through `tool/capture_rust_ui.ps1`:

- `reader-reference-thumbnails`: a distinct reference navigates to page 2 while the main reading remains on page 1. Releasing the navigator empties its thumbnail slots before they are rebuilt. Main thumbnail state remains empty.
- `reader-reference-thumbnails-small-200`: the same behavior at a requested 640×480 window, 200% scaling, Spanish, light theme and reduced motion. The Pages and selected Page 2 controls are enabled and fully visible.
- `reader-reference-outline`: the independent hierarchy, collapse/expand and canonical destination remain functional.
- `reader-reference-book`: independent reference search and swapped panes remain functional.
- `thumbnails-5000-last`: the shared main thumbnail view reaches page 5,000 within its existing residency checks.

The [native captures and manifest](../screenshots/rust-reference-navigator/capture-manifest.json) record the executable and source hashes. Only synthetic fixtures were used. The small capture confirms this popup and navigation path; it does not establish physical DPI or Narrator acceptance. No standalone Rust tests or broad workspace suite were added or run for this checkpoint.

Other reference reader tools, per-pane shortcut routing, overlays, full performance and accessibility acceptance, and Release packaging remain pending.
