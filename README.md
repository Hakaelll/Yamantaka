<p align="center">
  <img src="assets/branding/yamantaka-logo-purple.png" alt="Yamantaka" width="80" />
</p>

# Yamantaka

**A private PDF library for reading and study on Windows.**

Yamantaka brings your documents into a personal library, keeps your reading context close, and stores your work locally. The native Rust rewrite pairs a gallery of physical 3D book covers with a focused PDF reader, warm paper environments, and a consistent desktop interface.

No account is required. Documents and library data stay on your computer. The product is designed to operate without telemetry or network access during normal use.

> **Development status:** the Rust rewrite is in progress. It is not yet a replacement release. Flutter 0.15.0+16 remains the recoverable reference implementation. The [acceptance matrix](docs/architecture/rust-rewrite-parity.md) distinguishes verified behavior, partial implementations, and outstanding work.

<p align="center">
  <img src="docs/screenshots/rust-stage2/library-dark.png" alt="Native Yamantaka library with procedural 3D book covers and a compact continue-reading strip" width="1100" />
</p>

*Native Windows capture of an isolated synthetic study collection. The capture demonstrates the current gallery; it is not a photograph of a personal library or proof of complete feature parity.*

## The experience

- **An immersive library.** Large book covers with connected boards, spines, page blocks, six procedural cover families, and a damped response to the pointer. Reduced motion retains a static volume.
- **A practical catalogue.** Cover and list views, title and author search, favorites, reading-state and missing-file filters, stable sorting, and pages of up to 80 documents. A document inspector preserves the width of the gallery.
- **A reader built around the document.** Native PDFium rendering, GPU presentation, page navigation, fit-to-page and fit-to-width, zoom, rotation, margins, page text, canonical history, and positions A/B. Single, facing and virtualized continuous layouts are connected through the canonical navigation model. Six reading environments are stored per document.
- **A coherent desktop interface.** Dark graphite and warm ivory themes, functional violet accents, local Segoe UI Variable typography, custom controls, visible keyboard focus, Spanish and English surfaces, reduced motion, and text enlargement.
- **Independent storage.** A separate Rust library, versioned atomic preferences, and compatibility adapters for historical data. The Flutter installation remains a fallback during the transition.

These describe connected portions of the current implementation. Automatic and manual cover images, complete reader interaction acceptance, annotations, organization, continuity, search management, and recovery flows still have outstanding acceptance work. See the matrix before relying on a capability.

## Platform

| Item | Target |
| --- | --- |
| Operating system | Windows 10 / 11 |
| Architecture | x64 |
| Presentation | Native Rust; egui/eframe, wgpu and AccessKit |
| PDF engine | PDFium 7881 through pdfium-render |
| Storage | SQLite with FTS5 through rusqlite |
| Interface languages | Spanish and English |
| Active product scope | Local PDF reading and study on Windows |

Android, iOS and web directories belong to the historical Flutter project. They are not targets of the active rewrite. Printing, OCR, encrypted backups and secondary reader windows are proposals or contracts, not completed Rust features.

## Run the development build

The following commands are for **development**, not for installing a finished release.

You need Windows x64, the Microsoft C++ build tools required by Rust's MSVC target, and Rust installed through rustup. The repository pins Rust 1.97.1 in `rust-toolchain.toml` and dependency versions in `Cargo.lock`.

```powershell
# From the repository root
cargo build -p yamantaka --locked
./tool/fetch_rust_pdfium.ps1
./target/debug/yamantaka-rust.exe
```

The PDFium fetch script downloads and verifies the pinned native dependency. Dependency setup may use the network; that is separate from normal application use.

To open a document from the command line:

```powershell
./target/debug/yamantaka-rust.exe "C:\Books\Example.pdf"
```

The default compositor uses DirectX 12. A separately checked OpenGL route is available:

```powershell
./target/debug/yamantaka-rust.exe --opengl
```

A finished Release bundle must include `pdfium.dll` beside the executable. Do not copy only the executable out of a bundle. The final bundle, hashes, SBOM, notices and installation instructions will be added after the parity and validation gates pass.

### Isolated visual collection

Generate the synthetic PDF corpus with Python and the `reportlab`, `Pillow` and `pypdf` packages, then open the demo:

```powershell
python ./tool/create_rust_pdf_fixtures.py
./target/debug/yamantaka-rust.exe --demo
./target/debug/yamantaka-rust.exe --demo --light
./target/debug/yamantaka-rust.exe --demo --reader
```

The demo uses an in-memory database and synthetic documents. It does not open the personal library. It currently depends on the repository's generated fixtures and is a development tool.

## Data and migration

