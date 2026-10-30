# Reference note editing checkpoint

The reference navigator now includes Notes. Its editor and list share the main reading implementation for individual page notes, passage annotations, saved-note editing and removal, colors, tags, review state, geometry fallback, and 80-record pagination. The reference selection popup exposes Annotate selection. Pane-owned constructors freeze the correct document, fingerprint, page, rotation, and selection snapshot before preparing the existing typed mutation.

The extracted presentation emits actions. SQL and PDFium remain behind the application runtime. Each pane retains its own AnnotationState, query token, immutable record snapshot, and draft. Replies require matching document and request identity. Closing the navigator cancels reads while preserving drafts and accepted writes.

A settled mutation refreshes lists and marks for the matching document in both panes, including failures whose mutation may have committed before its follow-up read failed. Refresh retains an open draft. A stored draft that predates another write disables Save until the reader cancels and reopens it. Read retries preserve that conflict status. Global review actions also respect drafts in either pane.

An unsaved reference draft blocks reference closure or replacement and exposes a save-or-cancel message. Once a write has been accepted, closure retains its document and cancellation-token ownership separately until acknowledgment. The application can then refresh another view of that document and surface a failure after the original editor has closed.

## Evidence

The final native binary compiled offline, and cargo fmt --all --check passed. Five native scenarios completed through tool/capture_rust_ui.ps1:

- reader-reference-notes uses two document identities. The reference creates a page note on page two, edits it, removes it, and creates a final page-one note. The main reading retains its two original notes and page.
- reader-reference-notes-shared-close shows the same PDF in both panes. An accepted reference edit is followed immediately by reference closure. The main list receives the updated note, the retired write settles, and the native PDF stays open for the main reading.
- reader-reference-notes-editor-small-200 retains an unsaved reference draft in Spanish, light theme, reduced motion, a requested 640 by 480 window, and 200% text scaling. The scenario scrolls the editor to its action row and checks visible Save note and Cancel controls.
- annotation-editor-light-es retains the existing main-editor presentation after extracting its shared components.
- annotation-multi-saved-es retains the existing primary multi-page annotation save scenario.

The scenarios invoke the same controller actions and real SQLite application actor used by the UI. They assign draft values programmatically, so they do not certify physical typing, pointer interaction, clipboard access, or Narrator. Captures use synthetic documents and isolated records. No Computer Use, personal PDFs, new standalone tests, or broad workspace suite were used.

The [capture manifest](../screenshots/rust-reference-notes/capture-manifest.json) records executable, source, image, and control hashes. Those hashes were checked against the final working tree and binary before publication.

## Remaining acceptance

The reference popup does not duplicate advanced annotation filters, bulk actions, or export. Those remain available in the main and library Notes tools. Reference selection creation is connected through the shared constructor but is not exercised by these five scenarios. Stale-draft conflict handling, unsaved-close rejection, fault recovery, broader mutation concurrency, and memory-pressure behavior need additional interaction evidence. Physical DPI, keyboard and accessibility acceptance, complete performance comparison, and Release packaging remain pending.
