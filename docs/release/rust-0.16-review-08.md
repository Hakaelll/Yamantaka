# Yamantaka 0.16.0-alpha.1: native Windows review candidate

This candidate packages the Rust rewrite for local review. Full replacement-release acceptance remains open. Keep the Flutter installation and its original library as a separate recovery copy.

## Changes since the earlier bundle

- Opening a reading prepares its actual opening viewport, including spread pages, before replacing the previous reading. Restored comparisons prepare both panes and apply saved page, zoom and position following before presenting the split.
- The application uses the black abstract open-folio mark in its branding and Windows icon resources.
- Global-search exclusions have a complete browser with title/author filtering and pages of 80 records. Including a document does not enable indexing or start it without consent.
- The pinned text layer handles Arabic, Hebrew and mixed-direction text with logical cursor and copy mapping, visual selection ranges and local font fallback. The dependency inventory identifies the local egui and epaint overrides.
- Safe mode offers explicit review and preservation of a damaged startup record before creating a replacement. The current run stays in safe mode until the application is reopened.
- Recovery of an interrupted library operation offers reviewed current/previous-library choices. The operation record and complete libraries are retained. Review includes complete valid SQLite WAL data through independent temporary copies.

The [acceptance matrix](../architecture/rust-rewrite-parity.md) records the scope and evidence for each capability. These changes do not establish complete feature parity or performance acceptance.

## Start the candidate

1. Extract the complete ZIP into a writable directory.
2. Start `yamantaka/yamantaka.exe`. Keep `pdfium.dll` beside it.
3. Review the independent Rust library before importing existing data. Close Flutter before capturing its library through Yamantaka's import workflow.

Flutter, Dart and a Rust installation are not required to run the bundle. The target platform is Windows 10/11 x64 with working graphics drivers. Physical Windows 10 execution and broader driver coverage remain acceptance tasks.

A portable bundle contains `.yamantaka-rust-portable`. It keeps its independent `rust-data` and `rust-cache` beside the executable. The package does not contain a personal library or PDF corpus. Reading and organization remain local; originals are not rewritten.

## Package contents and verification

The archive contains the native executable, pinned PDFium, application MIT terms, Cargo.lock, user instructions, third-party notices, license texts and a CycloneDX 1.5 inventory. The outer package includes a per-file manifest, SHA-256 sums and packaging reports.

See the [updated bundle checkpoint](../validation/rust-windows-review-bundle-2026-11-03.md) for the exact source revision, hashes, executed checks and limitations. The [bundle instructions](RUST-WINDOWS-BUNDLE.txt) explain the directory layout and independent storage.

## Remaining acceptance

Final replacement-release readiness requires complete functional and migration acceptance, paired Flutter/Rust performance measurements, physical DPI/monitor/Narrator checks and the remaining recovery cases. Multiple pending library records, missing library folders and damaged selection metadata still require separate recovery. Incomplete or stale WAL tails are preserved and rejected by the conservative whole-store review.

The text patch still needs broader script/font, IME, complex-selection and physical accessibility validation. Printing, secondary reader windows, OCR and encrypted backups are not presented as completed features.

No distribution-signing certification is included. Package verification and native smoke checks have a narrower scope than product acceptance.
