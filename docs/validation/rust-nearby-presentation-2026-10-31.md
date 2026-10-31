# Nearby GPU presentation checkpoint

Date: 2026-10-31. Scope: bounded wgpu preview residency and transfer into visible page slots. This is not a p95 performance acceptance result.

## Connected behavior

Each pane can prepare at most two adjacent 768-pixel previews after visible pages and crop refinement settle. Native requests for GPU preparation use `Priority::Nearby`; CPU-only preparation retains `Priority::Prefetch`. Both remain below visible-page requests. Native PDFium work remains serialized.

The application actor delivers the tracked immutable raster to the owning optional-preview consumer. The default compositor uploads it using the existing page-memory governor, device bounds, aligned staging admission and fence-bound texture ownership. The result reference is dropped after delivery. Upload denial leaves CPU preparation usable and does not set a visible reading error. Glow does not retain optional GPU previews; safe mode disables preparation.

Navigation cancels pending optional consumers while preserving completed preview textures until the next visible geometry pass can claim them. Matching includes document, page, region, physical dimensions and rotation. A match can transfer the unique texture lease into the new delivery generation; the page slot cancels any redundant pending preview request. It then uses the existing whole-page or visible-region refinement pipeline. No GPU upload is needed for that transferred preview. Document replacement clears the preparation state; remaining unmatched textures retire when planning changes. Hidden previews do not create visible page slots.

## Evidence

Native synthetic profiles exercise a prepared page turn in one pane and independent turns in both comparison panes. The controller waits for GPU previews, navigates through the real reader methods and waits for promotion. Recorded summaries include prepared and promoted generations, texture identity, promotion counts, residency and tracked memory. Capture assertions require promotion in each exercised pane, distinct navigation generations, at most two hidden GPU previews, no redundant small preview raster for the newly visible primary page and the configured page-memory bound.

Additional profiles exercise safe mode and the alternate Glow path. [The capture manifest](../screenshots/rust-nearby-presentation/capture-manifest.json) records artifacts, source and executable hashes. Clippy with warnings denied, formatting and the offline native application build are checked for this checkpoint.

## Limits

No page-turn latency or universal no-flash result is claimed. Rapid navigation and scrolling, memory-pressure injection, device-loss reconstruction, the broader PDF corpus, full GPU/Glow accounting and paired Flutter/Rust performance runs remain pending. Physical DPI, Narrator and multiple monitors were not exercised. Captures use synthetic fixtures and contain no personal documents.
