# Native page resources — 2026-10-09

The executed WGPU reader now shares the PDF worker's memory governor. PDFium writes
RGBA into an admitted buffer; a page upload reserves both texture payload and
256-byte-aligned staging before creating resources. The CPU buffer, old texture,
replacement and transfer are independent live owners. Staging survives its upload
fence; retired textures survive the last submitted draw and retirement fence.

The global page category remains 72 MiB. The running UI uses a 24 MiB CPU cache
subset and 14 MiB whole-page preview cap to leave room for replacement/staging.
The default worker API still supports its existing configurable limits. GPU
dimensions are bounded by both the actual adapter/device and 16,384 pixels.
Preview refinement, tracked live text, UI atlases, Glow accounting and device-loss
reconstruction remain pending. Opaque driver allocations are not counted as RGBA
payload bytes. This is not complete global-memory acceptance.

## Tests actually executed

`cargo fmt --all`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --offline -- -D warnings`, and
`cargo test --workspace --offline -- --nocapture` passed: **63 tests, zero failures,
zero ignored**. The real headless DX12 adapter was **Intel Iris Xe Graphics,
IntegratedGpu**. The upload test checks nonaligned 65×3 RGBA rows by exact native
readback, joint admission rollback, callback-held staging and texture leases,
retirement between frames, and total zero after the last CPU owner drops.

Logs are in `build/validation/rust-stage2/`:

- `tests.log` SHA-256 `4792220AADC869FBAE09E997CCFBABFAA1FFD0DE6E34B9A65630B64E082F3A5D`.
- `clippy.log` SHA-256 `8F0C100E1DAB774BAFA1892BBE58C6950080378EE1B39CF04272B5DB995AA4AC`.

The later UI changes add explicit page retry after failed admission, capture
memory diagnostics and reset of retry state on navigation/zoom. Formatting,
Clippy and the application build also passed after those changes.

## Native integration regression

Four fresh native captures exited successfully: normal reader, text panel,
640×480 with text at 200%, and OpenGL reader. The first three WGPU captures have
**zero changed RGBA pixels** against the previously reviewed captures. This
confirms that the native upload changes preserve that composition and those PDF
pixels; it does not extend the visual review to other documents or flows.

The [native-upload manifest](../screenshots/rust-native-upload/capture-manifest.json)
records the exact executable and source hashes. Adjacent control records name
the presenter and report tracked CPU/GPU/transient counters with their limited
scope. The normal reader capture reported 6,136,408 CPU bytes and 3,255,208 GPU
bytes, 9,391,616 total, with transfer staging already released. OpenGL is explicitly
labelled as using the older egui upload path. Its appearance passed; its equivalent
resource accounting is pending. Metadata scan: four rasters, zero missing origins.

Native PDFium tests now render regions directly into tile-sized bitmaps. Twenty
full renders are compared with 180 tiles across explicit/intrinsic rotations,
mixed sizes, text/vector geometry and scans. Bounds, cancellation, retained leases
and original PDF hashes are checked. PDFium can change antialiasing at clipped
glyph edges; tests require mean channel difference below 0.01/255, fewer than
0.01% changed channels, exact alpha and significant differences only near edges.
Bit-exact clipped glyphs and UI halo composition are not claimed. Interactive
form parity, in-call native cancellation and complete continuous UI are pending.
