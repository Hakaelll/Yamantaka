# Comparison position swapping checkpoint

**Link views → Swap positions** exchanges the visual positions of the main reading and its reference. In a stacked layout this places the reference above the main reading. Each pane retains its document, page, zoom and canonical anchor. Navigation, following options and difference marks keep their existing main/reference roles.

The divider ratio continues to describe the main reading's share of the available space. Swapping preserves that share and reverses the drag direction used to update it. Geometry changes request canonical anchor restoration. The operation only changes presentation state; it does not reopen PDFs or replace the active reading. Workspace comparison saves now retain the selected order, and older records default to the main reading first.

The native binary compiled. One existing storage test and the existing `reader-reference-book` native scenario passed; no standalone tests or broad suite were added. Storage round-tripping includes the swapped order. The native scenario opens a distinct reference PDF, sets its independent zoom to 120%, swaps positions, and confirms the original main document remains at page 1 with its original zoom setting while the reference retains its identity, page and scale. No reference close was issued. The control record additionally checks that the reference's Close button lies in the left half of the window. The [synthetic native capture](../screenshots/rust-comparison-swap/reader-reference-book.png) shows that arrangement.

This checkpoint directly checks side-by-side swapping. Physical acceptance of stacked resizing, secondary reader tools, overlays, accessibility and performance acceptance, and Release packaging remain pending.
