# Nearby page previews

While you read, Yamantaka can prepare up to two nearby page previews per panel. It waits until visible pages and region refinement are ready, then schedules lower-priority native work. With the default wgpu compositor, completed previews can also retain GPU textures. Navigating to the prepared page transfers that texture into the visible page slot. A rotation or viewport-size change can reuse the earlier texture while the current geometry refines. The image retains its original raster dimensions and rotation, so it is not mistaken for the finished replacement. [Geometry-change verification](../validation/rust-nearby-geometry-2026-11-03.md) records the native two-pane sequences. This preparation has no progress message or extra page panel.

The same preview sizing is used for visible-page previews and preparation. When a completed cached raster matches a later request, its immutable pixels can be reused across navigation changes. Pages still refine for the current zoom when opened. Reopening or replacing the native document changes its cache identity.

Changing the reading context cancels its old preparation requests. Memory or queue pressure can skip optional work without showing a reading error. A failed optional GPU upload leaves the CPU cache available; Glow uses that CPU-only preparation. Preparation stops outside the reader and in [safe mode](safe-startup.md). Page previews use the in-memory cache; the cover-image disk cache remains a separate setting.

This is one part of reader responsiveness. Complex native rasters, pages outside the prepared range and device recovery remain relevant to page-turn latency. The [CPU cache checkpoint](../validation/rust-page-prefetch-2026-10-31.md) and [GPU preview checkpoint](../validation/rust-nearby-presentation-2026-10-31.md) record the exercised cases and remaining performance acceptance.

## Opening a reading

When you open another document in the primary reader, the current reading stays available while its replacement obtains preferences and previews of the pages visible in the first view. This includes continuous and facing layouts. Zoom, rotation, panel space, cover pairing, reading direction, and restored scroll follow the saved reading context. If the navigator drawer hides the PDF, a preview of the target page is kept. If preparation fails, the current reading is retained so you can try opening the other document again. Original PDFs remain unchanged.

Native checks cover continuous opening, a 5,000-page PDF, opening from the library, enlarged text with a navigator, and failed facing-page preparation with retry. [Initial viewport verification](../validation/rust-opening-viewport-2026-11-03.md) records matching planned and painted views. Rapid resizing and broader performance acceptance remain pending. [Facing opening verification](../validation/rust-opening-spreads-2026-11-03.md) and [opening verification](../validation/rust-first-raster-opening-2026-11-03.md) record earlier checkpoints.

Changing the reference to another document also keeps the previous pane until its replacement has a page image. If that preparation fails, Retry reopens the failed document. Saved comparisons prepare their saved reference page before the switch. [Reference replacement verification](../validation/rust-reference-opening-2026-11-03.md) covers native failure, cancellation, retry, password, and workspace sequences. Preparing the initial same-document comparison before it appears remains pending.

## High zoom

With the default wgpu compositor, a page can retain a small whole-page preview while PDFium refines the visible region. Refinement waits for 150 milliseconds of stable region geometry and for the preview to finish. This delay is a scheduling policy, not a measured response time.

The crop uses the current rotation and physical scale, with a two-pixel sampling margin. Each pane divides its crop allowance among visible pages. The previous crop stays in its page-relative position while a replacement arrives. An optional crop failure falls back to the whole-page pipeline without a reader error. Safe mode and the alternate Glow compositor keep the whole-page pipeline.

[Region refinement validation](../validation/rust-page-region-2026-10-31.md) records the exercised native cases. Rapid panning, device recovery and paired performance acceptance remain pending.

## Text selection failures

If a required text page fails while extending a selection, Yamantaka pauses new background requests for the remainder of that selection. Visible pages can still request their text. The existing **Retry selection** action clears failed text entries and allows requests to continue. Selecting a smaller passage or ending the selection releases text that is no longer needed. An unrelated visible-page failure does not pause a different selected range.

This affects text extraction, not PDF page rendering or the original files. The [selection admission checkpoint](../validation/rust-selection-failure-admission-2026-11-02.md) records the exercised cases. egui text layout, font textures and opaque driver allocations are not included in the tracked PDF-buffer totals.
