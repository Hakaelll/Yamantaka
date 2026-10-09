# Cross-page text selection semantics

Date: 2026-10-09

## Implemented domain behavior

The selection model uses native character boundaries, independently of UTF-8 byte length and screen rotation. It orders anchor and extent by document page and character index, so reverse selection produces the same text as forward selection. Each page projection validates its endpoints against an immutable native snapshot before exposing borrowed text and normalized regions.

A materialized copy requires every page in the selected range, in exact order. Missing, duplicate or reordered snapshots are rejected rather than silently producing a truncated passage. Nonempty page fragments join with a newline, matching the legacy selection's page-separator behavior. Empty intermediate pages do not add extra separators. Invalid endpoints are rejected rather than clamped to an unrelated passage.

Geometry coverage is explicit: complete, partial or unavailable. Missing bounds on whitespace do not disqualify otherwise complete visible text. Missing bounds on non-whitespace characters remain visible as partial/unavailable coverage; selected text is preserved and no rectangle is invented. Annotation consumers must use this information to offer the required page-note fallback; that UI is still pending.

The copied text receives its own Text-memory reservation before allocation, separately from source snapshots. Borrowed page projections allocate no text or rectangle vectors. Copies can outlive their source snapshots and release the reservation when dropped. Budget failure leaves the original snapshot intact. This contract does not account for OS clipboard or opaque native/UI memory.

## Verification

Four domain tests cover reversed multi-page Unicode selection, empty intermediate pages, partial ranges, missing/duplicate/reordered pages, invalid endpoints, geometry coverage, empty selection, allocation-budget failure and independent copy lifetime. Test text includes accented characters, a supplementary scalar and Hebrew. This verifies scalar/UTF-8 boundaries; it does not prove shaping, grapheme-aware interaction or bidirectional pointer hit testing.

The existing real PDFium integration now verifies a reversed cross-page selection through all four intrinsic rotations and a native cropped-out character with unavailable geometry. Native results still preserve original-file SHA-256 and deterministic memory release. The empty second page is explicitly included in the selected range.

All 224 workspace tests pass. The final native text-geometry integration suite also passes after the coverage assertions were added. Formatting and warning-denied workspace/all-target Clippy pass. Local logs are `build/verification/text-selection-tests.log` and `build/verification/text-selection-clippy.log`.

## Remaining integration

This is the tested selection/copy domain behavior. Reader pointer and keyboard selection, caret/word hit testing, multi-page drag lifecycle, selected-passage display, clipboard execution, annotation create/edit/review/export and the page-note fallback remain pending. Callers must bind snapshots to the current document and request identity, as the existing text-panel coordinator does. No native UI capture, OS interaction, accessibility, physical DPI or full release acceptance is claimed for this change.
