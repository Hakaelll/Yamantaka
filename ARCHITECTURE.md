# Architecture

Yamantaka is a native Rust Windows application. Its six Cargo crates separate
business rules, orchestration, storage, PDF execution, platform services and UI.

| Crate | Responsibility |
| --- | --- |
| `rust/domain` | Typed documents, navigation, geometry, cancellation and memory budgets |
| `rust/application` | Actor commands, imports, search, background jobs and recovery |
| `rust/storage` | SQLite v11, historical migrations, backups and atomic preferences |
| `rust/pdf` | Serialized PDFium ownership, render jobs, text and source validation |
| `rust/windows` | Native selectors, clipboard, instance activation and filesystem integration |
| `rust/ui` | egui/eframe views, GPU composition, interaction and accessibility |

The UI emits commands; it does not execute SQL or call PDFium. One dedicated
worker owns native PDF resources and prioritizes visible rendering over optional
work. Consumers cancel independently, retained previews survive refinement, and
ownership governs deterministic release. The viewport virtualizes continuous
pages. PDF buffers and textures use explicit memory admission and device limits.

SQLite keeps the main schema at version eleven. Historical schemas one through
eleven and JSON backups one through nine retain compatibility adapters. Backup
exports use version nine. Native preferences and recreatable auxiliary state are
separate. Reviewed import captures a closed Flutter installation into an
independent store using SQLite's backup API; it never synchronizes two libraries.

PDF originals are immutable inputs. Annotations, metadata corrections, reading
context and organization live in the library. Passwords are transient; local
full-text indexing requires consent. Normal use has no account, telemetry or
cloud dependency.

See [PRODUCT.md](PRODUCT.md), [DESIGN.md](DESIGN.md), the
[parity record](docs/architecture/rust-rewrite-parity.md) and
[Flutter recovery instructions](docs/legacy/README.md).
