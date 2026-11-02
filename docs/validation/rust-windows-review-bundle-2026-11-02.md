# Native Windows Release review bundle

This checkpoint prepares the optimized Rust application for local Windows x64 review. It does not approve a final replacement Release or retire Flutter. The feature-parity, physical accessibility and comparative performance gates remain open.

## Package contents and provenance

`tool/package_rust_windows.ps1` builds the explicit MSVC target with locked, offline Cargo resolution, a static C runtime and remapped workspace/profile paths. It refuses to overwrite a prior output directory. The generated application folder contains `yamantaka.exe`, the pinned PDFium DLL, application MIT terms, Cargo.lock, user instructions, third-party notices and a CycloneDX 1.5 component inventory. A portable candidate also contains the Rust-specific portable marker.

The outer directory contains the ZIP, SHA-256 sums, the per-file manifest and verification reports. Archive entries are sorted and use fixed timestamps and permissions. This supports repeating archive construction from identical inputs on the same host; it does not claim reproducible compilation across machines or toolchains.

The manifest records its base source commit, whether runtime sources differ from that commit, and hashes of the runtime sources and packaging inputs. It lists every shipped file, size and SHA-256. The component inventory includes normal and build dependencies reachable for the Windows target, excluding dev-only dependencies. Build/proc-macro components are marked separately from runtime components. Cargo archive checksums identify source archives, not compiled output bytes.

Both the PowerShell entry point and the Python packager perform a current Cargo build before assembly. The Python implementation owns the build environment without modifying its parent process. It compares input hashes before and after compilation and again before writing the manifest, rejecting a tree that changed during the operation.

Some published crates omit their workspace-level license files. Seventeen pinned package versions use nine upstream notice files retrieved from eight exact repository commits. Their URLs and hashes are recorded in `tool/rust_license_sources.json`; the separate fetch step validates each download before caching it. Packaging verifies the cache without network access and fails on missing or changed license bytes. Existing crate notice sets, PDFium notices, the bundled font license and the Rust standard-library notice catalog are copied into the bundle. These inventories and copied notices are not a legal compliance certification.

The owner selected MIT for Yamantaka. The root LICENSE and the six workspace package declarations now state those terms. Third-party software retains its own terms.

## Build-path handling

Development fixture paths previously embedded the compiler's workspace directory through unconditional `CARGO_MANIFEST_DIR` strings. The fixture-root helper now keeps that compile-time default in debug builds. Release builds use an executable-relative QA directory. An absolute `YAMANTAKA_QA_UI_ROOT` override is accepted only with both `--demo` and `--capture=...`. Ordinary application launches do not use that override. Release PDFium loading requires the adjacent packaged DLL.

This allows native Release captures against repository-generated fixtures without shipping development documents or build-host profile paths. The targeted privacy scan checks the current build profile path in UTF-8 and UTF-16, including slash and JSON-escaped forms. It does not prove absence of every possible personal value in arbitrary files.

## Executed evidence

- Release build for `x86_64-pc-windows-msvc`: passed. The application is a GUI-subsystem AMD64 PE image, 29,010,432 bytes.
- Full packaging and verify-only commands: passed. The ZIP is approximately 15.9 MB, with 457 shipped files and 238 inventory components including the application.
- Pinned PDFium, manifest sizes and hashes, exact ZIP contents, duplicate-entry checks, targeted build-profile scan and current-host PE import resolution: passed. The two shipped native binaries import no external `vcruntime`, `msvcp` or `concrt` library.
- Executable resource inspection: all ten icon frames match the generated ICO; ProductVersion matches `0.16.0-alpha.1`.
- Two assemblies from identical current inputs produced byte-identical ZIPs. This checks archive construction on this host, not compilation reproducibility on another machine.
- The SBOM passed the [official CycloneDX 1.5 schema](https://cyclonedx.org/schema/bom-1.5.schema.json) using [PowerShell Test-Json](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/test-json). The current candidate retains the validated shipped file inventory.
- An extracted ZIP opened the generated PDF and installed real page images with both native wgpu and OpenGL presentation. Both processes exited successfully with settled page ownership. A separate extracted-library run produced all ten PDF-derived covers with no pending uploads. These are native execution checks, not a statement about physical interactions or frame times.
- All eleven generated source PDFs retained their hashes. No personal PDF or library is included in the published evidence.
- Altering a shipped README caused verification to reject its hash. The original was restored and reverified. Temporarily removing the extracted adjacent DLL caused the Release process to exit with failure rather than loading the development DLL; the file was restored.
- Ordinary launch of the extracted portable application, without demo flags or the QA override, created an independent database and a responsive Yamantaka window. That review window was left open. The ordinary launch wrote runtime data to the extracted review copy; the distributable application folder retained its verified inventory.
- Workspace Clippy across all targets passed with warnings denied. `cargo fmt --all --check` passed. No standalone Rust tests were added.

The [status report](artifacts/windows-review-bundle.json) contains the pre-commit candidate hashes and check results. Native evidence was captured from an earlier assembly with the same shipped file inventory; its archive hash is retained separately from the repeated current-input archive hash. The runtime source changes were uncommitted when those candidates were assembled and are identified by the manifest's source hashes and dirty flag.

## Acceptance limits

Authenticode status is unverified. Windows 10 execution, physical DPI/monitor/Narrator acceptance, full product parity, prolonged offline traces, process-wide resource measurements and the required paired Flutter/Rust performance runs remain pending. Static import checks inspect PE imports on this host; they do not prove every dynamically loaded graphics-driver dependency on every supported machine. The local review candidate must not be described as the final replacement Release.
