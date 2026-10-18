# Reader loading without a page-number overlay

The reader no longer draws a page number in the middle of an empty page while
waiting for its first raster. The page surface remains plain until PDF pixels
arrive. Existing PDF pixels remain visible during refinement. Page navigation
and the accessible page description retain their numbering.

This removes the text-paint operation from the shared page viewport, so the
change applies to single-page, facing, and continuous layouts.

The UI's 108 tests pass, formatting passes, all-target UI Clippy has no warnings,
and the native build succeeds. Six native captures check dark, light Spanish,
facing, continuous, rotated sepia, and narrow text at 200%. The reader verifier
checks real render submissions, installed previews, retained images during
refinement, stable viewport dimensions, and the governed memory limit.

The captures show settled pages. They do not measure loading latency or capture
the transient empty-page state. Removal of that state's centered number is
verified directly in the shared drawing branch. Physical DPI, Narrator, and
paired release performance acceptance remain pending.
