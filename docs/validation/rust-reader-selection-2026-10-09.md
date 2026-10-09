# Reader pointer selection checkpoint

The native reader now provides a Select text action in Reading options. Dragging over PDF character geometry selects an ordered passage and paints its regions through the same page-coordinate mapping used by links and search. Copy passage sends the validated Unicode passage to egui's clipboard output. Clear selection, Escape and Done remove the selection; Done also releases its text snapshots.

Text extraction remains asynchronous on the existing PDFium worker. Selection and the Text panel share snapshots instead of requesting duplicate page text. At most two extraction requests are pending. Leaving a document cancels pending consumers and releases retained results. Selected pages remain retained when the viewport moves; copying refuses incomplete page ranges. Failed requests expose an explicit retry action. Selection mode takes precedence over PDF link activation while active, and password/external-link confirmations block selection input.

## Evidence

Four UI tests cover bounded admission, document replacement and cancellation, retained selected pages and incomplete-copy rejection, caret progression through all four user rotations with synthetic LTR/RTL geometry, and actual egui pointer press/move/release events through the production selection function. The latter preserves the press anchor and stops extending after release. These supplement the existing domain and real PDFium selection/geometry tests.

Workspace tests, `cargo fmt --all --check` and warning-denied workspace/all-target Clippy passed. A separate native executable was built without replacing the user's running preview. Its two Cargo warnings concern the explicit alternate output path; warning-denied Clippy passed independently.

Five native synthetic captures cover dark, light/Spanish, clockwise rotation, 640×480 at text scale 200%, and a scanned page without text. The four textual cases report the selected native range `[0, 0]` through `[0, 47]`; the scanned page reports zero characters and no selection, with Copy passage disabled. Captures inject pointer events into the same native egui input route rather than assigning a completed selection directly. Capture completion requires a nonempty selection or a ready page without any character geometry.

Local evidence is under `build/rust-visual/selection-native`. Its capture manifest SHA-256 is `CFA093AAE465655F79AC77F41A1AF13A99C74F8C1D635F393CD5CD8866F7914B`. It records the executable, UI source, Cargo.lock, image and control-state hashes. These artifacts use isolated synthetic data and remain outside the public checkpoint.

## Remaining acceptance

This checkpoint does not establish keyboard range selection, grapheme/word selection, mixed-direction shaping, rotated individual-glyph hit testing, multi-page drag/autoscroll acceptance or selection performance on large text pages. OS clipboard execution, Narrator, physical monitor DPI and multi-monitor checks remain pending. Annotation creation, editing, global review, export and the page-note fallback still need connecting and verifying. The complete rewrite and release acceptance remain open.
