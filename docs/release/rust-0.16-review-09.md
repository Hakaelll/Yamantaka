# Yamantaka 0.16.0-alpha.1: Windows review 09

This local review candidate packages the native Rust runtime at
`ff9b58eb7ea493a3077218a15dc362cc697d1ebe`. It is not the final replacement
Release. Keep the Flutter installation and its original library available while
the remaining acceptance work continues.

## Changes since review 08

- Reading controls float over each pane and hide after a short pause. Revealing
  them leaves PDF position, scale and layout unchanged. Keyboard focus, open
  menus and active interaction keep them available.
- The optional electronic-ink profile uses white paper, light controls and
  single-page navigation without decorative scrolling or page shadows. Switching
  back restores the document's saved presentation settings.
- Native PDF reads and cache hits verify the loaded source version. Changed
  sources are rejected. A visible reading retains its native owner until it
  closes; closing and reopening loads the replacement source.
- Startup recovery handles damaged or unavailable library selections with
  reviewed choices while retaining complete libraries and operation records.
- Test captures await and clean up their own processes. Managed interactive
  previews replace earlier test previews.

The [reader guide](../user-guide/reader-responsiveness.md) and
[profile guide](../user-guide/reader-profiles.md) explain the reading controls and
source-change workflow.

## Package

Extract the complete ZIP into a writable folder and start
`yamantaka/yamantaka.exe`. Keep `pdfium.dll` beside it. A portable marker keeps
the independent Rust data and cache beside that executable. Flutter, Dart and
Rust are not required to run the package.

| Item | Value |
| --- | --- |
| Target | Windows 10/11 x64 |
| Archive | `yamantaka-0.16.0-alpha.1-windows-x64-portable.zip` |
| Archive size | 16,301,266 bytes |
| Archive SHA-256 | `48cc5e2a794e2c8a4462197474a18f7fe96e684401d7da56217f8f9a18ab6435` |
| Executable size | 30,189,568 bytes |
| Packaged files | 462 |
| SBOM components | 239, including Yamantaka |
| Application license | MIT |
| Authenticode observation | NotSigned |

The bundle includes pinned PDFium, licenses, Cargo.lock, instructions, a per-file
manifest, SHA-256 sums and a CycloneDX 1.5 inventory. The archive remains local
under `build/release/rust-0.16.0-alpha.1-review-09`. This checkpoint does not
publish a GitHub Release.

## Verification and remaining acceptance

All 56 retained workspace tests pass, together with workspace formatting and
all-target Clippy with warnings denied. Five serial captures use the executable
extracted from the ZIP. They cover dark/light PDF-cover galleries, hidden
reading controls, electronic-ink reading through OpenGL, and an idle wgpu reader.
Native control, scrolling-profile and bidirectional editor/copy probes also pass.

The original package passes a fresh verify-only check after execution on the
separate extracted copy. The SBOM passes the cached offline CycloneDX schema.
All 42 generated source PDF hashes remain unchanged.

The idle check observes zero intermediate egui updates during its two-second
interval. It does not measure GPU activity, energy, CPU use or interaction-time
percentiles. Complete functional and migration acceptance, physical DPI,
Narrator and monitor checks, Windows 10 execution and paired Flutter/Rust
performance measurements remain open in the
[acceptance matrix](../architecture/rust-rewrite-parity.md).
