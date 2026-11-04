# Yamantaka

**A private PDF library for reading and study on Windows.**

Yamantaka combines an immersive library of three-dimensional book covers with a
focused native reader. Organize books, compare passages, annotate documents and
return to your saved reading context. Everything runs locally, without accounts,
telemetry or a cloud backend.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/rust-readme-2026-11-03/readme-library-dark.png" />
  <img src="docs/screenshots/rust-readme-2026-11-03/readme-library-light.png" alt="Yamantaka's native library with dimensional book covers and compact reading controls" />
</picture>

*Native application screenshots from a generated study collection, captured on
2026-11-03.*

## Download

[Download Yamantaka 0.16.0 for Windows x64](https://github.com/Hakaelll/Yamantaka/releases/download/v0.16.0/yamantaka-0.16.0-windows-x64-portable.zip)

Windows 10 or Windows 11, x64. Extract the complete ZIP into a writable folder
and start `yamantaka/yamantaka.exe`. Keep PDFium, licenses and companion files
together. Flutter, Dart and Rust are not required to run the application.

The [release page](https://github.com/Hakaelll/Yamantaka/releases/tag/v0.16.0)
includes SHA-256 checksums, a file manifest and a CycloneDX SBOM. The distribution
is unsigned; published hashes identify the prepared files.

## Read, organize and study

| Area | Capabilities |
| --- | --- |
| Library | PDF-derived and manual covers, dimensional books with pointer physics, grid/list views, combined filters, sorting, grouping and bulk actions |
| Reading | Document and Book modes, single/spread/continuous layouts, LTR/RTL reading, zoom, pan, rotation, margins and reading environments |
| Navigation | Contents, thumbnails, cancellable search, visible-page match highlights, history, bookmarks and A/B positions |
| Study | Unicode passage and word selection, copying, highlights, underlines, strikeouts, notes, review and Markdown/JSON export |
| Organization | Collections, tags, smart views, relations, backlinks, revisions and reviewed local automation with undo |
| Continuity | Reading progress and sessions, recent documents, workspaces, queue, focus and optional synchronized comparison |
| Local search | Opt-in FTS5 indexing, resumable indexing, filters, ranked previews and exclusion management |
| Windows | Native selectors and clipboard, drag and drop, single-instance activation, fullscreen, saved geometry, ES/EN and keyboard accessibility |
| Recovery | Historical database and backup compatibility, independent migration, safe mode, quarantine and reviewed restore/rollback |

Original PDFs remain unchanged. Annotations and metadata decisions live in the
library. Full-text indexing stores extracted text locally only after consent.
PDF passwords stay transient in memory. Scanned books can need manual metadata
or page notes when readable text or reliable geometry is unavailable.

Printing, OCR, encrypted backups and secondary reader windows are outside this
release's completed feature scope.

## First use and migration

1. Add PDFs with the native selector, drag and drop, command-line paths or folder import.
2. Open a book. Use **Navigator** for contents, pages, search and bookmarks.
3. Use **Reading options → Compare pages** to open a reference pane.
4. Use **Settings → Storage** for backups, migration, cache maintenance and session-only folder authorization.

Portable builds contain `.yamantaka-rust-portable` and keep `rust-data` and
`rust-cache` beside the executable. Other builds use the independent
`io.yamantaka.rust` application identity. Extracting a new portable folder does
not automatically move an existing library.

Close Flutter before reviewed migration. Yamantaka captures an independent copy
using SQLite's backup API and preserves the original installation. SQLite schema
v11, historical schemas v1–v11 and JSON backups v1–v9 remain supported; exports use
backup v9. Prefer the application's backup/import workflows to manually copying
an open database.

The default compositor uses DirectX 12 through wgpu. An initialization failure
attempts the alternate OpenGL compositor once; `--opengl` selects it explicitly.
See [graphics recovery](docs/user-guide/graphics-recovery.md).

## Build from source

Use Windows x64, the pinned Rust toolchain, MSVC build tools and a Windows SDK.
Initial dependency and PDFium setup can use the network; normal application use
has no network dependency.

```powershell
./tool/fetch_rust_pdfium.ps1
cargo build --locked -p yamantaka
./target/debug/yamantaka-rust.exe
```

To create an optimized portable release, use Python 3.11+ with Pillow and restore
Cargo dependencies first:

```powershell
python ./tool/fetch_rust_licenses.py
./tool/package_windows.ps1 -Portable -OutputDirectory build/release/yamantaka-0.16.0
```

Packaging uses locked, offline Cargo resolution, the static Microsoft C runtime
and remapped build paths. It generates the ZIP, file manifest, SHA-256 sums,
license notices and CycloneDX SBOM. It does not publish to GitHub automatically.
Choose a fresh output directory for each package.

## Architecture and validation

The six-crate workspace separates domain, application, storage, PDF execution,
Windows integration and UI. The interface uses egui/eframe, wgpu and AccessKit;
PDFium handles PDF rasterization on an owning serial worker. SQLite uses
rusqlite with FTS5. Local changes to upstream components are documented in
`third_party/`.

The owner accepted final validation on 2026-11-04 after testing review 13 without
reported faults. Finalization adds the Rust-only project cutover, relocated
Windows icon resources and release versioning. No new agent-run runtime suite,
physical checks or comparative measurements are claimed. Historical evidence
retains its original scope; no unreported numerical speedup is implied.

See [release notes](docs/release/v0.16.0.md), the
[acceptance and evidence matrix](docs/architecture/rust-rewrite-parity.md),
[testing guide](TESTING.md), [architecture](ARCHITECTURE.md),
[privacy policy](PRIVACY.md) and [contribution guide](CONTRIBUTING.md).

The Flutter source and tools remain recoverable from the
[archived Git checkpoint](docs/legacy/README.md). The active project and roadmap
support native Rust on Windows only.

## Guides

| Topic | Guide |
| --- | --- |
| Titles, authors and local extraction | [Book metadata](docs/user-guide/book-metadata.md) |
| Automatic and protected covers | [Cover recovery](docs/user-guide/cover-recovery.md) |
| Passage and word selection | [Text selection](docs/user-guide/text-selection.md) |
| Visible-page match highlighting | [Document search](docs/user-guide/document-search.md) |
| Reading positions and sessions | [Reading continuity](docs/user-guide/reading-continuity.md) |
| Undo and redo for annotations | [Annotation history](docs/user-guide/annotation-history.md) |
| Session-only folder permissions | [Authorized folders](docs/user-guide/authorized-folders.md) |
| Protected PDF indexing | [Protected search](docs/user-guide/protected-search.md) |
| Safety libraries and rollback | [Previous library](docs/user-guide/previous-library.md) |

## License

Yamantaka project code is licensed under [MIT](LICENSE). Bundled dependencies and
fonts retain their own licenses, supplied in the Windows distribution.
