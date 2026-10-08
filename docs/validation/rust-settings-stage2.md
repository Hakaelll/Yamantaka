# Rust settings: second design stage

The native settings surface now uses four named categories: Appearance,
Accessibility, Language and Privacy. Categories are retained in egui temporary
memory for the current session. At 850 available logical points the settings-only
navigation uses a narrow left column; below that threshold it wraps horizontally.
The entire surface scrolls vertically, including its heading and navigation.

Appearance contains two semantic radio choices with authored geometric diagrams
of the library and PDF reader. Selection is available by mouse and keyboard, has
an explicit radio indicator and an independent focus outline. The border settles
over 180 milliseconds after selection; reduced motion uses a zero-duration state
change. There is no idle animation or simulated action button.

Existing dark-mode, text-scale, reduced-motion and language settings continue to
use `settings_changed`, preserving their preference keys and save behavior. Text
size has a slider, labelled numeric input and 100/150/200 percent presets. Copy is
provided in Spanish and English. Native controls wrap and previews stack when
there is insufficient width for two choices. The module does not add app state.

Integration validation: formatting, compilation and Clippy without warnings ran
successfully using the approved native-tool escalation. Native captures include
light settings and 640×480 with text at 200%. The cramped composition shows a
theme choice in the first viewport; the heading/category controls are compact
and the complete surface scrolls. The preview pigments match current covers.
The independent bounded correction review resolved this change; its scope and
manifest are in `rust-visual-stage2.md`. Narrator, physical DPI, complete settings
parity and release readiness remain pending.
