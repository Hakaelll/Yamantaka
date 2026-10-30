# Saved reference annotation rendering checkpoint

The reference now draws saved geometry-backed annotations through the same AnnotationPaint renderer as the main reading. Highlight fills, underline and strikeout lines, color and page-rotation projection share one implementation. This checkpoint connects rendering and refresh, not a reference annotation editor.

Visible-page mark queries moved into the shared controller. Each pane keeps its own document, page slots, cancellation tokens and memory-governed payloads, with at most two pending requests. Reply routing checks the exact document, page and request token before acceptance. Leaving a page releases its slot; reference close and document replacement cancel pending requests and discard cached marks.

A settled main annotation write refreshes the corresponding document in both panes, including a write acknowledged after the main context has changed. Library-note batch writes refresh both active mark caches. Refresh also runs after a write error because a mutation may have committed before its follow-up read failed.

Refresh retains the previous visible payload while requesting its replacement. The shared paint path uses that retained payload until the new data arrives. Failed reads offer retry. An allocation failure drops the retained snapshot so that it cannot permanently consume the capacity needed for recovery. The reference exposes Reload marks without adding healthy-state chrome.

## Evidence

The final native binary compiled offline and formatting passed. Four focused native scenarios ran through tool/capture_rust_ui.ps1:

- reader-reference-marks uses distinct document identities. The main PDF starts with one yellow highlight, while the reference has a green highlight, underline and strikeout. A typed actor edit changes the main mark to blue. The reference retains its own three green marks. Navigating the reference to its unmarked second page and back releases and reloads its page cache without changing the main reading.
- reader-reference-marks-shared shows two views of the same PDF. The main edit refreshes both copies to blue while each retains its own visible-page ownership. The native sequence observes the previous payload during the refresh before accepting the replacement.
- reader-reference-marks-small-200 repeats shared-document refresh and navigation in Spanish, light theme, reduced motion, a requested 640×480 window, 200% scaling and 90-degree rotation.
- annotation-many-rotated retains the existing primary rendering case with 181 saved marks and rotated pages.

Before editing, the sequence cancels and replaces reference mark queries, then waits for a complete snapshot. It asserts document identity, page ownership, retained data during refresh, final counts and colors, an empty second page and recovery on return. The mutations and reads use the real application actor and SQLite; captures use the native PDFium/compositor path. Only synthetic PDFs and records were used. No Computer Use or standalone tests were added.

The [captures and manifest](../screenshots/rust-reference-marks/capture-manifest.json) record source and executable hashes. The reference annotation list/editor, direct creation from its selection, physical input/Narrator/DPI, fault and memory-pressure acceptance, broader annotation concurrency, performance acceptance and Release packaging remain pending. No broad workspace suite was run.
