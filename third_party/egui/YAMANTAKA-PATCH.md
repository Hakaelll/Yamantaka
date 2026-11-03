# Yamantaka text patch

Runtime source from egui 0.36.2, upstream commit `49682f8baa058bf49e011035cfbd6e825f88a5ef`. Local text selection uses visual stretches and previous-word navigation clamps obsolete cursor positions. The upstream sample home path is neutralized; the Cargo author field retains the publisher's name. Original licenses are preserved.

See [native text layout](../../docs/architecture/native-text-layout.md) for source revisions, maintenance and validation scope. This crate is excluded from the application workspace and used through its pinned Cargo patch.
