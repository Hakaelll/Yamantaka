# Native electronic-ink reader profile

Settings → Accessibility now connects Standard and Electronic ink profiles through the same versioned local preferences used by other native settings. The exact Flutter `visualProfile: electronicInk` value is recognized. Unknown profile values remain stored and use Standard presentation until the user makes an explicit choice.

Electronic ink uses single-page canonical navigation, a white canvas, light reader controls and flat PDF pages. Programmatic scrolling is immediate. The local egui override can finish a discrete wheel movement in the current update and disable drag inertia; both reader panes opt into those controls. Ctrl-wheel zoom and Shift-wheel horizontal scrolling retain their modifier semantics. Trackpad phases received from Windows remain intact.

The profile derives effective presentation preferences from each document's saved preferences. It does not replace saved book mode, layout or paper environment. Opening preparation, active navigation, page composition and comparison use the same effective policy. Returning to Standard rebuilds the presentation while retaining existing page images until their replacements arrive. Profile controls wait for document opening or comparison preparation to finish. The PDF worker does not reopen a document for a profile change.

## Executed evidence

The [result artifact](artifacts/electronic-ink-profile.json) binds the native executions to their executable and changed-source hashes. The cases cover a standard continuous reader, electronic ink over continuous and facing/book preferences, comparison, the OpenGL compositor, a live return to Standard, and the compact Accessibility selector at 200% text zoom. Every process exited with code 0 and was confirmed terminal. The selector's recorded control bounds fit inside its clipped viewport.

The live return asserts equality of the saved document preferences before and after the profile change. It also saves and reloads both profile values through the production atomic preferences implementation in an isolated synthetic store. The normal reader and live-return screenshots match the earlier standard reader pixel for pixel, using explicit RGBA byte comparison. All 42 generated source PDF hashes remain unchanged.

The separate native input probe feeds raw egui wheel and pointer events without controlling the operating-system mouse. Its default-wheel and default-drag cases are positive controls. The electronic-ink path applies the complete 40-point line movement in one update, with zero movement in the next two updates. Ctrl zoom completes in that update too. Drag offsets stop at release with inertia disabled; the positive control continues moving. This probe adds no standalone Rust tests.

Locked offline builds, workspace formatting and scoped all-target Clippy with warnings denied passed. The three edited vendored egui files also passed their direct formatting check. Raw PNGs, control records, stderr and complete manifests stay local under `build/verification/eink-profile-final-runtime`; the earlier standard reference stays under `build/verification/eink-profile-before`.

## Reproduction and limits

```powershell
./tool/capture_rust_ui.ps1 -Executable build/verification/yamantaka-eink-profile-05.exe `
  -OutputDirectory build/verification/eink-profile-final-runtime `
  -CaseNames @('eink-standard-reader','eink-continuous-reader','eink-book-reader',
               'eink-reference','eink-glow','eink-return-standard','eink-settings-small')
```

The native input probe uses `--demo --verify-reader-profile-input=<result.json>`. Missing output, failed assertions or a nonzero process exit invalidate its result. Captures run serially and clean up their own child on timeout.

These are synthetic debug-runtime checks. They do not certify physical electronic-ink refresh behavior, touchscreen drivers, Windows trackpad momentum, Narrator, physical DPI, every opening/comparison race, Release presentation times or the full rewrite. The acceptance matrix remains Partial.
