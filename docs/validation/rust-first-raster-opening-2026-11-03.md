# Primary reader adoption waits for a first image

The primary reader now prepares its saved preferences and a bounded preview of the requested opening page before replacing the current reading. Previously native open completion retired the earlier page surface before either result arrived. A native baseline reproduced adoption without an image and two empty-page paint calls during a synthetic reading switch.

## Preparation and ownership

One candidate belongs to the exact pending open request. Preferences use the existing repository actor; preview rendering uses the existing serialized PDF worker at visible priority. Its whole-page raster is at most 768 pixels per side and uses a share of the existing page-memory budget, capped at 2 MiB. No new worker, disk cache, or persistent format is introduced.

The candidate receives its own render cancellation token. Results must match that token and the complete requested render key. The preview uploads through the existing governed wgpu or Glow presenter before adoption. The image keeps its actual raster dimensions and rotation; only its delivery generation changes when it enters the reader surface. Normal layout and refinement then use the existing page mesh and render plan.

The earlier reader remains the confirmed native owner while the candidate is prepared. Its session and tab context are settled at adoption rather than at native-open completion. Annotation-edit and request-identity guards run again before replacement. Incoming saved session context belongs to the pending opening, so navigation in the earlier reading does not consume it. Native confirmation follows adoption. Replaced or failed candidates cancel their requests and release prepared images; queue pressure retries admission through the existing finite repaint delay.

A first-preview or preferences failure preserves the current reading and rejects the pending ownership handoff. The former reader remains available for retry. Comparison restoration now also waits while the primary confirmation is awaiting submission. Its initial regression run exposed an attempted reference opening before admission; the corrected complete workspace run passed.

## Executed evidence

- The baseline adopted a synthetic catalogue entry without an image and recorded two empty-page paint calls. The matching fixed sequence adopted it with an image and recorded zero.
- Native second-page opening passed with Glow. A separate wgpu run switched to a different generated PDF, injected an invalid first-preview page request, verified retention of the earlier reading, and successfully retried the requested second page without empty-page paints.
- The existing native failed-open scenario still preserved its earlier continuous reading and settings. Protected opening with a temporary password passed. The complete saved-workspace/comparison scenario passed after the confirmation-order correction and restored the expected panel width and columns.
- The settled distinct-PDF capture showed the expected second page. All 39 generated input PDFs retained their hashes across the final capture runs.
- Native debug compilation, workspace formatting, and workspace Clippy across all targets with warnings denied passed. A capture-only native self-check was added; no standalone Rust test cases or dependencies were added. The full workspace test suite was not rerun for this checkpoint.

The [status artifact](artifacts/first-raster-opening.json) records outcomes without document paths. Local captures remain under `build/verification/first-raster-opening`.

This checkpoint prepares the requested primary opening page. It does not establish complete first-visible-frame coverage for facing/continuous layouts, staging of reference-pane replacement, rapid supersession/closure stress, GPU-pressure fault acceptance, physical input/accessibility, or paired performance targets. The observed paint counts are functional evidence, not measured presentation latency or a universal no-flash guarantee.
