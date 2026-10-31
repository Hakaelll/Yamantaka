# Nearby CPU preview preparation checkpoint

Date: 2026-10-31. Scope: generation-safe native raster reuse and bounded CPU preview preparation in both reading panes. This is not a complete page-turn, performance or GPU-halo acceptance result.

## Connected behavior

Native raster-cache identity now includes the native document epoch and the requested pixel-content properties. Delivery generation stays in request keys, cancellation checks and returned pages. A cache hit in another generation creates a reply with that generation while sharing the immutable tracked pixel allocation. Reopening/replacing a native document cannot hit an earlier epoch. Page and tile caches use the same typed content identity; dimensions, region and rotation remain significant.

Each pane plans at most two hidden previews from its existing reader geometry. Candidate frames use the same layout, fit, physical sizing and 768-pixel preview cap as the visible pipeline. Planning does not scan the entire document. It waits for visible target rasters and a 200-millisecond settling interval before submitting `Priority::Prefetch` work. This interval is a policy, not a measured page-turn time.

The application actor owns those tickets and publishes a small completion result. It drops the returned raster reference before publication; hidden preparation does not create GPU presentation slots. Cancellation follows the pane/context lifecycle. Errors and admission pressure mark optional work skipped rather than entering the visible reader error path. The state does not retry continuously while idle. Safe mode and non-reader surfaces disable it. The primary viewport also uses its quiet canvas instead of the previous preparation message while layout/preferences are unavailable.

## Executed evidence

Two retained real-PDFium tests were extended, without adding standalone tests:

- `visible_readers_survive_background_eviction_and_two_pins_reject_extra_opens` passed. A prefetch-priority raster and a visible raster in different generations share the same pixel address while retaining their distinct generation tags. Reopening the same ID on another source produces a different live allocation; existing pin/eviction assertions still pass.
- `native_tile_only_allocates_its_buffer_and_rejects_invalid_or_cancelled_regions` passed. A second-generation tile shares its allocation under a budget that fits exactly one tile. A different region is still denied while live consumers hold that lease. The allocation remains charged until the last shared consumer drops, then reaches zero. Existing region/cancellation and unchanged-source-hash assertions still pass.

Offline binary Clippy with warnings denied, an offline native build, application all-target compilation and workspace format checking passed. No broad workspace suite or performance benchmark was run for this unit.

Four controller-driven synthetic native cases passed state and horizontal control-bound checks:

| Case | Observed state |
| --- | --- |
| reader-prefetch-single | Page 1 is visible; the following landscape page is ready in the CPU preview cache, capped at 768 pixels. The pane retains one presentation slot |
| reader-prefetch-comparison | Primary page 1 and reference page 2 stay independently visible. Each pane completes preparation for its other page; the primary still has one presentation slot |
| reader-prefetch-continuous | Both fixture pages are already visible. Two presentation slots remain and no hidden page is prepared |
| reader-prefetch-safe | The visible page renders; preparation is disabled and its request list remains empty |

Single and comparison compositions were visually inspected. The [capture manifest](../screenshots/rust-page-prefetch/capture-manifest.json) records source, lockfile, executable and artifact hashes. Published PNGs contain origin metadata. No Computer Use or personal documents were used.

## Remaining acceptance

These cases do not measure page-turn latency or certify a no-flash first-raster handoff. Hidden GPU residency, the higher-priority nearby presentation tier, region refinement, device-loss recovery, fault/backpressure injection and the broader protected/damaged/rotated/5,000-page corpus remain pending. Native raster calls remain serialized and are not interrupted mid-call. The paired Flutter/Rust p50/p95 runs, physical DPI/Narrator/multi-monitor acceptance, migration acceptance and Windows Release remain required.
