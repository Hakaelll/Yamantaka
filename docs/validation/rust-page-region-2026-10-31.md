# Visible-region refinement checkpoint

Date: 2026-10-31. Scope: optional visible-region PDFium rasters layered over retained whole-page previews in the native wgpu reader.

## Behavior

The primary and comparison panes use the same region planner and presentation state. After the visible geometry settles for 150 milliseconds and its preview is ready, the planner requests a padded visible crop at physical display scale. Canvas dimensions respect the device limit and the 16,384-pixel ceiling. The crop is reduced to fit its allowance when necessary; it is omitted when it would not improve the existing whole-page target resolution.

A pane's allowance is one sixth of the configured tile budget, divided by two in comparison and then among its visible pages. With default limits, one pane can request up to 4 MiB of crop pixels. Native CPU buffers, GPU textures and aligned upload staging remain charged through the existing memory governor. This allowance leaves room for replacement lifetimes; it does not claim to account for opaque driver allocations.

A useful crop switches the whole-page target to the shared 768-pixel preview. The old preview and crop remain visible until their replacements arrive. Crop meshes map their canvas-relative region into the page rectangle before annotation, link and selection overlays. Rotation changes suppress an incompatible old crop. Cancellation tokens and complete render keys reject stale deliveries. Removing a page cancels its crop; optional submission, native or upload failures skip that crop and restore the ordinary whole-page target without placing the reader in an error state.

Nearby CPU preparation waits for visible refinement to settle. Safe mode and Glow do not request crops. Their existing whole-page rendering stays available.

## Validation

- Offline UI Clippy passed with warnings denied; the offline native application build passed.
- The retained `native_tiles_stitch_to_full_render_for_mixed_pages_and_every_rotation` real-PDFium test passed. It compares clipped rasters against full renders for textual, rotated and scanned fixtures at every quarter-turn, including seams, and checks unchanged source hashes.
- Four native synthetic capture profiles exercise high zoom, a quarter-turn, comparison and Glow. Capture summaries record region canvas, crop rectangle, delivered generation, tracked memory and settled state. Published artifacts and source hashes are in [the capture manifest](../screenshots/rust-page-region/capture-manifest.json).

## Limits

These cases do not establish a no-flash guarantee under rapid input or a p95 frame budget. GPU halo presentation, source handoff, device loss, failure injection, full corpus acceptance and complete Glow memory accounting remain pending. Physical DPI, Narrator and multiple monitors were not exercised. No personal PDF or library is included in these artifacts.
