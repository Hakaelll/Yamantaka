# Reference position following checkpoint

**Link views → Follow position** follows the main reading's canonical page and normalized PDF point. It works independently of zoom following and includes page changes. Manual reference page selection disables both page and position following. The reference's finite canvas may clamp a point near its edges; this does not feed back into the main reading.

The reference remembers the last source point, so an unchanged source does not repeatedly reposition a clamped viewport. Motion within the same page updates its scroll anchor without cancelling page rendering or creating navigation history entries. Changing page resets the reference's page requests. Zoom changes, resizing and divider movement allow the source point to be applied again in the new geometry. Opening another reference clears the link and its remembered point; opening and closing phases suspend propagation.

The existing native `reader-reference-sync` scenario was extended rather than adding standalone tests. It follows page 2 and 160% zoom, moves to a visible point 5% down that page and checks the actual position in both views. Reapplying the unchanged point must preserve the reference render generation. It then verifies that manual reference zoom can return to 80% while the main reading remains at 160%. The native binary compiled and this scenario passed. The [synthetic capture](../screenshots/rust-reference-position/reader-reference-sync.png) and adjacent control record retain its result. No broad suite was run.

This is one-way following from the main reading. Secondary reader tools, saved comparison restoration, physical accessibility checks and full release validation remain pending.
