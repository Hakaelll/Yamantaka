# Page reference checkpoint

The later [reference document checkpoint](rust-reference-book-2026-10-22.md) adds a second catalogued document and acknowledged native closure. This page records the earlier same-document feature.

Open a PDF, then choose **Reading options → Compare pages**. The full reader stays available beside a reference view of the same document. The reference has its own page selector, zoom controls and whole-page fit. **Follow page** follows the main reader's page; choosing a reference page independently disables that option. Drag the divider to change the proportions. Narrow windows stack the views. **Close reference** returns the space to the main reader.

Both views use the same native PDF handle, application runtime, GPU presenter, memory governor and page-painting implementation. Each owns its render requests and accepts replies only for the matching key and cancellation token. The existing 14 MiB preview allowance is divided between the views. Clearing the reference cancels its consumers and releases images without closing the shared document. Accepted document replacement, leaving the reader and application shutdown also clear it. The comparison action requires the native GPU presenter.

The binary compiled and the two bounded native scenarios passed: `reader-reference` and `reader-reference-stacked`. Their control records verify the main view on page 1, the reference on page 2, the same synthetic document ID and a resident reference image. The [lateral capture](../screenshots/rust-page-reference/reader-reference.png) and [stacked capture](../screenshots/rust-page-reference/reader-reference-stacked.png) show both orientations. No broad test suite was run.

The captures record the initial two-page layout. A subsequent anchor adjustment restores the canonical positions when entering or leaving comparison, resizing the window or dragging the divider; physical scroll-restoration acceptance is still pending.

This is a transient page-reference feature. Secondary annotations, text selection and other reader tools, two-document admission in the UI, bidirectional synchronization, saved split restoration and physical keyboard/accessibility acceptance remain pending. It does not establish full split-reader parity or Release readiness.