The rewrite must preserve identifiers, relationships, progress, annotations, manual corrections, cover decisions, workspaces, sessions and automation journals. Originals are never PDF editing targets.

| Data | Compatibility contract |
| --- | --- |
| Main database | Preserve SQLite schema v11; migrate historical schemas v1–v11 |
| JSON backups | Import versions v1–v9; retain version 9 export |
| Flutter preferences | Capture separately from SQLite, including unknown keys |
| Rust preferences | Versioned JSON, written atomically |
| Initial import | Independent, explicit, idempotent and fail-closed |
| SQLite capture | WAL-coherent backup; never just copy an open main database file |
| Exact duplicate detection | Full SHA-256 verification; inherited partial fingerprints alone are insufficient |

Compatibility tests cover populated historical schemas and backup contracts. The complete migration preview, confirmation and recovery interface remains pending. **The current executable does not provide a complete one-click migration flow.**

Rust uses independent installed and portable data locations. It does not synchronize changes back to Flutter. See [the acceptance matrix](docs/architecture/rust-rewrite-parity.md) for the precise implemented boundaries and [historical Flutter documentation](docs/architecture/flutter-readme-v015.md) for the incumbent workflows.

## Architecture

The workspace keeps presentation, persistence and native rendering separate:

| Crate | Responsibility |
| --- | --- |
| `yamantaka-domain` | Typed contracts, persisted semantics, canonical navigation, reader geometry and memory accounting |
| `yamantaka-application` | Commands, events, cancellation, actor runtime and incremental indexing coordination |
| `yamantaka-storage` | SQLite repositories, historical migrations, backups, preferences and FTS5 |
| `yamantaka-pdf` | Serialized PDFium worker, prioritized tickets, bounded caches and native rendering |
| `yamantaka-windows` | Paths, native activation, clipboard, Explorer and Windows integration |
| `yamantaka` | Desktop composition, custom controls, physical covers and GPU page presentation |

UI components dispatch commands and receive state. They do not execute SQL or call PDFium directly. Native PDF handles remain on one worker; GPU resources have explicit lifetimes. Initial global category limits are 72 MiB for pages, 24 MiB for tiles and 8 MiB for text. Complete live-resource coverage and device-loss recovery remain acceptance tasks.

## Validation

Run the Rust checks from the repository root:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Generate the PDF fixtures and fetch PDFium first to run the native PDF suite. Tests exercise text, vectors, scanned pages, passwords, damage, intrinsic and requested rotation, mixed page sizes, tiles and documents of 5,000 pages. Storage tests cover historical schemas, populated relationships, backup formats, cancellation and rollback. Native Windows tests cover second-process activation and Unicode paths.

Current evidence is documented separately so dates and scopes remain explicit:

- [Acceptance matrix](docs/architecture/rust-rewrite-parity.md): the source of truth for completion.
- [Flutter baseline](docs/validation/flutter-baseline-2026-10-08.md): 330 tests and analysis executed on the reference implementation.
- [Native gallery and settings validation](docs/validation/rust-visual-stage2.md): captures, control bounds and the bounded visual review.
- [Search-controller validation](docs/validation/rust-search-controller-2026-10-09.md): cancellation, durable resume, exclusions and nonblocking native boundaries.
- [Search actor validation](docs/validation/rust-search-runtime-2026-10-09.md): consent, native slot handoff, responsive catalogue queries and shutdown. The search settings and results surface are still pending.
- [Native page-resource validation](docs/validation/rust-native-page-resources.md): GPU readback, allocation, retirement and scoped rendering evidence.

Catalogue measurements cover synthetic libraries of 5,000, 25,000 and 100,000 documents. They do not establish a complete performance advantage over Flutter. Paired startup, UI/GPU, PDF, CPU, memory and closure measurements are still required.

Physical DPI certification at 100/125/150/175/200%, Narrator, multiple monitors, the complete offline application smoke and all critical-flow captures remain pending. Automated tests and synthetic screenshots do not replace those checks.

Before publishing a development checkpoint, review the staged files:

```powershell
python ./tool/check_public_checkpoint.py --staged
```

This guard checks exact Git blobs and synthetic capture metadata for recognized private paths, contacts, credentials and unexpected data files. It complements human review; it does not establish complete privacy certification.

## Project documentation

- [Product brief](PRODUCT.md)
- [Design system](DESIGN.md)
- [Visual contract](docs/architecture/rust-visual-contract.md)
- [Memory plan and implemented boundaries](docs/architecture/rust-live-memory-plan.md)
- [Contribution guide](CONTRIBUTING.md)

Third-party software and font notices remain with their respective components. The final Release bundle must include a complete license inventory and SBOM. No finished Rust release or distribution-signing status is claimed by this README.
