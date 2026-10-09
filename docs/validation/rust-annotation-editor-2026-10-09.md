# Connected reader annotation editor

The reader now opens a Notes panel from a selected passage. Its editor retains the source text snapshot independently of later selection changes. Users can choose one of the six persisted annotation kinds and five colors, enter a note and save through the application actor. Existing notes can be edited, assigned an editorial review state, removed or used to navigate to their page.

The editor checks the 4,000 UTF-16-unit limit before enabling Save. Incomplete or insignificant glyph geometry offers an explicit page-note checkbox. Saving requires that confirmation, retains the selected text and creates no invented rectangles. Existing records without rectangles cannot be changed into a geometric kind through the editor.

Reads and writes show pending state. Failed saves retain the editor for retry. Results must match both the pending document and request token. Changing documents cancels pending reads and releases their editor/page snapshots. Accepted writes keep their independent lifetime; a failure for a departed document appears in the application error area. Successful current-document writes replace the governed page snapshot and close the editor.

The document list pages through 80 records using the existing repository contract. Loaded attached records paint over the PDF through the same normalized-page and user-rotation transform as selection and search. Highlight and note/marker regions use translucent fill; underline and strikeout use transformed line segments. Page comments have no geometric marks. This checkpoint paints the currently loaded annotation page, so complete coverage beyond its 80 records is still pending.

## Presentation evidence

The panel follows the existing graphite/violet and warm-paper reader styles. A passage preview separates source text from the editable note, and the saved state returns to the note list. The editor uses the existing resizable navigator. Compact windows use its full-width scrollable drawer. Selection-toolbar labels shorten in compact mode, and the toolbar recedes while the editor owns the task.

Four final native wgpu captures are local in `build/rust-visual/annotations-native`:

- `annotation-editor-dark` shows the actual selected PDF passage and editor.
- `annotation-editor-light-es` shows the light Spanish interface.
- `annotation-editor-small-200` shows the top of the scrollable drawer at 640 x 480 logical window size and 200% text. This is presentation evidence, not a completed keyboard/scroll accessibility test.
- `annotation-saved-rotated` saves through SQLite and the actor, clears the selection, closes the editor and paints the stored mark over a page rotated by 90 degrees. Its captured state confirms one resident/persisted result, no pending command and no open editor.

Final manifest SHA-256 is `9D789F57FEE8D6889B879B8980173B6A27AD450D0530A4A45CA0D799E55FF2CE`. Executable, Cargo.lock, source, PNG and control-state hashes match. Control coordinates are finite. All documents and notes in these captures are synthetic. Physical monitor DPI, Narrator and multi-monitor acceptance remain pending.

The first compact capture timed out after the added selection action consumed another toolbar row. Compact labels resolved that case. Inspection then found that the synthetic save scenario reopened its editor after completion; limiting preparation to the initial preview states resolved it. The final capture asserts that the saved editor stays closed. No broader performance claim follows from these captures.

## Verification and remaining scope

Three controller regressions use actual SQLite storage to cover frozen Unicode passages, geometry confirmation, successful save and memory release; cancellation of departed reads with independent writes and visible departed-write failure; stale document/token replies, supplementary-character note limits and preserved drafts after failure. Workspace formatting, warning-denied workspace/all-target Clippy and workspace tests pass. The local log is `build/verification/annotation-editor-tests.log`.

Creation currently accepts a selection on one page. Multi-page annotation transactions, page notes without a text selection, complete paint coverage, intrinsic/mixed glyph-direction line acceptance, distinct note/marker affordances, mutation history, tag management, global review and Markdown/JSON exports remain pending. Broader real mouse/keyboard editor interactions, fallback-control interaction, compact scrolling, OS clipboard, latency, accessibility and final release gates remain open in the parity matrix.
