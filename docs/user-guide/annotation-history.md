# Undo annotation changes

Open the reader's Notes panel and choose **Undo** or **Redo**. With the document canvas focused, use **Ctrl+Z** and **Ctrl+Shift+Z**. You can change these assignments in Settings → Shortcuts, or find the actions in **Ctrl+K**. Text fields retain their own editing shortcuts.

History belongs to each document. In comparison reading, the focused pane determines which document changes. Two panes showing the same document share its saved annotations. Unsaved editor drafts must be saved or cancelled before using annotation history.

Creation, editing, deletion and reader note batches can be reversed. A multi-page creation is one step. Undo restores stored geometry, timestamps, inherited extension columns and annotation tag links. It also refreshes annotation search results and visible marks. Original PDF files are never changed. Created tag definitions remain in the tag catalogue when their annotation links are undone.

A new change after Undo discards that document's redo branch. History retains up to 100 changes per document, subject to an 8 MiB combined history budget. Older entries expire first when that budget fills. Memory admission can reject an oversized operation before saving it. A failed or cancelled transaction leaves its history position unchanged.

History is temporary and is not included in backups. Closing the application releases it. Attachment revalidation and cross-document review batches invalidate affected document histories. If another operation changes a recorded annotation, Undo refuses to overwrite that newer state and shows an error in Notes.

See the [validation checkpoint](../validation/rust-annotation-history-2026-10-31.md) for exercised paths and remaining acceptance work.
