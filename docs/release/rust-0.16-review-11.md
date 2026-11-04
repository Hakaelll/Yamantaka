# Yamantaka 0.16.0-alpha.1: Windows review 11

This portable development candidate packages the changes after published review
10. It is intended for user review; it does not establish final replacement
acceptance. The published review-10 executable and existing portable libraries
remain separate.

The packaged runtime is commit
`fd91aa32396e86f53c08f50f55b6d45d97a5150c`, with unchanged runtime sources during
packaging.

## Changes since review 10

- Automatic cover batches can unlock protected PDFs through the existing
  transient-password dialog. **Cancel batch** stops unfinished items while
  retaining completed replacements.
- Protected batch items retain their reviewed source and cover decision. A
  later manual choice is preserved rather than overwritten. Automatic reset
  still requires the existing batch confirmation.
- A protected batch waits if native document slots are busy, retains its full
  source hash and checks the saved choice again before reopening. Passwords are
  not retained across a later opening attempt.
- Temporary cover documents retain their close operation when control admission
  is full. A channel deadline separates retries; cancellation still completes
  this cleanup before the save pipeline can proceed.
- Native reply admission reports queue saturation separately from PDF failure.
  The existing ordinary/control reservation limits stay unchanged.
- Metadata enrichment keeps its current page and earlier title/author evidence
  when native admission is temporarily full. Text, reopening and source-check
  stages retry after a short pause. Cancellation returns a cancelled outcome
  rather than classifying the extraction as unavailable.

The [cover guide](../user-guide/cover-recovery.md) and
[metadata guide](../user-guide/book-metadata.md) describe the development changes.
The [protected batch notes](../architecture/rust-protected-cover-batch.md)
describe their source, ownership and cancellation boundaries.

## Use the package

Extract the complete ZIP into a writable folder and start
`yamantaka/yamantaka.exe`. Keep its companion files and portable marker together.
The package runs without Flutter, Dart or Rust installed.

Each portable folder has its own `rust-data` and `rust-cache`. A newly extracted
folder does not transfer an earlier library. Original PDF files remain references
and are not rewritten. Keep earlier libraries available while reviewing this
candidate.

Main database schema eleven, JSON backups version nine and default-session
continuity version eight remain unchanged. Executables older than review 10 do
not support the new session format.

## Package

The local directory is `build/release/rust-0.16.0-alpha.1-review-11`. It includes
the portable ZIP, file manifest, SHA-256 sums, instructions, pinned PDFium,
licenses and a CycloneDX inventory.

An unpublished GitHub draft uses `v0.16.0-alpha.1-review.11` and the same runtime
commit. It contains the ZIP, `SHA256SUMS.txt`, `package-manifest.json` and
`SBOM.cdx.json`. GitHub's returned digests match the prepared files. The
[draft record](../validation/artifacts/github-review-11-draft-2026-11-04.json)
captures that state; access requires repository permission. Published review 10
remains unchanged.

| Item | Value |
| --- | --- |
| Target | Windows 10/11 x64 |
| Archive | `yamantaka-0.16.0-alpha.1-windows-x64-portable.zip` |
| Archive size | 16,356,259 bytes |
| Archive SHA-256 | `4e179bbb164af11f68afccc5034ab8b882f003bcc32ae5bdd4e908757a73aa99` |
| Executable size | 30,322,688 bytes |
| Executable SHA-256 | `28b6e6cc345bb09ba0d69f31d00a4ee82e31cd555e3b0fa346d48117f43b549e` |
| Packaged files | 462 |
| SBOM components | 239, including Yamantaka |
| Application license | MIT |

## Validation scope

The changed native development source compiled successfully. The new behavior
has no new application tests, captures, pressure or cancellation runs. Existing
fixtures were adapted where interfaces changed; no new test cases were added.

The locked, offline optimized Windows build and distribution packaging completed
successfully. The package inventory, archive contents, hashes, pinned PDFium,
licenses and executable resources passed packaging checks. The ten Windows icon
sizes are included. The [package record](../validation/artifacts/windows-review-11-2026-11-04.json)
identifies the build-and-packaging-only scope. No application instances were
opened for these packaging checks.

The user's general functional acceptance applies to review 10. This candidate
adds the changes above and does not extend that earlier acceptance. Protected
password retry, mixed batches, source changes, capacity pressure, cancellation
and resource release need runtime acceptance. Complete parity, physical
accessibility and comparative performance remain open in the
[acceptance matrix](../architecture/rust-rewrite-parity.md).
