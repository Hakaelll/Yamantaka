# Independent reference search checkpoint

The reference header now has a search popup with phrase entry, case and whole-word options, cancellable progress, bounded match batches, result selection and previous/next navigation. Results highlight in that pane and use its canonical navigation. Page synchronization determines whether selecting a reference result also moves the main reading. The main query, selected match and cancellation token remain separate.

Both panes use the extracted FindState controller for requests, bounded traversal, wrap behavior, selected targets and highlight geometry. Actor replies route by document and exact cancellation identity before the main search handler. Closing the popup stops a running reference scan; closing the reference cancels it before native closure. Dropping or replacing the pane releases its search state and batch leases. PDFs remain unchanged.

The native binary compiled. Two existing native scenarios passed; no standalone tests or broad suite were added. `reader-reference-book` starts and cancels a reference query, then searches for Mixed in a distinct reference PDF. It selects the page-2 match, retains the main reading on page 1 with its search state untouched, and checks the result button is enabled and wholly inside its clip. The [reference capture](../screenshots/rust-reference-find/reader-reference-book.png) shows that selectable result. The popup uses one scrollable content area with room for asynchronous results to avoid retaining the smaller height measured during scanning.

`find-previous-wrap` verifies the main search's existing reverse traversal and wrap behavior through the shared controller. Its [native capture](../screenshots/rust-reference-find/find-previous-wrap.png) and the adjacent control records retain that evidence. The [comparison guide](../user-guide/comparing-pdfs.md) describes the connected user flows.

Other secondary reader tools, per-pane keyboard shortcuts, overlays, physical accessibility and performance acceptance, and Release packaging remain pending.
