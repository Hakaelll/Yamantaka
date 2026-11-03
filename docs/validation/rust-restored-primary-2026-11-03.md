# Restored comparisons prepare newly exposed primary pages

Saved comparison restoration now retains its confirmed reference offscreen while preparing the primary frame for the saved split. The primary stage shares the manual comparison planner, render admission, pixel allowance, and canonical split geometry. Its requests are bound to the primary preferences token. The reference remains owned by the existing native confirmation and close controller throughout preparation.

The saved context becomes available for valid continuity and binding snapshots after restoration. Presentation waits separately for primary images. Missing pages are staged outside the live primary surface, then transferred together before releasing the deferred reference. Existing images remain available. Queue-full delivery changes the consumer token and retries after a delay; source changes, closure, and errors use the existing cancellation and reference-close paths. Preparation does not run while reference confirmation or closure is busy.

## Executed evidence

- The extended workspace scenario restores a continuous, fit-width primary in a 1024×960 window and a side-by-side comparison at ratio 0.6. The original primary opening prepared page 0. The split required pages 0 and 1, and page 1 was staged before presentation. The first primary frame matched the planned 610.400024×882 logical viewport, scroll, zoom, and page set, with no missing images. Reference adoption had an image, its first paint was populated, and provisional paints remained zero.
- The original stacked Book workspace scenario still passed its context, panel-width, columns, binding, management, and comparison assertions. Its first primary frame also matched the plan without missing images.
- The manual continuous 5,000-page comparison still staged primary pages 1 and 2 and presented pages 0 through 2 without missing images. Its settled capture matched every decoded RGBA byte of the earlier manual checkpoint.
- Native debug compilation, workspace formatting, and workspace Clippy across all targets with warnings denied passed. All 39 generated PDFs retained their hashes. No dependencies, persisted formats, or standalone Rust tests were added.

The [status artifact](artifacts/restored-primary.json) records selected results and the executable hash. Local captures remain under `build/verification/restored-primary`.

The subsequent [synchronized restoration checkpoint](rust-saved-sync-2026-11-03.md) covers combined page, zoom and position following at first presentation. Broader synchronization, cancellation and pressure combinations, rapid input or resizing, physical accessibility/DPI, and paired performance measurements remain pending. These logical paint checks do not establish physical presentation latency or whole-rewrite readiness.
