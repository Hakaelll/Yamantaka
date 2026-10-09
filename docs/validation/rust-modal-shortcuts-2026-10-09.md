# Modal keyboard isolation

Date: 2026-10-09

## Behavior

The external-link confirmation previously left reader shortcuts active. Ctrl+F, F3 and Shift+F3 could change reader state, and Ctrl+Shift+F could replace the reader with global search while confirmation was open. Global search could also leave a password dialog behind.

The connected application now routes keyboard commands through one shared reader/search shortcut module. Both password entry and external-link confirmation block those commands and page-arrow navigation. The router leaves modal input events available for the dialog, including Escape and Tab. F11 remains available to resize the presentation through fullscreen. Outside dialogs, document commands require the reader; focused text input retains arrow keys, while document find remains available. Global search takes precedence over reader commands in the same frame.

This change affects keyboard routing only. It does not certify other input, asynchronous document changes, external handler launch or configurable shortcuts.

## Verification

Three regressions run actual egui input frames through the production router:

- An open modal blocks global search, document find, match navigation and page navigation without consuming the dialog's events; F11 still produces fullscreen.
- Document shortcuts require the reader, and Ctrl+Shift+F routes to global search without also opening document find.
- A focused text field keeps page-arrow input and still permits Ctrl+F.

All 208 workspace tests pass. Warning-denied Clippy for every workspace target and formatting checks pass. Local logs are `build/verification/modal-shortcuts-tests.log` and `build/verification/modal-shortcuts-clippy.log`.

The tests exercise egui events and text-field focus, not OS keyboard injection or Narrator. Native dialog captures from the preceding checkpoint remain historical evidence for presentation; they are not captures of this source revision. Full rewrite, physical DPI, accessibility and release acceptance remain pending.
