# Nearby page previews

While you read, Yamantaka can prepare up to two nearby page previews per panel in the existing memory cache. It waits until the visible pages are ready, then uses lower-priority native work. This preparation has no progress message or extra page panel.

The same preview sizing is used for visible-page previews and preparation. When a completed cached raster matches a later request, its immutable pixels can be reused across navigation changes. Pages still refine for the current zoom when opened. Reopening or replacing the native document changes its cache identity.

Changing the reading context cancels its old preparation requests. Memory or queue pressure can skip optional work without showing a reading error. Preparation stops outside the reader and in [safe mode](safe-startup.md). Page previews use the in-memory cache; the cover-image disk cache remains a separate setting.

This is one part of reader responsiveness. GPU presentation of a newly visible page, complex native rasters and further device-recovery work remain relevant to page-turn latency. The current [validation checkpoint](../validation/rust-page-prefetch-2026-10-31.md) records the exercised cases and remaining performance acceptance.

## High zoom

With the default wgpu compositor, a page can retain a small whole-page preview while PDFium refines the visible region. Refinement waits for 150 milliseconds of stable region geometry and for the preview to finish. This delay is a scheduling policy, not a measured response time.

The crop uses the current rotation and physical scale, with a two-pixel sampling margin. Each pane divides its crop allowance among visible pages. The previous crop stays in its page-relative position while a replacement arrives. An optional crop failure falls back to the whole-page pipeline without a reader error. Safe mode and the alternate Glow compositor keep the whole-page pipeline.

[Region refinement validation](../validation/rust-page-region-2026-10-31.md) records the exercised native cases. Rapid panning, device recovery and paired performance acceptance remain pending.
