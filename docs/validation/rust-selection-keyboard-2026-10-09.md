# Keyboard selection checkpoint

The selectable PDF page now accepts keyboard input when it has focus. Clicking or dragging requests focus and the page displays a violet focus outline. Tab traversal can reach the same native egui interaction. The page's accessible label and tooltip explain Shift+arrows, Ctrl+A and Ctrl+C in English and Spanish.

Left/right move through extended Unicode grapheme boundaries. Shift extends the range while preserving its anchor. Ctrl+left/right move through word boundaries; Home/End use text newline boundaries, including CRLF, while Ctrl+Home/End use the current page boundaries. Ctrl+A selects the current page. An unmodified arrow collapses an existing selection toward its corresponding edge. Keyboard movement adjusts an anchor that was clicked inside a combined character to a grapheme boundary.

Selection keys are consumed before the application's page-navigation router. Input from a different focused widget and input while a password/external-link confirmation is open remain untouched. Copy and Escape explicitly recognize page focus, so making the page focusable does not disable passage copying. Text editors retain their own copy events.

`unicode-segmentation` 1.13.3 was already present transitively in Cargo.lock. The UI now declares the same pinned version directly. No package version or checksum changed. Segmentation iterators borrow the governed native snapshot and do not materialize another text or boundary vector.

## Verification

Three new UI regressions cover accented combined characters, a joined emoji, a pointer anchor inside a grapheme, reversed selection, CRLF lines, words, full-page selection, actual egui key events, page-navigation suppression, modal/other-focus preservation, copy routing and Escape. All workspace tests pass, as do formatting and warning-denied workspace/all-target Clippy. The final workspace log is `build/verification/selection-keyboard-tests.log`.

Three native captures exercise pointer focus followed by actual Ctrl+Home and Shift+Right input through the production application route. Dark, clockwise-rotated and 640×480 at text scale 200% all select native character range `[0, 0]` through `[0, 1]` while remaining on page zero. Action bounds remain inside their clips. These captures do not assign the resulting selection directly.

Local artifacts are under `build/rust-visual/selection-keyboard-native`. The manifest SHA-256 is `66A0BD48CCC74973BBB91BF8036BC96E38E119551ADF56663F5CF6433B431BC6`. Executable, UI source, Cargo.lock, image and control-state hashes were verified against the final capture run. Synthetic data, executables and capture artifacts remain outside the public checkpoint.

## Remaining acceptance

Movement follows logical native text order. Visual bidirectional caret navigation, vertical movement, pointer word/grapheme selection, a collapsed-caret presentation, multi-page keyboard/drag acceptance and autoscroll remain pending. Home/End follow explicit text newlines; inferred visual PDF line navigation is not implemented. Large-page interaction performance, OS clipboard execution, Narrator and physical monitor DPI remain pending. This checkpoint does not complete annotation parity or release acceptance.
