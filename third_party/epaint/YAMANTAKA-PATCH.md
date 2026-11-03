# Yamantaka text patch

Runtime source from epaint 0.36.2, upstream commit `49682f8baa058bf49e011035cfbd6e825f88a5ef`. Local changes cover bidirectional levels, script boundaries, composite outline clusters, visual line ordering, cursor hit testing and selection stretches. Original licenses and publisher attribution are preserved.

See [native text layout](../../docs/architecture/native-text-layout.md) for source revisions, maintenance and validation scope. This crate is excluded from the application workspace and used through its pinned Cargo patch, with `unicode-bidi` fixed at 0.3.18.
