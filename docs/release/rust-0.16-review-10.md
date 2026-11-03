# Yamantaka 0.16.0-alpha.1: Windows review 10

This portable review candidate includes the native Rust changes through commit
`1659213eebd1b590ee4db457b8cd2b88e7c2a302`. It is a development build for user review, not the final replacement
Release. The application remains local and independent of the Flutter store.

## Changes since review 09

- Startup recovery reviews multiple pending operation records, retains their
  original bytes and keeps an interruption guard until archival finishes. A
  reviewed previous library can also be restored when the active folder is
  absent; this case records explicitly that no current folder exists to save.
- Saved comparisons retain the reference's Document/Book mode, page layout,
  cover pairing, initial side and control-hiding choice. Imported Flutter split
  readings use the same preference adapter.
- Main readings retain a normalized point within the PDF page, including
  horizontal panning, across tab changes, duplication, reopening and workspace
  restoration. Opening preparation and final presentation use the same point.
- Reduced motion applies to both reading panes, disables drag inertia and
  additional discrete-wheel smoothing, and completes pending scroll targets
  when animation is disabled.
- Front-matter analyzer version four orders text lines by page geometry. A
  supported cover title can replace embedded metadata that merely repeats the
  filename. Guarded reanalysis uses the same rule and retains manual corrections.
- A visible-page or refinement source-change failure retains available base
  images, stops further page work in that pane and offers localized close/reopen
  guidance through the existing note-protected close commands.
- Refusing to close a reference because of an unfinished note retains its
  comparison recovery configuration. The inherited split adapter also preserves
  an available within-page reference point.

See the guides for [book metadata](../user-guide/book-metadata.md),
[reading continuity](../user-guide/reading-continuity.md),
[reader profiles](../user-guide/reader-profiles.md) and
[reader responsiveness](../user-guide/reader-responsiveness.md).

## Use the package

Extract the complete ZIP into a writable folder and run
`yamantaka/yamantaka.exe`. Keep `pdfium.dll`, the portable marker and the supplied
files together. The package does not require Flutter, Dart or Rust at runtime.
Its `rust-data` and `rust-cache` folders belong to that portable installation;
extracting a separate review build does not transfer an earlier library.

New default-session snapshots can use continuity version eight. Older review
executables do not support that version. The main database stays at schema
eleven and JSON backups at version nine. Older supported session snapshots
remain readable; original PDFs are not rewritten.

The local output directory is
`build/release/rust-0.16.0-alpha.1-review-10`. It includes the ZIP, a per-file
manifest, SHA-256 sums, pinned PDFium, MIT and third-party licenses, Cargo.lock,
instructions and a CycloneDX inventory. This checkpoint does not publish a
GitHub Release.

| Item | Value |
| --- | --- |
| Target | Windows 10/11 x64 |
| Archive | `yamantaka-0.16.0-alpha.1-windows-x64-portable.zip` |
| Archive size | 16,352,931 bytes |
| Archive SHA-256 | `99da170913e2514c3d82c954e5bdd36a9ff59907624be4b4b389067b9e2a3d96` |
| Executable size | 30,305,280 bytes |
| Executable SHA-256 | `f024be3129319b8b8f6925cb150cb2dc95a02a0e586059fbedd7daaa4c61f85e` |
| Packaged files | 462 |
| SBOM components | 239, including Yamantaka |
| Application license | MIT |
| Runtime sources modified during packaging | No |

## Validation scope

This checkpoint generates the optimized executable and its distribution files.
The packaging tool's integrity, resource and license checks concern those files.
They do not establish runtime acceptance.

The locked, offline Release build completed successfully. Package inventory,
archive contents and hashes, pinned PDFium and executable resources passed the
packaging checks. The ten icon sizes are included. The
[package record](../validation/artifacts/windows-review-10-2026-11-03.json)
retains the reported facts and explicitly identifies the build-only acceptance
scope. No additional application instances were opened.

No new application tests, native captures, physical checks or comparative
performance runs are included. Earlier reports retain their original source
scope and do not certify the changes listed above. Functional and migration
acceptance, DPI, Narrator, multiple monitors, Windows 10 execution and paired
Flutter/Rust performance remain open in the
[acceptance matrix](../architecture/rust-rewrite-parity.md).
