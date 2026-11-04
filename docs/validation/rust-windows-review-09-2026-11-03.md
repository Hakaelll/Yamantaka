# Windows Release review 09

The optimized portable candidate builds runtime commit
`ff9b58eb7ea493a3077218a15dc362cc697d1ebe` with unchanged runtime sources.
The [English candidate notes](../release/rust-0.16-review-09.md) describe its
changes, installation, exact archive hash and remaining acceptance.

The existing packager completes its locked, offline MSVC Release build with
static C runtime linkage and build-path remapping. A fresh verify-only pass
checks the original bundle after execution checks use a separate extracted ZIP.
The 462-file inventory, archive contents, sizes, hashes, pinned PDFium,
build-profile privacy scan, PE import checks, ten icon frames and version resource
pass. The SBOM contains 239 components and validates with PowerShell `Test-Json`
against the cached CycloneDX 1.5 schema with bundled offline references.
Authenticode inspection reports `NotSigned`.

Five serial native captures finish with code 0. Both galleries retain ten
PDF-derived covers. The reader captures exercise floating hidden controls and
electronic-ink presentation through the alternate OpenGL compositor. The wgpu
idle capture observes zero intermediate updates over 2,001 milliseconds, without
input or lost readiness. Separate extracted-executable probes pass control
visibility/focus, electronic-ink wheel and drag behavior, and bidirectional
editing/copy checks. All 42 generated source PDFs retain their hashes.

The two gallery screenshots are published with synthetic provenance for the
rewritten English README. The contribution guide now covers the native workspace,
fixture setup, focused checks, isolated previews and public-safe checkpoints.
All local README and contribution-guide links resolve.

The [result artifact](artifacts/windows-review-09-2026-11-03.json) binds these
checks to the executable, package and source inventory. Public native evidence
uses generated fixtures. Raw captures remain under `build/verification/review09-*`.
The package remains under `build/release/rust-0.16.0-alpha.1-review-09`.

This review does not establish full parity, physical accessibility, Windows 10
execution, GPU/CPU/energy behavior or paired Flutter/Rust performance. The
[acceptance matrix](../architecture/rust-rewrite-parity.md) continues to govern
the final replacement Release.

## User follow-up: 2026-11-04

The user reported no issues when asked about review 09 with two monitors. This
is a user observation for that build. The report does not specify a test
procedure, DPI scales or Narrator results, and does not cover the later review-10
runtime. The physical acceptance row records this limited scope.
