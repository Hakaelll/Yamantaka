# Yamantaka patch for pdfrx 2.4.7

Source: `pdfrx` 2.4.7 from pub.dev, licensed under the included MIT
`LICENSE`.

Yamantaka changes:

- `PdfImage.createImage()` keeps the raw-image codec alive until
  `getNextFrame()` completes. The upstream callback helper disposes its codec
  before the Windows Skia engine returns the frame, leaving PDF pages blank.
- Preview-render failures are reported through `FlutterError` instead of being
  silently discarded.
- The package version has the local suffix `+yamantaka.1`.
- Web-only WASM assets, examples and development files are omitted because
  Yamantaka currently targets Windows, Android and iOS.

Keep this directory isolated. Remove the path dependency and return to the
hosted package after an upstream release is visually verified on Windows.
