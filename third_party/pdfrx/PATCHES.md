# Yamantaka patch for pdfrx 2.4.7

Source: `pdfrx` 2.4.7 from pub.dev, licensed under the included MIT
`LICENSE`.

Yamantaka changes:

- `PdfImage.createImage()` keeps the raw-image codec alive until
  `getNextFrame()` completes. The upstream callback helper disposes its codec
  before the Windows Skia engine returns the frame, leaving PDF pages blank.
- Preview-render failures are reported through `FlutterError` instead of being
  silently discarded.
- Progressive rendering keeps the previous/preview bitmap visible, refines
  only after an idle delay, cross-fades the final region, respects reduced
  motion, uses high-quality sampling and avoids a white placeholder flash.
- Visible pages enter the serialized rendering queue before the nearby cache
  ring. Work outside that ring is cancelled and cancellation-token bookkeeping
  is removed when complete or cancelled.
- Vertically ordered continuous layouts can binary-search the cache range, so
  paint scheduling is proportional to nearby pages instead of total pages.
- The final rendering scale is bounded through the host callback and aggregate
  preview/refinement/cancellation/tile-reuse events can be observed without
  exposing page or document data.
- The package version has the local suffix `+yamantaka.2`.
- Web-only WASM assets, examples and development files are omitted because
  Yamantaka currently targets Windows, Android and iOS.

Keep this directory isolated. Remove the path dependency and return to the
hosted package after an upstream release is visually verified on Windows.
