# Native visual correction batch — 2026-10-09

The user rejected the first Rust screenshots as too simple. This batch changes
the executed native library, reader and settings, retaining physical 3D covers
and the existing spring response. It is a section of the full rewrite, not its
completion or a replacement release.

## Executed evidence

`tool/capture_rust_ui.ps1` runs the native executable against an isolated in-memory
synthetic collection. Sixteen cases exited successfully: dark/light library,
list, selected/focused list, details, Spanish, compact, 200% text, pointer tilt,
reduced motion, OpenGL, light settings, settings at 200%, reader, reader text panel
and reader at 200%. No personal store was opened.

The host produced 1.25 pixels per logical point normally, and 2.5 with text zoom
200%. Large captures are 1800×1200 pixels; the 640×480 window cases are 800×600.
Text zoom changes logical composition; it is not a physical DPI test.

The published [capture manifest](../screenshots/rust-stage2/capture-manifest.json)
records arguments, executable/Cargo.lock/source hashes, viewport and pixels per
point. PNGs retain origin metadata. Both original and metadata-bearing PNG hashes
are recorded. The origin scan reports **16 rasters, zero missing**.

Adjacent `controls.json` files contain actual custom-control rectangles and clip
bounds. In the compact reader the five controls occupy x=18..292, y=8..48 within
the 320×240 logical viewport; none overlap or exceed their clip. This checks those
rectangles, not the whole accessibility tree or keyboard interaction.

## Independent bounded review

The initial review returned **fix**. The subsequent independent confirmation
resolved all eight named corrections and the compact-reader regression:

- Compact continuity strip restores the collection's prominence.
- Distinct title/art compositions and local serif/bold voices remain on the rigid
  cover plane.
- The lower inspector preserves the gallery's five columns and horizontal positions.
- Inspector title, real progress, actions and original filename are separated.
- Library and settings at 200% show meaningful content; the rest remains scrollable.
- Miniatures communicate through pigment/art without broken title fragments.
- Selected list rows use fill and a marker; keyboard focus uses a separate outline.
- DESIGN.md describes the current build, including corrected preview pigments.
- Compact-reader controls share one bar without overlap.

The final confirmation returned **remaining: clear; disposition: ship**, solely
for these scored corrections. No approved comp or full QUALITY BAR card was
available. This is not whole-interface or whole-rewrite approval, and does not
certify Narrator, physical DPI, multiple monitors, full parity or critical flows.

The captures precede the subsequent native page-upload integration. They preserve
the exact reviewed build through source/executable hashes; new GPU behavior must
receive its own regression evidence.

## Reproduction

Generate the PDF fixtures, compile the Debug application, then run:

```powershell
./tool/capture_rust_ui.ps1
./tool/publish_rust_captures.ps1
```

The publisher requires the installed Impeccable metadata tool and checks source
PNG/control hashes before publishing. It never treats a stale PNG as a successful
capture. Capture and publication outputs are separate so embedded origin metadata
does not alter the native capture hashes.
