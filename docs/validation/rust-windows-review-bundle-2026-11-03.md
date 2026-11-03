# Updated native Windows Release review bundle

The local `review-08` candidate builds the committed runtime at `9822b6126a27351497bc6660e09139e6adaa7168`. Runtime sources match that commit. This checkpoint adds package evidence and English [candidate notes](../release/rust-0.16-review-08.md); it does not approve the final replacement Release.

## Package evidence

The existing packager completed a locked, offline, optimized `x86_64-pc-windows-msvc` build with static C runtime linkage and build-path remapping. The package contains 462 files and 239 SBOM components, including the application. The executable is 29,831,680 bytes and the portable ZIP is 16,186,420 bytes. Its SHA-256 is `6a796dc6de242cfeaeac7a44f467f0b8247ae1cc4093cee3e1da68d6ed5a7167`.

The manifest inventory, ZIP contents, sizes/hashes, pinned PDFium, targeted build-profile privacy scan and current-host PE import checks pass. All ten Windows icon frames and the application version resource pass. A fresh verify-only invocation checks the original distributable after the execution checks, which use a separate extracted copy.

The manifest's source inventory matches the current packaging inputs, including the local egui and epaint sources, manifests, licenses and patch descriptions. Their SBOM entries identify the local overrides; unicode-bidi 0.3.18 and both upstream license sets are present. The SBOM validates against the official CycloneDX 1.5 schema resources after bundling their cached external references locally. Direct `Test-Json` schema-file loading could not resolve those external references in this environment. The artifact records the three original schema hashes and the offline-reference method.

Authenticode inspection reports `NotSigned`. No signing certification is claimed.

## Executed application checks

All six checks launch the executable extracted from the verified ZIP, wait for its process to finish and check its exit code:

- The native wgpu reader installs a real PDF page image and settles managed page ownership.
- The native OpenGL reader does the same through `egui-glow`.
- The gallery prepares ten real PDF-derived covers with no pending or deferred uploads.
- The text-layout probe passes mixed-direction layout, composite marks, logical editor input and copy assertions.
- The startup-record probe passes preservation, stale/cancelled request rejection, valid-marker protection and subsequent normal startup assertions.
- The interrupted-operation window reviews a real WAL-only record, confirms the current library, archives the exact pending record and preserves the complete library inventories.

The actual reader and gallery PNGs were visually reviewed. These checks exercise native rendering and application paths; they do not measure frame-time percentiles or physical input. The Release QA override is supplied only for explicit demo captures. No corpus or personal library is shipped in the package.

A separate normal launch uses the extracted portable executable with a synthetic PDF argument, no demo flags and no QA override. It creates an independent schema-v11 database, imports that argument and opens a responsive Yamantaka window, which remains open for review. Two sampled Windows TCP/UDP-table observations during a requested ten-second interval contain no records owned by that process. This sampling does not establish packet-level absence, offline execution or all-use network behavior.

All 42 generated source PDFs retain their SHA-256. The distributable folder remains separate from the running portable store.

The [result artifact](artifacts/windows-review-bundle-2026-11-03.json) records package hashes, source provenance, native results, schema checks and the sampled normal startup. The bundle remains local at `build/release/rust-0.16.0-alpha.1-review-08`; native evidence remains under `build/verification/native-release-bundle-2026-11-03`.

## Remaining acceptance

Physical Windows 10 execution, DPI/monitor/Narrator checks, complete functional and migration parity, prolonged offline/resource traces, paired Flutter/Rust performance runs and the remaining recovery cases stay open. This unit changes no Rust source and adds no standalone tests. The [acceptance matrix](../architecture/rust-rewrite-parity.md) continues to govern replacement-release readiness.
