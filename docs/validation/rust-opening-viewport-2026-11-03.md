# Opening prepares the actual primary viewport

Primary opening now plans its initial images from the native reader's prospective viewport rather than a target-page or whole-spread approximation. An invisible child UI measures the shared toolbar and navigator containers without advancing the live layout, painting controls, reading SQL, or changing the current reading. It excludes the library header and inspector, retains common notices and the synthetic verification footer, and applies the pending session and tab context in adoption order.

The candidate caches its domain layout. Its normalized rotated page start, zoom, optional restored scroll, and native scroll container produce a canonical `ReaderFrame`. Only intersecting pages are staged. A narrow navigator drawer that hides the PDF retains a target-page preview instead. If the visible page set changes while preparation is pending, its old tickets and images are released and the new set is planned. The 2 MiB initial raster allowance is divided among required pages; existing GPU and transient governor charges remain in force.

Adoption still waits for every required image. It occurs after measurement in the first UI pass. A composition change requests egui's additional layout pass before painting the new reader, including when opening from the library. The old reading remains active on preparation failure. The navigator uses one shared width reservation for real content and measurement; long or enlarged controls scroll inside the selected width. No persistence format, dependency, or standalone Rust test was added.

## Executed evidence

The extended native opening probe records the planned frame and the first painted frame, including viewport dimensions, scroll, zoom, visible pages, and missing images. It asserts exact agreement for that geometry and continues through settled rendering before checking empty-page paint calls.

- The two-page continuous baseline prepared only page 0. Its first frame contained pages 0 and 1, with page 1 missing and one empty-page paint. The final implementation prepared both, with no missing images or empty-page paints.
- Opening from the library with its inspector enabled, 1.75× zoom, 90° rotation, and the second page passed. The initial scroll was `[12, 1116]`; only visible page 1 was prepared.
- The Spanish light reader at 200% text, with a navigator and fit-width continuous layout, passed. Planned and painted viewport dimensions were both 468×402 logical points. The requested and actual navigator widths were both 300 points. The rendered capture was inspected.
- Facing opening in Glow passed an injected member failure and successful retry without replacing the previous reading prematurely. Both visible pages were ready at adoption.
- The existing saved-workspace scenario passed, including Book mode with a raw single-page preference, panel width, columns, and comparison restoration. Its first-raster probe is disabled, so this is not a reference-pane first-frame claim.
- A generated 5,000-page PDF opened in continuous mode with pages 0 through 6 visible. Exactly those seven pages were staged, with matching planned and painted geometry, no missing images, and no empty-page paints.
- The settled two-page continuous before/after captures were pixel-identical at 1800×1200. This verifies unchanged settled composition, not presentation latency.
- Native debug compilation, workspace formatting, and workspace Clippy across all targets with warnings denied passed. All 39 generated source PDFs retained their hashes.

The [status artifact](artifacts/opening-viewport.json) contains executable hashes and selected results. Local captures are in `build/verification/continuous-opening`.

The first enlarged-text panel attempt exposed a measurement error: an empty egui panel shrank to its minimum width instead of reserving the chosen width. The shared content container corrected that mismatch. An earlier 900×700/200% attempt hid the PDF behind the narrow drawer and could not advance this viewport-based probe; that owned capture was stopped. It is not a passing compact-drawer opening check.

## Remaining acceptance

Rapid resize and input during preparation, broader restored-scroll/context combinations, sustained cancellation and memory pressure, reference-pane handoff, physical DPI/accessibility, and paired frame-time and latency measurements remain open. These logical paint checks do not prove uninterrupted physical frame presentation or full-rewrite readiness. The final replacement Release remains gated on the full parity matrix.
