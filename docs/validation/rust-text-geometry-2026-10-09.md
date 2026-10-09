# Native text geometry for selection

Date: 2026-10-09

## Implemented contract

`AsyncPdfEngine::try_text_geometry` returns a nonblocking ticket containing an immutable, memory-governed page result. PDFium characters remain in native order. Every character records its UTF-8 byte range and an optional normalized page rectangle. Selection uses half-open character indices, so accented characters, supplementary Unicode scalars and Hebrew do not split UTF-8 sequences. Missing geometry does not remove text or invent a rectangle.

The worker owns every PDFium handle, admits this request alongside page links/search, checks cancellation and document epoch before extraction, between characters and before publishing, and cancels queued work during shutdown. The page result can outlive the native document; its Text lease remains charged until the last Arc is released. No PDF bytes are edited.

Extraction uses the existing page coordinate space, including intrinsic rotation and crop offsets. Invalid, zero-area or wholly cropped character bounds become absent geometry. Undefined native Unicode scalars or NUL become the Unicode replacement character. User rotation remains a presentation transform. No reading-order reconstruction or OCR is implied.

## Work and memory limits

The native character count is checked before creating character vectors. The work limit is 128,000 characters per page. A 4 KiB initial Text reservation precedes inspection. The reservation is then resized for the full character result, the SDK's temporary i32 character-index vector, worst-case four-byte UTF-8 text and fixed overhead, before those buffers are allocated. Temporary indices are dropped before the lease shrinks to retained capacities and result/Arc metadata.

This accounts for application-owned buffers. Opaque PDFium document/text-page allocations remain part of the broader native-memory acceptance work; this test does not measure the process's full working set or prove the overall renderer budget.

## Verified behavior

Two domain tests verify multibyte selection, empty and out-of-range selections, missing geometry, invalid boundaries, uncovered text and final-Arc lease release.

Four real PDFium tests cover:

- WinAnsi accented text mapped to Unicode, exact selection boundaries, normalized geometry through crop and all four intrinsic rotations, empty pages, retained results after document close and unchanged original-file SHA-256.
- Cancelled requests, invalid pages, budget exhaustion, zero leaked Text bytes and successful subsequent extraction.
- More than 128,000 native characters rejected before result-vector allocation, followed by a successful empty-page request. The fixture uses five short text operators because a single oversized PDF literal is truncated by the native parser.
- Text outside the visible crop remains selectable as text while its rectangle is absent.

All 214 workspace tests pass. `cargo fmt --all --check` and warning-denied workspace/all-target Clippy pass. Local logs are `build/verification/text-geometry-workspace-tests.log` and `build/verification/text-geometry-clippy.log`.

## Remaining integration and acceptance

This is the tested native extraction layer. Application actor events, reader pointer/keyboard selection, multi-page selection, clipboard commands, selection presentation, annotation persistence/editing/review/export and the page-note fallback still need connecting and verifying. UI parity remains incomplete. Non-Latin embedded-font native fixtures, protected-document selection interaction, mid-extraction cancellation stress, OS accessibility, physical DPI and performance acceptance are also pending. No release-completion claim is made.

Subsequent actor and Text-tab integration is recorded in [Connected native page-text panel](rust-text-panel-2026-10-09.md). The acceptance list above describes this extraction checkpoint.
