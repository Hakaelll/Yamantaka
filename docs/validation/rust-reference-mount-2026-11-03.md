# Manual comparison prepares both panes before appearing

The manual Compare action now keeps the full primary reading visible while a pending comparison prepares its images. The pending view is bound to the primary preferences token and document. Changing readings, leaving the reader, closing, or toggling Compare again cancels its consumers without cancelling the primary token. Retained comparison context is discarded only after successful mounting.

Preparation uses the same split-rectangle function as live comparison. After the actual reader toolbar and navigator have laid out, the canonical primary layout computes the prospective frame. Existing primary images remain available; newly exposed pages are staged separately. The reference receives its own first image. Both sets must be ready before the split appears. Transferred images retain their actual raster geometry; only their delivery generation is rebound to the current primary surface.

The reference and primary staging pools each allow up to 1 MiB of initial raster pixels, subject to the configured page allowance and device limits. GPU, CPU, and transient copies retain their existing shared-governor charges. Changed primary page sets cancel old staging requests. Queue-full delivery replaces its consumer token and waits before retrying. Render admission and image installation are shared with reference replacement. No additional PDF worker, dependency, persisted format, or standalone Rust test was added.

## Executed evidence

- The native baseline mounted the initial reference without an image and recorded six empty-page paints. The corrected normal sequence mounted with its image ready and recorded zero. Its settled 1800×1200 capture matched every decoded RGBA pixel of the baseline.
- The first primary frame is also observed after mounting. The probe asserts exact agreement with the planned viewport, scroll, zoom, and visible pages, and checks that none lacks an image.
- A 5,000-page generated PDF was opened with the existing continuous and fit-width options in a 1024×960 window. The split exposed primary pages 0, 1, and 2. Pages 1 and 2 were staged before mounting. All three had images in the first primary frame, and the reference's first paint was populated. The fixture's actual PDF page count was checked independently.
- A light Spanish, rotated, stacked comparison with text at 200% passed in Glow. Its first primary viewport matched the planned 600×182 logical points, and both panes had images. The stacked and continuous captures were visually inspected.
- An injected reference raster failure left the primary image visible, reported the error, and mounted no reference. Cancellation after submission also mounted no reference, retained the primary image, and left the primary token uncancelled.
- Injected queue-full delivery recorded one retry, then mounted successfully without an empty reference paint or missing primary image. This is not a sustained queue-saturation benchmark.
- The existing different-document failure/retry and saved-workspace scenarios still passed. The workspace scenario uses a separate construction path and is a compatibility check, not initial-mount acceptance for automatic restoration.
- Native debug compilation, workspace formatting, and workspace Clippy across all targets with warnings denied passed. All 39 generated PDFs retained their hashes.

The [status artifact](artifacts/reference-mount.json) records executable hashes and selected results. Local captures are under `build/verification/reference-mount`. The earlier reference-replacement, continuous-opening, and rotated-region settled pairs were also rechecked by comparing all decoded RGBA bytes; all matched.

## Remaining acceptance

Automatic construction of saved comparisons still uses a separate path and needs the same initial presentation guarantee. Rapid changes to reading context or window size during preparation, broader failure and pressure stress, physical DPI/accessibility, paired performance measurements, and the full replacement Release gates remain pending. Logical paint checks and settled captures do not establish physical frame-presentation latency or whole-rewrite completion.
