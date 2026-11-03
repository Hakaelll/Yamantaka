# Reference replacement waits for its first image

Opening a different document in the reference pane now stages its navigation, preferences, and first page image before replacing the current pane. The initial raster is bounded to 768 pixels per side and 2 MiB, subject to the device and shared memory limits. GPU and transient allocations retain their existing governor charges. The page render has an independent cancellation token and must match the complete requested key before upload.

Saved comparison settings are supplied before preparation, so the candidate uses the saved page, position, zoom, rotation, theme, and optional scroll. The normal reference-opening layout remains single-page. Successful preparation transfers the image and pane together, then confirms the native document set. Failure or cancellation keeps the previous pane and rejects the pending native opening through the existing confirmation protocol.

The Retry action retains the failed document and its saved comparison context. It now reopens that request instead of merely restarting renders of the old pane. Passwords are not part of this retained request. Queue-full admission and delivery failures wait 100 ms before resubmission. Delivery retries receive a new token, so late results from the superseded request cannot be adopted. Closing is checked before upload, and cancellation is checked again after upload before transfer. Successful-reader health recording moves with adoption, preserving the existing quarantine recovery behavior.

## Executed evidence

The capture-only probe records whether adoption had an image, whether the first page paint was empty, and subsequent empty-page paint calls. These are logical painting checks, not physical frame-presentation measurements.

- The native baseline adopted a different reference without an image and recorded six empty-page paints. The corrected normal sequence adopted with an image and recorded zero. Its settled 1800×1200 capture was pixel-identical to the baseline and was visually inspected.
- An injected raster failure in Glow retained the previous reference. The shared Retry handler reopened the failed document and completed with no empty-page paints after adoption.
- Closing after render submission prevented adoption and preserved the primary reading. The native opening was rejected before ordinary reference-close handling was needed.
- An injected queue-full delivery exercised token replacement and delayed resubmission. It recorded one retry, no preparation failure, and no empty-page paints after adoption. This does not establish sustained real queue-saturation acceptance.
- Saved-workspace restoration passed with reference page 1, 90° rotation, 0.8× zoom, and ratio 0.6. The replacement probe recorded an image at adoption and no empty-page paints.
- A protected reference passed the existing incorrect-password/correct-password sequence, with two password submissions and no empty-page paints after adoption.
- Closing an adopted reference passed the existing native-close assertion and preserved the main reading.
- Native debug compilation, workspace formatting, and workspace Clippy across all targets with warnings denied passed. All 39 generated PDFs retained their hashes. No standalone Rust tests, dependencies, or persisted formats were added.

The [status artifact](artifacts/reference-opening.json) records the executable hashes and selected results. Local captures are under `build/verification/reference-opening`.

## Remaining acceptance

Creating the initial comparison of two pages from the same document still mounts a pane before its image is prepared. This checkpoint covers replacement with a different document, including the saved comparison target; it does not close that initial-mount gap. Broader cancellation and real pressure stress, physical DPI/accessibility, paired performance measurements, and the full replacement Release gates remain pending.
