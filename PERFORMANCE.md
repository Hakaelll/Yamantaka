# Performance and resource policy

The native reader virtualizes continuous pages and retains the previous image
until a replacement is ready. Visible rendering has priority over nearby-page
prefetch and optional background work. Settled views do not request continuous
UI repaint. Folder notifications wait on native events instead of polling UI.

Initial PDF cache budgets are 72 MiB for pages, 24 MiB for tiles and 8 MiB for
text. CPU payloads and governed PDF GPU resources require admission. Actual
texture dimensions respect the graphics device and the 16,384-pixel ceiling.
These accounting figures are not a measurement of total process or opaque driver
memory. Font/UI allocations and operating-system overhead have separate costs.

The owner accepted final validation on 2026-11-04. No new paired Flutter/Rust
p50/p95 dataset was supplied with that statement, and no quantitative speedup is
claimed here. Historical measurements, source revisions and corpus limitations
remain in the [validation record](docs/architecture/rust-rewrite-parity.md).

For future measurements, keep host, corpus and actions identical across versions;
report startup, first page, UI, PDF raster, GPU presentation, search, CPU, memory
and post-close resources separately. The original targets are 16.7 ms p95 for
light interaction at 60 Hz and 50 ms p95 for catalogue queries. Do not substitute
widget-construction timings for presented frames.
