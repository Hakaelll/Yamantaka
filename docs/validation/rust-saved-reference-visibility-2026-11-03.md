# Saved comparisons hide their provisional reference

Automatic comparison restoration now keeps the primary reading visible while the reference opens, obtains its image, and completes native confirmation. The provisional reference is excluded from comparison snapshots. The restoration controller releases it for presentation only after applying the saved context and checking image readiness. Failed deferred workspace restoration uses the existing reference-close protocol rather than leaving an invisible reference behind.

The native workspace scenario passed its existing saved-page, rotation, zoom, ratio, panel-width, column, and binding assertions. It recorded zero provisional reference paint calls, an image at adoption, and no empty reference paints. The manual comparison scenario also passed. Native debug compilation, workspace formatting, and workspace Clippy across all targets with warnings denied passed. No standalone tests, dependencies, or persisted formats were added.

The [status artifact](artifacts/saved-reference-visibility.json) records selected results and the executable hash. Captures remain local under `build/verification/saved-reference`.

This closes the provisional reference display gap in the exercised restoration. The subsequent [restored-primary checkpoint](rust-restored-primary-2026-11-03.md) adds automatic preparation of newly exposed primary pages before the split. Broader automatic restoration failures, cancellation, physical accessibility, and performance acceptance remain pending. These checks do not prove whole-comparison frame continuity or replacement Release readiness.
