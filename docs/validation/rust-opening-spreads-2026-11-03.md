# Primary opening prepares the complete facing spread

Primary opening now resolves saved preferences before planning its preview pages. The prospective context applies session restoration, then a matching pending tab's preferences, in the same order as reader adoption. The canonical domain spread policy determines the required pages, including Book mode's effective facing layout, cover pairing, and reading direction.

Each page has an independent request token and render-key check. The previous reading remains available until every required texture has uploaded. Admission can resume after queue pressure without resubmitting completed requests. A failed member cancels the whole candidate and releases its prepared images. Adoption transfers the complete set together. The opening raster budget remains capped at 2 MiB and is divided among the pages; GPU copies and staging retain their existing governor charges.

## Corrected paint observation

The earlier opening probe could finish after adoption but before the new viewport painted. It therefore could not establish completed-frame coverage. The probe now requires page painting after the adoption's recorded paint revision before checking the result. It also records which page textures were transferred.

With that correction, the previous single-page sequence still recorded zero empty-page paints. The facing-page baseline exposed the missing companion: only page 0 was prepared and this run recorded seven empty-page paint calls. The fixed sequence transferred pages 0 and 1 and recorded zero. These are functional counts for the exercised runs, not frame presentation or latency measurements.

## Executed evidence

- Native facing opening passed in a light Spanish 640×480 window with text at 200%, reduced motion, and RTL page placement. Rendered inspection confirmed the portrait first page on the right and its landscape companion on the left.
- A Glow sequence opened the second page of a different generated PDF. It deliberately failed the last member of the opening set, verified the previous reading remained available, and successfully retried both pages without empty-page paints.
- The existing workspace scenario now restores a Book tab whose raw stored layout remains single-page. It verified the effective facing layout and both prepared pages, exercising pending-tab preference precedence over the document defaults.
- The existing two-pane rotation sequence still passed without empty-page paints after its prepared textures moved into view.
- Equivalent whole-page pixel dimensions can now reuse a quarter-turned texture while preserving its actual raster key. Different resolutions still refine normally. In the verified 8×/90° case, the earlier implementation requested an intermediate 768×576 whole-page raster before its crop. The corrected implementation used the retained image and obtained the crop directly. The settled captures were pixel-identical. A separate higher-resolution case still requested and installed a larger raster over its retained preview.
- All 39 generated PDFs retained their hashes. Native debug compilation, workspace formatting, and workspace Clippy across all targets with warnings denied passed. Native capture checks were extended; no standalone Rust tests or dependencies were added.

The [status artifact](artifacts/opening-spreads.json) records the outcomes and executable hashes. Local images remain under `build/verification/opening-visible-pages`.

At this checkpoint, continuous opening still prepared only its requested page. The subsequent [initial viewport checkpoint](rust-opening-viewport-2026-11-03.md) replaces target-page and whole-spread planning with the canonical visible frame, including continuous opening. Reference-pane replacement, broader cancellation/pressure stress, physical input/accessibility, and paired performance acceptance remain pending. This checkpoint does not establish whole-rewrite or final-release readiness.
