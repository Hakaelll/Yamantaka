# Initial images for restored synchronized comparisons

The restored reference previously prepared its saved page, then changed page on its first visible frame when synchronization followed the main reading. The native workspace scenario reproduced this with a saved reference page of 1 and an effective synchronized page of 0. Its first-paint assertion failed because that frame had no image.

Restoration now applies the existing page, zoom and position synchronization before releasing the deferred comparison. Position feedback is calculated from the planned primary frame using the same canonical layout method as the visible reader. If synchronization exposes a different reference page, a bounded preview is prepared before either pane is presented in the split. The reference keeps its native document ownership throughout.

Reference preparation shares the existing cancellation, source identity, queue retry and governed upload path with staged primary pages. The ordinary page-following rule is shared by preparation and visible comparison. No persisted format or dependency changed.

Executed evidence:

- The native synchronized continuous scenario completes the full workspace create, restore, update, persistence and undo sequence. Saved reference page 1 becomes synchronized page 0. The primary split prepares page 1 and presents pages 0 and 1 with no missing images.
- The synchronized Book scenario completes with a rotated reference and the same effective page transition. Both primary spread pages have images on the first split frame.
- The independent workspace scenario still restores reference page 1 and completes its persistence sequence.
- All three records show a populated reference first paint, zero empty reference paints and zero provisional paints. Primary planned and first-painted geometry match.
- The offline native debug build, workspace formatting and workspace Clippy across all targets with warnings denied pass. All 39 generated PDF hashes are unchanged. No standalone tests were added.

The [status artifact](artifacts/saved-sync.json) records the executable hash and selected native results. Captures remain local under `build/verification/saved-sync`.

These cases exercise page, position and zoom synchronization together. Page-only combinations, unequal document lengths, bidirectional input, cancellation or queue pressure during this additional preparation, rapid resizing and physical presentation still need broader acceptance. This checkpoint does not establish performance targets or final rewrite readiness.
