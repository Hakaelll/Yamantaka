# Prepared page textures survive geometry changes

The default compositor can now reuse a prepared nearby page after rotation or viewport geometry changes. Previously promotion required the exact prepared dimensions and rotation. Planning also discarded prepared textures when those values changed, even though the page itself remained valid.

Prepared images now retain their actual render key alongside the texture. Promotion requires the same document and page and a whole-page image. It transfers the texture to the visible slot with its original dimensions and rotation, updating only the delivery generation. The existing page mesh maps that original rotation to the current reading rotation. The normal render plan then obtains the appropriate replacement without treating the earlier raster as a completed render for the new geometry.

Planning preserves matching pending work and can carry a usable image into a changed target. Navigation cancels optional work without discarding an available image. Promotion cancels any redundant optional replacement request. If an optional GPU upload fails, an earlier usable texture remains available. Source replacement still clears the preparation state; no texture moves between documents or pages. Residency remains bounded at two nearby slots per pane under the existing memory governor.

## Executed evidence

The native baseline prepared neighboring pages in two readers, rotated both by 90 degrees, and navigated through the normal reader methods. It recorded zero promotions and three empty-page paint calls in the primary pane, with four in the reference pane.

The fixed sequence promoted a prepared texture in each pane and recorded zero empty-page paint calls in both. An additional native sequence allowed a layout-planning pass between rotation and navigation. Its assertions verified that the original textures survived with their original render rotation while the targets changed; both were then promoted without an empty-page paint. The settled capture showed correctly rotated pages in both panes.

An ordinary prepared-page turn still promoted both textures without a geometry mismatch. An alternate OpenGL reader smoke completed with its existing CPU-only preparation. It does not receive the GPU-promotion improvement described here.

All 39 generated input PDFs retained their hashes across the final native runs. Recorded page-buffer allocations stayed within the existing 72 MiB category limit. That invariant does not measure complete process or driver memory. The native debug build, workspace formatting, and workspace Clippy across all targets with warnings denied passed. The existing native self-check was extended; no standalone Rust tests or dependencies were added.

The [status artifact](artifacts/nearby-geometry.json) contains the native outcomes. Captures remain local under `build/verification/nearby-rotation`.

These counts describe the exercised synthetic sequences, not latency, frame presentation, or a universal no-flash guarantee. Cold or unprepared pages, opening another document, the broader PDF corpus, GPU pressure/failure injection, physical interaction, and paired performance acceptance remain pending.
