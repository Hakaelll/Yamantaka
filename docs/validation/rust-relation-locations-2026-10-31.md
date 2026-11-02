# Connection page locations checkpoint

Date: 2026-10-31. Scope: optional page anchors when creating a document relation, and navigation to preserved page, bookmark, or attached-annotation anchors. This checkpoint does not establish whole-rewrite or release acceptance.

## Baseline and implementation

Flutter stores optional source/destination anchor kinds and IDs in v11 document_relations. Its connected creation dialog passes whole documents; the repository accepts page/bookmark/annotation anchors. The native manager previously preserved those fields but ignored them when opening a connection.

The native creation form now places optional source/destination page controls in a collapsed Link pages section. Displayed page numbers are one-based; stored page IDs remain zero-based decimal strings. Neither field is required. Selecting another destination clears its page choice. Writes validate page bounds against the catalogue inside the same immediate transaction as insertion and relation FTS refresh.

The v11 unique source/destination/type identity is retained. Repeating the same explicit locations reuses the row. A different explicit page proposal rejects the write without replacing the existing connection or clearing inherited anchors. Whole-document creation retains its previous idempotent behavior.

Relation pages now resolve locations in the same read transaction as their list and count. Outgoing entries resolve the target anchor; backlinks resolve the source anchor. Bookmark/annotation lookups require ownership by the other document, and annotations must remain attached. The resolved page must fit the current catalogue page count. Missing, unsupported, detached, incomplete, or invalid anchors produce an unavailable location while preserving their original fields. The UI offers normal document opening in that case.

Known destinations show Open page N and use the existing scoped document lookup, opening intent, asynchronous PDF runtime, and canonical reader navigation. This path does not write the PDF or introduce another reader. Number fields keep checkbox labels through AccessKit's label relation. Narrow layouts wrap the controls; they do not widen the application.

## Executed evidence

The retained compatibility test overrides_win_and_queries_are_bounded_and_escape_wildcards was extended through its existing relation helper. One passed, five filtered, 4.43 seconds. Added assertions cover persisted two-sided page anchors, outgoing/backlink resolution, duplicate identity, conflicting and out-of-range proposals, bookmark ownership, attached/detached annotations, unsupported/negative/out-of-range/incomplete anchors, whole-document fallback, and SQLite integrity. No standalone test was added. An initial run using the system temporary directory failed in the unrelated library-removal helper with Windows access denied; the same retained check passed with its temporary directory inside the workspace.

Offline native build, formatting, and binary Clippy with warnings denied passed. Three focused native scenarios passed state assertions and control bounds:

| Case | Observed result |
| --- | --- |
| relations-locations | Four persisted relation types with source page 1 and target page 2; optional controls expanded in Spanish light theme |
| relations-locations-small-200 | Same persisted locations; requested 640 × 480 logical viewport, 200% text and reduced motion; both page controls visible after scrolling |
| relations-location-open | Opens the target through the actual PDF runtime and reaches zero-based page 1; screenshot displays page 2 of 2 |

The normal form, final small form, and rendered destination were visually inspected. A first small capture showed only the screen heading, so the preview was corrected to scroll to the fields and assert their visibility. All final captures use isolated synthetic data. Source, Cargo.lock, executable and artifact hashes were verified before publication, and PNGs include origin metadata. See the [capture manifest](../screenshots/rust-relation-locations/capture-manifest.json).

## Remaining acceptance

The form creates page anchors, not new bookmark/annotation anchors, and does not edit existing links in place. Existing unsupported anchors remain preserved. Resolution uses the list snapshot; a source or anchor changed after loading requires refresh, and reader opening retains its normal missing-file and page-clamping behavior. Physical pointer/keyboard, Narrator, monitor DPI and whole-library performance acceptance remain pending. No Computer Use, personal library publication, broad suite, or release packaging was performed for this checkpoint.

## Editing follow-up

[Existing relation-location editing](rust-relation-location-edit-2026-11-02.md) now connects in-place updates from either endpoint, with explicit replacement and frozen-record checks. The earlier creation-only scope above retains its checkpoint meaning.
