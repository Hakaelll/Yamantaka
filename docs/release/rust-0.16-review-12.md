# Yamantaka 0.16.0-alpha.1: Windows review 12

This portable development candidate packages runtime commit `5946e3c`, including
the protected-cover and metadata changes from review 11. It is intended for user
review and does not establish final replacement acceptance. Published review 10
remains unchanged.

## Changes since review 11

- Settings → Storage now offers session-only folder authorization, pause/resume,
  revocation and unavailable-root retry. Notifications refresh saved catalogue
  records without importing or deleting documents. Safe mode disables this work.
- Pointer text selection can extend through page gaps and scroll at viewport
  edges in the main and reference readers. It retains the original anchor,
  finishes on button release and waits for visible-page text instead of skipping
  it. Direct scrolling has no easing or inertia.
- Find now loads visible-page highlights independently of result-list paging.
  Each pane uses up to two outstanding native requests with 80-match continuation,
  cancellation, queue retries and the existing text memory governor. The selected
  occurrence keeps its stronger fill and outline without duplicate painting.
  A short notice explains partial results when a native or resource limit is hit.

The guides explain [authorized folders](../user-guide/authorized-folders.md),
[text selection](../user-guide/text-selection.md) and
[document search](../user-guide/document-search.md). Review 11's
[cover recovery](../user-guide/cover-recovery.md) and
[metadata](../user-guide/book-metadata.md) changes are included.

## Use the package

Extract the complete ZIP into a writable folder and start
`yamantaka/yamantaka.exe`. Keep its companion files and portable marker together.
Flutter, Dart and Rust are not required to run the package.

Each portable folder has independent `rust-data` and `rust-cache` directories.
Extracting a new review does not transfer a previous library. Keep earlier
libraries available and use the reviewed backup or migration tools to transfer
data. Original PDFs remain references and are not rewritten.

Database schema eleven, JSON backups version nine and default-session continuity
version eight remain unchanged. Executables older than review 10 do not support
the current session format.

## Validation scope

The development source compiles successfully. No application tests, captures,
physical checks or comparative performance measurements were run for these
changes at the owner's request. Source inspection and compilation do not prove
pointer behavior, notifications, search under load or physical accessibility.

Windows can omit notifications for remote filesystems and does not report changes
to the watched root itself. Dense PDF pages remain subject to existing text,
match-count and memory limits. Full parity, migration, physical accessibility,
visual acceptance and paired performance gates remain open in the
[acceptance matrix](../architecture/rust-rewrite-parity.md).

## Package

The locked, offline optimized Windows build completed in 5 min 50 s. Distribution
inventory, archive bytes, hashes, pinned PDFium, licenses, SBOM and all ten icon
sizes passed packaging checks. These checks did not execute the application.
The [package record](../validation/artifacts/windows-review-12-2026-11-04.json)
records their scope.

The local directory is `build/release/rust-0.16.0-alpha.1-review-12`.

An unpublished GitHub draft uses `v0.16.0-alpha.1-review.12` and the same runtime
commit. Its ZIP, `SHA256SUMS.txt`, `package-manifest.json` and `SBOM.cdx.json` have
matching local and server-reported sizes and SHA-256 digests. The
[draft record](../validation/artifacts/github-review-12-draft-2026-11-04.json)
records that state. Draft access requires repository permission; this preparation
did not publish or replace a release.

| Item | Value |
| --- | --- |
| Target | Windows 10/11 x64 |
| Archive | `yamantaka-0.16.0-alpha.1-windows-x64-portable.zip` |
| Archive size | 16,382,976 bytes |
| Archive SHA-256 | `9fc857b6d397ca411fa82cfdfd944c115dfd64126638728014a761441325d3b4` |
| Executable size | 30,374,912 bytes |
| Executable SHA-256 | `e952e3705c52e9dedc0953d4ff1c5a18b5f32fe5b5886647557b60f03b6c9247` |
| Packaged files | 462 |
| SBOM components | 239, including Yamantaka |
| Application license | MIT |
