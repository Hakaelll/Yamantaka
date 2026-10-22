# Bidirectional reference synchronization checkpoint

**Link views → In both directions** lets either pane change the linked page or zoom. The page and zoom checkboxes remain independent. With this option disabled, manual reference page or zoom changes retain the earlier independent-control behavior. Position following always runs from the main reading to the reference, and manual reference page navigation disables it.

Reference controls enqueue a one-time change for the main reading. The existing canonical navigation and preference update paths apply it after the reference viewport has calculated its actual scale. Fit therefore synchronizes the displayed scale even when the two panes have different available sizes. Following a main-reading change does not enqueue a reverse change; unchanged pages and scales are skipped. Opening and closing phases discard reverse requests.

The option is retained with the workspace comparison. Existing native records without it default to one-way following. Adapted Flutter split settings enable both directions for their stored page and zoom synchronization intent. Original Flutter split JSON remains unchanged.

The native binary compiled. One existing storage test and the existing `reader-reference-sync` native scenario passed; no standalone tests or broad suite were added. Storage assertions include the legacy synchronization direction. The native scenario verifies main-to-reference page, zoom and position following, independent manual reference zoom in one-way mode, then reference-to-main page and 90% zoom changes. It also applies Fit from the reference and finally changes the main reading to page 2 at 110%, confirming both views settle there with their links enabled and reverse requests consumed. The [synthetic native capture](../screenshots/rust-reference-bidirectional/reader-reference-sync.png) and control record preserve the final state.

Secondary reader tools, overlays, physical accessibility and performance acceptance, and Release packaging remain pending.
