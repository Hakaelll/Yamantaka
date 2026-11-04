# Yamantaka 0.16.0-alpha.1: Windows review 13

This portable development candidate packages runtime commit `4243c38`, including
all changes from review 12. It is for user review; final replacement acceptance
remains open. Published review 10 is unchanged.

## Changes since review 12

- Pointer selection now uses the keyboard's Unicode grapheme boundaries.
  Selection endpoints do not split combining accents or joined emoji sequences,
  including when extending a passage through viewport-edge scrolling.
- Double-click selects the Unicode word-boundary segment under a glyph in both
  reading panes. Blank margins do not select distant text. The selection control
  explains the action in Spanish and English, including its accessible label.

Word segmentation is not a dictionary-based language recognizer. Scanned pages
without reliable text geometry still need page notes. See
[text selection](../user-guide/text-selection.md).

The candidate also includes review 12's session-only authorized folder
notifications, pointer edge scrolling and visible-page PDF search highlights,
plus review 11's protected-cover batch and metadata admission fixes. See the
[review 12 notes](rust-0.16-review-12.md) for those changes and their limits.

## Use the package

Extract the complete ZIP into a writable folder and start
`yamantaka/yamantaka.exe`. Keep the executable, PDFium, companion files and
portable marker together. Flutter, Dart and Rust are not required to run it.

Each portable folder has independent `rust-data` and `rust-cache` directories.
Extracting a new review does not transfer an earlier library. Keep the previous
library and use the reviewed backup or migration tools to transfer data.
Original PDFs remain references and are not rewritten.

Database schema eleven, JSON backups version nine and default-session continuity
version eight are unchanged. Executables older than review 10 cannot read the
current session format.

## Validation scope

The development source compiles successfully. No application tests, captures,
physical input checks or comparative performance measurements were run for these
changes at the owner's request. Compilation does not establish interaction,
Unicode, clipboard or accessibility acceptance.

The remaining parity, migration, physical accessibility, visual and paired
performance gates are recorded in the
[acceptance matrix](../architecture/rust-rewrite-parity.md). This package does
not authorize removal of the Flutter fallback or a stable replacement release.

## Package

The locked, offline optimized Windows build completed in 4 min 34 s. Distribution
inventory, archive bytes, hashes, pinned PDFium, licenses, SBOM and ten executable
icon sizes pass packaging checks. These checks did not run the application.
The [package record](../validation/artifacts/windows-review-13-2026-11-04.json)
records that scope.

The local directory is `build/release/rust-0.16.0-alpha.1-review-13`.

An unpublished GitHub draft uses `v0.16.0-alpha.1-review.13` and the same runtime
commit. The ZIP, `SHA256SUMS.txt`, `package-manifest.json` and `SBOM.cdx.json` have
matching local and server-reported sizes and SHA-256 digests. See the
[draft record](../validation/artifacts/github-review-13-draft-2026-11-04.json).
Draft access requires repository permission. This preparation did not publish
or replace a release.

| Item | Value |
| --- | --- |
| Target | Windows 10/11 x64 |
| Archive | `yamantaka-0.16.0-alpha.1-windows-x64-portable.zip` |
| Archive size | 16,383,130 bytes |
| Archive SHA-256 | `cf42edf924917748ff3e241dc15454e14c1669fbf830c3646c7abd8f04b14ae1` |
| Executable size | 30,376,960 bytes |
| Executable SHA-256 | `6822d590db0d2f3ed7690211a3dbe8a521f7b64f910b4555cb78b92ff09b62b6` |
| Packaged files | 462 |
| SBOM components | 239, including Yamantaka |
| Application license | MIT |
