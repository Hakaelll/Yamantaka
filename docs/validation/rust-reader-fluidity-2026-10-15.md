# Stable PDF presentation and progressive previews

The reader now keeps a stable viewport while background work completes. The
global processing row has been removed. Import and opening cancellation remain
available through Tools and Reading options without inserting another row above
the document.

## Render behavior

A page without an image first requests a raster with a maximum side of 768
physical pixels, subject to the existing device and memory limits. Small pages
need only one request. The first raster stays visible while the reader requests
the final resolution after the target has remained stable for 120 ms.

Zoom and viewport changes update the desired target without immediately
requesting a full raster for every intermediate state. A pending initial preview
can finish during resizing. Existing request tokens and full render keys still
reject cancelled, foreign-page and obsolete consumer results.

The retained texture uses UV coordinates that account for its original rotation
and the current page rotation. It therefore turns immediately with the page
geometry and annotation overlays while the replacement raster is pending.

The settling timer requests one future repaint when needed. Waiting for native
results uses the existing worker notification. A completed target schedules no
further repaint through this policy.

## Navigator geometry

The first native capture round exposed a second source of movement: the text
panel followed the width of asynchronous content. In that capture the PDF
viewport width changed from 1159.53125 to 1182.40625 logical points between its
preview and refinement. These values describe geometry, not render latency.

A regression using the real egui panel failed before the correction, changing
the remaining viewport boundary from 220 to 467 and back to 220 logical points
as content widths changed. The navigator now fills its allocated panel and uses
scrolling for overflowing content. The user can still resize it within its
existing 220–500 point range. The regression and the final native capture both
confirm a stable PDF viewport while content arrives.

## Import ownership

Import now has its own cancellation scope. Starting, replacing or cancelling an
import does not cancel the current reader's render token. Replies only finish
the matching import. Its failure does not mark an unrelated reader render for
retry. Cancelling refreshes the catalogue so already committed imports remain
visible.

## Verification

- All 458 workspace tests passed, including 101 UI tests. Formatting, workspace
  Clippy with all targets and warnings denied, and the native application build
  passed.
- New scheduling tests check the small initial raster, final refinement, no
  duplicate request while native work is pending, no idle timer, resizing during
  the first preview and 30 rapid target changes producing only the final full
  refinement request. This is a deterministic policy test, not a timing benchmark.
- Texture tests check quarter-turn UV mapping and unchanged coordinates when
  both image and page already use the same rotation.
- The real panel regression checks changing asynchronous content widths.
- Import ownership tests cover supersession, foreign replies, cancellation,
  completion and shutdown without cancelling an independent reader token.
- Six native wgpu captures cover dark, light Spanish with the Text panel,
  facing pages, continuous layout, rotated sepia and 640×480 with 200% text.
  Their traces record successfully submitted requests and installed images.
  Seven visible page instances use a retained-preview refinement; the compact
  case uses one small raster. Every page reaches its final target, and each
  case keeps its viewport unchanged between preview and refinement.
- The capture verifier checks 71 UI source hashes, executable and lockfile
  hashes, capture hashes, request/install correspondence, image retention,
  stable geometry, tracked-memory limits and zero remaining upload staging.
  It rejects the first-round text-panel trace because its viewport changed.

The final executable SHA-256 is
`A28F6BAAB6AEAC1C58CB0830E134DD00B1199C347D1B21EC843C1CF2707C9722`.
Published images and trace files are in
[`rust-reader-fluidity`](../screenshots/rust-reader-fluidity/capture-manifest.json).
Their provenance metadata identifies isolated synthetic data. The original
text fixture SHA-256 remains
`1545B0DFC2BA930DCB22D4EB2C7E6FFB378B1AC5E752D723016C077BF91484E5`.

```powershell
cargo test --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p yamantaka --bin yamantaka-rust --locked
./tool/capture_rust_ui.ps1 -Executable build/verification/yamantaka-reader-fluidity-final.exe -OutputDirectory build/rust-visual/reader-fluidity-final -CaseNames @('reader-dark','reader-text-light-es','reader-facing','reader-continuous','reader-sepia-rotated','reader-small-200')
python tool/verify_rust_reader_fluidity.py build/rust-visual/reader-fluidity-final build/verification/yamantaka-reader-fluidity-final.exe --published docs/screenshots/rust-reader-fluidity
```

## Remaining acceptance

These changes do not prove a frame-time improvement or universal smoothness.
The captures use a development build and a small synthetic PDF. Release p50/p95
traces with heavy vector, scanned and long PDFs, import concurrent with reading,
device loss, physical DPI and Narrator remain pending. PDFium is unchanged.
Region refinement, nearby-page prefetch and retaining a complete reader through
the entire first-raster transition also remain pending.

Opening still waits for complete page geometry and stored preferences before
its first reader raster. Import metadata still needs evidence-based title and
author validation, front-matter proposals and review controls; the existing
metadata fields alone are not proof of accuracy. Automatic cover detection and
manual-choice precedence retain their earlier verification. The full rewrite
and replacement release are unfinished.
