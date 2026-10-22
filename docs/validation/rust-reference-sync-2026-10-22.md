# Reference page and zoom following checkpoint

The reference header groups **Follow page** and **Follow zoom** under **Link views**. Each option follows the main reading in one direction. Zoom copies its actual displayed scale, including changes caused by fitting the main page. Manual reference zoom or Fit disables zoom following; manual page navigation disables page following. Opening another reference resets both options.

Page following changes the reference only when its page number differs, clamping to its last page when the main document is longer. It preserves scrolling within the same page instead of repeatedly resetting the normalized position and restarting rendering. Following pauses while the reference is opening or closing. Zoom requests change only when the scale changes, retaining existing page imagery during replacement.

The native binary compiled and the focused `reader-reference-sync` scenario passed. It enabled both links, moved the main reading to page 2 at 160%, and confirmed that the reference followed. It then changed the reference to 80% and confirmed that zoom following stopped, the main reading stayed at 160%, and both remained on page 2. The [native capture](../screenshots/rust-reference-sync/reader-reference-sync.png) shows the compact menu after that independent zoom change; the adjacent control record records both actual scales. No broad suite or new standalone tests were run.

Bidirectional position synchronization, secondary reader tools, saved split restoration, physical accessibility acceptance and full release validation remain pending. This is a development checkpoint, not a completed rewrite or Release.
