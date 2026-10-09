# Native reader thumbnails

Navigator → Pages now displays PDF page previews and uses canonical page navigation
when a page is selected. The list follows the active page after reader navigation.
User rotation applies to both the preview and the main page. The compact drawer
uses a single-row section selector and horizontal preview rows, with Spanish and
English labels, selected-page and keyboard-focus treatments, and hover animation
that respects reduced motion. Loading previews remain selectable. Rendering or
upload failures expose details and an explicit retry action.

## Scheduling and ownership

Render actions now carry the domain Priority value. The existing actor forwards
it to the same native worker: the reader requests Visible, and previews request
Thumbnail. All previous callers were migrated. No second PDF engine or unbounded
thumbnail job queue was introduced.

The egui virtual row range, including its small overscan, owns at most sixteen
preview slots. Abandoned slots are cancelled before admitting their replacements.
Every preview has its own cancellation identity; accepting a render result now
requires both its complete key and its consumer identity. Matching page numbers
or keys cannot deliver a different consumer's result. Leaving the panel, changing
documents or rotation, retrying, and shutdown release the preview slots. The
thumbnail slots are cleared before the native presenter shuts down.

Each preview's longest physical side is capped at 256 pixels and the actual device
limit. CPU rasters reuse the governor-backed native cache. WGPU uploads reuse the
existing admitted texture and staging path, including deferred retirement. Those
resources share the existing 72 MiB Pages and 104 MiB total limits. These are
payload accounting limits, not a claim about total process or driver memory.
Glow upload accounting and device-loss recovery remain outstanding.

## Behavior and native evidence

Three new state/geometry tests cover independent cancellation on scrolling,
document and rotation changes; stale render and failure rejection; retry cleanup;
and literal preview dimensions with rotation, large scale and small device caps.
The existing blocked-native actor test now checks both Visible and Thumbnail
priority delivery and confirms that a library SQL query completes while either
render ticket remains unresolved.

Native captures cover dark English, light Spanish, requested 640×480 at 200% text
zoom, a page-2 jump, user rotation of 90°, and the final page of the synthetic
5,000-page PDF. The jump cases invoke the same navigation handler as selecting a
preview; they do not certify physical mouse, keyboard or Narrator operation.
The compact review corrected a row that extended below the viewport. The study
fixture title now agrees with the synthetic catalogue title.

The final-page check displayed 5000 / 5000 and retained five thumbnail slots.
Its recorded tracked payload was 16,890,864 bytes total, including 16,610,800 bytes
in Pages, 280,064 bytes in Other, and no live staging bytes at capture time.
This is a point-in-time bounded-payload observation, not a startup, peak-memory or
paired Flutter performance benchmark. The ordinary study cases retained three
slots. Previews beyond the sixteen-slot presentation limit remain selectable
placeholders in exceptionally tall viewports.

The isolated PNGs, control geometry, viewport scale, memory observations, source
hashes and executable hashes are recorded under ignored
`build/rust-visual/thumbnails-final`. The existing generated study and 5,000-page
fixtures are synthetic. No personal library was opened by these checks.

```text
cargo test --workspace --locked                              166 passed
cargo clippy --workspace --all-targets --locked -- -D warnings passed
cargo fmt --all -- --check                                   passed
cargo rustc -p yamantaka --locked -- -o build/verification/yamantaka-thumbnails-preview.exe
```

The alternate executable preserves the user's already open demonstration window.
Physical DPI, keyboard/Narrator behavior, device-loss handling, large-corpus
latency, in-document search, PDF links, continuity and final release acceptance
still require their remaining gates. The full rewrite remains in progress.
