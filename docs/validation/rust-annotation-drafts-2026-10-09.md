# Selection annotation draft checkpoint

Selection can now prepare an independently owned annotation draft in the UI/FFI-independent domain. This is the preparation contract for the editor and storage integration, which remain pending. It does not save an annotation or modify a PDF.

The contract defines Flutter's six annotation kinds, five colors, attachment status and editorial review states. Outbound field names and rectangle keys follow the existing camelCase and normalized-page-v1 conventions. Drafts carry document identity, fingerprint, page, selected text, surrounding context, user rotation, note and geometry. Persisted IDs, timestamps, tags, historical decoding and repository operations still need connecting.

Reliable selections produce the requested kind with normalized native glyph bounds. Annotation regions omit whitespace while copied passage text keeps internal whitespace. Missing geometry or rectangles below Flutter's meaningful-area threshold produce an explicit PageNoteFallback proposal with the requested kind and coverage. The proposed page comment retains text and has no invented rectangles. The future editor must present and confirm that fallback before saving.

Notes use Flutter's 4,000 UTF-16-unit limit. Surrounding context keeps up to 80 UTF-16 units on each side and avoids splitting supplementary Unicode scalars. New draft strings follow the legacy creation behavior of trimming surrounding whitespace. Identity, text, note, context and rectangle allocations are reserved against the shared Text budget before allocation, with retained capacities charged after construction. Drafts can outlive their source text snapshots and release their reservation on drop. A denied reservation leaves the source selection intact.

## Verification

Three domain draft tests cover Unicode text, rotation, independent memory ownership, geometry fallback, note boundaries and admission failure. A fourth context regression checks both sides of a passage surrounded by supplementary characters at the 80-unit boundary.

The real PDFium text integration now prepares all six kinds from the same native Unicode selection through four intrinsic rotations. It compares resulting regions with the original native geometry, checks context and fingerprint provenance, and preserves the existing source-file SHA-256 and memory-release assertions. A cropped-out native character proposes a page note with unavailable geometry, retaining its text without rectangles.

Workspace tests, formatting and warning-denied workspace/all-target Clippy pass. The workspace test log is local at `build/verification/annotation-drafts-tests.log`. UI presentation did not change in this checkpoint, so no new visual or OS interaction acceptance is claimed.

## Remaining integration

The editor, explicit fallback confirmation, actor/repository persistence, annotation painting, mutation history, tags, review filters, global review and Markdown/JSON export remain pending. Historical migration preserves existing raw rows, but this draft contract does not prove editable historical annotation parity. Clipboard, physical DPI, Narrator, performance and final release acceptance remain open.
