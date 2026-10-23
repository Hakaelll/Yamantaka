# Reference PDF contents checkpoint

The reference header now offers a contents popup with the PDF's chapter hierarchy. Sections can expand or collapse, and selecting a destination uses the shared canonical PDF resolver for page, position and zoom. Comparison following options then determine whether the main reading follows. Loading, collapse state and destinations remain separate from the main contents panel.

Both views share outline request handling and the virtualized row presentation. Replies are matched by document and cancellation identity. Closing the popup cancels an outstanding reference request; closing or replacing the reference releases its state. Empty outlines and recoverable failures use the existing feedback and retry behavior. Expanding or collapsing explicitly requests another frame.

The normal reference toolbar uses a compact Close icon with the existing accessible name and tooltip. This makes room for its contents control without retaining the wider text button.

The native binary compiled. Two focused native scenarios passed; no standalone Rust tests or broad suite were added. `reader-reference-outline` loads six entries in a distinct reference PDF, collapses a section to five visible rows, expands it to six, and selects the Make room for thought destination. The reference reaches page 2 at 150% while the main reading remains on page 1 with its outline state untouched. The selected chapter control is enabled and wholly visible. The [reference contents capture](../screenshots/rust-reference-outline/reader-reference-outline.png) and adjacent control record retain that evidence. `outline-passage` also passed for the main reader through the shared view and resolver; its [capture](../screenshots/rust-reference-outline/outline-passage.png) records that regression check.

Other secondary reader tools, overlays, physical accessibility and performance acceptance, and Release packaging remain pending.
