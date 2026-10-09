# PDF destination resolution

The domain resolves a PDF outline destination into a canonical ReaderPosition
and an effective zoom without mutating navigation or preferences. The caller
must supply the current viewport's top-left point in PDF coordinates and its
effective zoom. Null coordinates retain those PDF values across page changes;
null or zero destination zoom retains the effective current zoom.

Resolution covers XYZ, page fitting, width fitting, height fitting, rectangle
fitting and the three content-bound fitting modes. It projects all rectangle
corners through native page geometry and user rotation. Rectangle and bounding
fits center the unused axis. Layout margins reduce the fitting area. Horizontal
left-edge targets are converted to viewport-center anchors, and normalized page
limits and the finite canvas still constrain navigation near edges. The complete
proposed frame is validated before returning a result. Malformed rectangles,
non-finite inputs, invalid pages and nonpositive zooms return errors.

For content-bound modes, the PDF worker unions PDFium page-object bounds and
intersects them with the visible page boundary. PDFium's page bounding-box API
itself only intersects MediaBox and CropBox, so it cannot substitute for content
bounds. See the [PDFium API contract](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/public/fpdfview.h).
Empty or degenerate content falls back to the corresponding page fitting mode.
Complex forms, clipping, invisible content and broad document compatibility still
need acceptance coverage beyond these synthetic paths.

A request permits at most 100,000 content-object visits across all its bounding
fit destinations. The worker checks cancellation and document epoch between
objects and rejects work that would exceed that budget. Repeated destinations
count toward the same request budget. The initial text reservation increases
from 4 to 6 MiB to accommodate the larger owned destination records and traversal
space. It remains subject to the shared 8 MiB text budget and shrinks to retained
result capacity. SDK allocations remain outside measured Rust allocation coverage.

Four domain tests check literal zoom values for every fitting mode, XYZ null and
zero behavior, user rotation, invalid inputs, empty content and free-axis
centering. A fourth native PDFium test checks actual content bounds, crop
intersection and unchanged source hashes. The existing native limit test adds a
100,001-object page and a repeated-destination case whose cumulative visits
exceed 100,000. Both return a PDF failure and release the outline reservation.

Validation on 2026-10-09:

```text
cargo test --workspace --locked                              155 passed
cargo clippy --workspace --all-targets --locked -- -D warnings passed
cargo fmt --all -- --check                                   passed
```

The final four native outline tests passed again after extending limit and hash
fixtures. The resolver is not connected to the reader panel yet. Reader integration
must obtain the current page's native coordinate space, use its actual viewport
and effective zoom, apply the destination and preferences together, and verify
zoom bounds and navigation history. The outline tab and interactive captures,
large-outline latency and full release acceptance remain pending.
