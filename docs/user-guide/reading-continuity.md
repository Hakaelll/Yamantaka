# Resume reading and unfinished edits

Enable **Restore last session** in Settings to reopen your readings on ordinary startup. An active comparison returns after the main PDF opens, including its reference page, zoom, appearance, divider and following options.

Newly saved comparisons also retain the reference's Document/Book mode, page layout, cover pairing and automatic control hiding. Imported Flutter comparisons use the secondary reading's saved choices. Older native snapshots without these fields keep their earlier single-page defaults. This implementation is awaiting functional acceptance; see [reference reading context](../architecture/rust-comparison-reading-context.md).

Newly saved main readings also retain the position within the page, including horizontal panning. Switching readings, reopening a recently closed reading and opening a saved workspace restore that point using the current window geometry. Older snapshots keep their saved vertical offset. Electronic ink returns to the page start. This implementation is awaiting functional acceptance; see [reading position context](../architecture/rust-tab-position-context.md).

If the reference PDF is missing, the main reading still opens. Yamantaka retains the reference configuration and unfinished work for later recovery. Closing a reference explicitly discards its comparison configuration. Removing a document from the library also removes its recovery context.

New page notes, new annotations on selected text and edits to existing annotations persist locally as drafts. Selected passages are checked against fresh PDF text and geometry before their editor returns. If the passage no longer matches, the draft stays separate. Matching annotation revisions reopen in the editor without saving. If an annotation changed or is unavailable, its recovered edit stays separate. **Copy draft** includes the complete draft even when the panel shows a shortened preview. **Retry recovery** checks the source again. **Discard draft** abandons that recovered edit without changing the annotation in the library.

If an annotation changes while you are editing it, Save is blocked and your text stays in the editor. Reloading the list does not make an older editor current. Preserve any text you need before cancelling and reopening the annotation.

Unfinished title and author corrections also persist locally. If the catalogue review changed, compare the recovered text with the current values and choose **Use recovered text** or **Keep current values**. Recovered text remains unsaved until you choose **Save changes**.

When closing the window with an unfinished note or metadata correction, **Return to editing** keeps the app open. **Discard and close** abandons the open draft. An accepted save finishes before closing.

Disable **Restore last session** to start without reopening saved readings or editors. Drafts stay retained until their matching reading or editor becomes available. Recovery after a graphics failure still attempts to return to the interrupted session without enabling that preference permanently.

Passwords and rendered page images are not part of the saved session. Protected PDFs may require their password again. Named workspaces remain separate reading arrangements. A forced exit can lose edits made since the last background snapshot.

Executed cases and limits are recorded in [comparison continuity](../validation/rust-comparison-continuity-2026-10-31.md), [page-note drafts](../validation/rust-page-note-recovery-2026-10-31.md), [metadata drafts](../validation/rust-metadata-draft-recovery-2026-10-31.md), and [edits to existing notes](../validation/rust-stored-note-recovery-2026-10-31.md), and [selection-note drafts](../validation/rust-selection-note-recovery-2026-10-31.md).
