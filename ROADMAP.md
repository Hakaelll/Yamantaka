# Windows Rust rewrite roadmap

Yamantaka targets Windows 10/11 x64. The native product uses Rust, egui/eframe,
wgpu, AccessKit, PDFium, and SQLite. Mobile and web releases are outside this
rewrite. The retained Flutter sources are a compatibility reference and recovery
checkpoint, not an active product roadmap.

The [acceptance matrix](docs/architecture/rust-rewrite-parity.md) is the detailed
source of implementation status and executed evidence. A connected feature or a
successful build does not establish complete replacement readiness.

## Preserve data and compatibility

Historical schemas and backup formats have native adapters. Independent migration,
reviewed restore, rollback, full-content duplicate verification, and preservation
of manual decisions are connected. Continue historical export, interruption, and
fault acceptance without modifying the original Flutter installation or PDFs.

## Complete the native reading and study workflows

The library, 3D covers, reader, annotation tools, search, workspaces, comparison,
organization, and recovery flows are connected in varying scopes. Finish the
remaining behavior and interaction acceptance recorded in the matrix. Native
bidirectional composition is connected through pinned text patches. Complete wider
script, IME and accessibility acceptance.

Keep reading controls compact, preserve book-cover physics, and reuse the approved
visual system in [DESIGN.md](DESIGN.md). Refine actual rendered states, including
small windows, enlarged text, errors, and reduced motion.

## Verify resource use and performance

Complete resource ownership and accounting, idle behavior, and large-catalogue
acceptance. Compare Flutter and Rust using the same host, corpus, and sequences
before reporting startup, rendering, latency, CPU, or memory improvements. Retain
separate measurements for native PDF work, UI work, and GPU presentation.

## Complete physical Windows acceptance

Verify DPI 100/125/150/175/200%, monitor changes, Narrator, keyboard and pointer
interaction, fullscreen, window restoration, and the alternate graphics path.
Synthetic captures supplement these checks; they do not replace physical evidence.

## Deliver the replacement release

A local optimized Windows review bundle already includes manifests, hashes,
licenses, and a CycloneDX SBOM. See the
[bundle instructions](docs/release/RUST-WINDOWS-BUNDLE.txt). It remains a review
candidate while parity, performance, and physical acceptance are incomplete.

After those gates pass, produce the current final bundle, release notes, and
verification results. Preserve a recoverable Flutter checkpoint before retiring
its build scaffolding from the active product. Signing remains dependent on an
authorized certificate; no certificate or credential belongs in Git.

Historical Flutter planning remains in [NEXT_TODOS.md](NEXT_TODOS.md) for context.
Its checked items and old platform phases are not Rust completion evidence.
