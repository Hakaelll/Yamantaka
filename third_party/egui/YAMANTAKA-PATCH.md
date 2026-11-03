# Yamantaka text patch

Runtime source from egui 0.36.2, upstream commit `49682f8baa058bf49e011035cfbd6e825f88a5ef`. Local text selection uses visual stretches and previous-word navigation clamps obsolete cursor positions. The upstream sample home path is neutralized; the Cargo author field retains the publisher's name. Original licenses are preserved.

See [native text layout](../../docs/architecture/native-text-layout.md) for source revisions, maintenance and validation scope. This crate is excluded from the application workspace and used through its pinned Cargo patch.

The input/scroll patch also exposes wheel-smoothing completion and an opt-out for drag inertia, used by the electronic-ink and reduced-motion profiles. `ScrollArea::animated(false)` now completes an already pending scroll target immediately instead of continuing its earlier animation. This preserves the target position when the motion policy changes between frames. The latest pending-target adjustment has not been compiled or exercised at the user's request.
