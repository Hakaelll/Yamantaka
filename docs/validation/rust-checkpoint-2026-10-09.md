# Rust checkpoint validation — 2026-10-09

This is an in-progress checkpoint, not the final rewrite or a replacement Release.

- `cargo fmt --all --check`: passed after the source updates.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo test --workspace --offline`: **99 passed**, no failures or ignored tests.
- `cargo build -p yamantaka --offline`: passed.

The [test transcript](rust-checkpoint-tests-2026-10-09.txt) redacts the local repository path. Raw transcript SHA-256: `8BB7E73B1C3D2FF947E746A2B6B8960E46438587152DCACB0ED86FD069BCA80B`.

Coverage includes real native PDFium fixtures, request saturation and shutdown, SQL during blocked PDF work, populated historical schemas and backups, lossless reader preferences, virtualized mixed-page geometry, native DX12 readback/retirement, FTS repositories and native Windows activation.

The incremental search controller compiles but does not yet have dedicated controller tests or actor/UI integration. The latest reader UI connects cached single/facing/continuous geometry, per-page independently cancelable render consumers, retained replacements and canonical navigation. The active PDF metadata is shared rather than copied each frame. This does not establish complete reader parity: contextual persistence, e-ink, auto-hide, progressive tile refinement, navigator tools and annotations remain incomplete.

Native synthetic reader captures and actual custom-control bounds are stored under `docs/screenshots/rust-reader-preferences`. Their manifest identifies the exact executable, source hashes, physical scale and arguments. They cover page fit, width fit, facing pages, continuous presentation, rotation/sepia, categorized options and a 640×480 window with 200% text. They do not certify physical DPI, Narrator, multiple monitors or arbitrary personal PDFs.

The [acceptance matrix](../architecture/rust-rewrite-parity.md) remains the authority for the entire objective. The Flutter checkpoint `c773acf` and its installed data remain independent and recoverable. Final packaging, release publication and full performance comparison are pending.
