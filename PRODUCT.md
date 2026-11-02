# Yamantaka

<!-- impeccable:product-schema 1 -->

## Platform

Windows 10/11 x64 native desktop. Windows is the only active rewrite platform.

## Stack

Rust Cargo workspace; egui/eframe, wgpu, AccessKit, PDFium through pdfium-render,
SQLite through rusqlite. No web frontend or Flutter in the final Rust product.

## Users

Personal-library readers studying documents and books, confirmed by the user.

## Product Purpose

Private local reading, study, annotations, organization and continuity. Success
requires verified functional parity, preserved data, correct PDF rendering and
a visibly exceptional, professional executed interface.

## Operating Context

Local PDFs, long reading sessions, mouse and keyboard, multiple monitors and
variable DPI. The Flutter installation remains a separate fallback during the
rewrite; migration captures a closed installation and never synchronizes it.

## Capabilities and Constraints

The supplied rewrite specification is the acceptance authority. Preserve schema
v11, import schemas v1–v11 and JSON backups v1–v9, export backup v9. Rust preferences
use versioned atomic JSON. Originals remain unchanged; passwords live in memory.
12 tabs, two hot documents, catalog pages of at most 80 entries. No accounts,
telemetry or network during normal use. Printing, secondary reader windows, OCR
and encrypted backups are proposals/contracts, not established complete features.

## Brand Commitments

Keep Yamantaka's name. The owner requested a corporate logo replacement on
2026-11-02; use the canonical geometric open-volume mark in `assets/branding/yamantaka-logo.svg`.
This replaces the earlier logo-geometry preservation requirement. The user asks
for a substantial visual replacement and an interface whose beauty and professional
finish are striking. Dark neutrals and functional violet; warm paper and ivory in
light mode; local Segoe UI Variable typography.

## Evidence on Hand

Flutter commit c773acf, connected lib/features implementation, committed tests,
docs/screenshots/windows-reader-editorial-v015.png. The recorded 330 tests and
performance regressions are historical evidence, not current measurements.

## Product Principles

- Reading and study lead the experience.
- Every persistent user decision survives migration.
- The original PDF is never an editing target.
- Explain errors and provide actionable recovery.
- Measure behavior and appearance before claiming completion.

## Accessibility & Inclusion

ES/EN, Unicode, actual bidi text shaping, keyboard alternatives, visible focus,
semantic custom controls, 200% text, 640×480 through ultrawide, reduced motion,
Narrator and physical DPI validation.
