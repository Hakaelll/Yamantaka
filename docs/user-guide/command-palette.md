# Find actions and recent documents

Press **Ctrl+K**, or choose **Commands** from Tools or the reader's options menu. Settings → Shortcuts lets you change this assignment.

Type an action name in Spanish or English. Search also accepts command identifiers and ignores common Spanish accents, so `pagina` finds page actions. Use **Up/Down** to select, **Enter** to run, and **Escape** or **Close** to dismiss. You can also click a result. The active keyboard assignment appears beside actions when space permits and in their tooltip.

The palette identifies the main or reference reading it will affect. Reader actions keep that document context while the palette is open. If it is replaced or closed, the old palette is dismissed. Comparison synchronization settings still apply to navigation and zoom. Commands use the same handlers as keyboard shortcuts.

Available actions include opening PDFs, library search, in-document search, page/history navigation, reader and tab controls, page layouts, reader themes, navigator sections, temporary A/B positions, document information, settings, library notes, and reading history. Unset temporary-position destinations remain disabled. Save, jump, and swap operations apply to the focused reader's own marks.

**Document information** shows the focused document's title, author, filename, and current native PDF page count. It is a read-only view. Use the book's library details to edit metadata.

The last eight catalogue documents, ordered by last opened, appear as **Recent document** results. Their snapshot is independent of current library filters. Search matches their title, author, or filename within bounded display/search text. Choosing one opens it through normal document lookup and tab opening. A record marked unavailable cannot be opened from the palette; other opening failures use the existing reader recovery flow.

Actions are available immediately while the recent-document snapshot is read in the background. A failed recent read leaves actions usable; close and reopen the palette to retry that snapshot. Closing cancels its outstanding query. A query with no matches shows an explicit empty state, and Enter does not run a background reader action.

The native palette does not yet provide annotation undo/redo. Those remain required rewrite work. Physical accessibility, keyboard-layout and full performance acceptance are recorded separately from the native scenarios in the [validation checkpoint](../validation/rust-command-palette-2026-10-31.md).
