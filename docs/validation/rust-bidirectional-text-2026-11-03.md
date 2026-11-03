# Bidirectional text layout and editor mapping

The unmodified egui 0.36.2 baseline reproduced incorrect logical glyph counts and editor positioning on this host. The Arabic sample has 13 characters but produced 19 layout glyphs. End, three Left events and insertion appended `x` to the text instead of inserting it inside the final word. The Hebrew sample has 12 characters but produced 18 glyphs.

The [pinned local text patches](../architecture/native-text-layout.md) correct the shared presentation layer rather than rewriting stored strings. They cover paragraph levels, line whitespace reset, font/script/direction runs, composite outline clusters, visual row placement, logical hit testing and selection stretches. Installed Segoe UI and symbol fonts extend the existing fallback families.

## Executed evidence

- A standalone registry baseline used original egui/epaint 0.36.2, the same font setup and the project's pinned shared dependencies. It records the incorrect glyph counts, visual positions and editor result.
- The native application probe verifies Arabic, Hebrew and mixed Latin/Hebrew visual order against literal expected character indices. Logical strings, character counts and hit tests remain consistent. Arabic combining marks, decomposed Latin accents and composed characters retain one layout glyph per logical character.
- The probe verifies Arabic joining within a Hebrew/Arabic line, actual trailing spaces in wrapped LTR and RTL rows, and two visual selection stretches for a logical range crossing a direction boundary.
- Scripted events in the native TextEdit widget now insert `x` at the expected logical position after End and three Left events. Its copy command returns the logical text. These checks do not drive a physical keyboard or the Windows clipboard.
- Native wgpu gallery captures complete in dark English and light Spanish with Arabic, Hebrew and mixed titles. A Glow capture completes with the same synthetic titles. The captures were inspected.
- Every decoded RGBA byte of the final Latin gallery matches the frozen pre-change executable at the same actual 1920 by 1200 dimensions. The baseline and patched Latin glyph positions, advances and dimensions also match exactly.
- Offline debug compilation, workspace formatting and workspace Clippy across all targets with warnings denied pass. The five existing publication-guard checks pass. Additional self-checks verify that modified license blobs remain rejected and that release source inventory covers every vendored Rust source, manifest and license.
- All 39 generated PDFs retain their hashes. No application unit tests were added. One pinned runtime dependency, unicode-bidi 0.3.18, was added. The local egui/epaint versions remain 0.36.2.

The [status artifact](artifacts/bidirectional-text.json) records selected results, baseline discrepancies and the current executable hash. Local captures and probe records remain under `build/verification/bidi`.

## Remaining acceptance

This is scoped native evidence, not complete Unicode conformance or physical accessibility certification. Wider scripts and fonts, complex multi-paragraph selections, IME interaction, physical Narrator, and interaction/performance acceptance remain open. The review bundle must be rebuilt from the current sources before any later Release verification.
