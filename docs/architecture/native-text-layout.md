# Native bidirectional text

Yamantaka keeps egui and epaint at 0.36.2 and carries their text corrections in `third_party/egui` and `third_party/epaint`. Cargo resolves both through local patches. There is no runtime download or alternate web renderer.

The unmodified release shapes glyphs with HarfRust, but does not reorder mixed-direction lines correctly and can emit more layout glyphs than logical characters. That breaks both displayed word order and editor cursor positions.

The local patch resolves paragraph embedding levels with pinned `unicode-bidi` 0.3.18. It splits shaping runs by font, direction and strong script, using HarfRust's own script data. Common and inherited characters remain attached to their run. Composite outline clusters share one atlas entry when needed, while layout glyphs retain logical character order. Copying therefore uses the original text.

After wrapping, line whitespace resets to the paragraph level before visual reordering. Hit testing, cursor positions, selection backgrounds and IME underlines use visual positions while selection ranges remain logical. ASCII text skips bidi resolution. Additional installed Segoe UI and symbol fonts extend the existing fallback families; proprietary font files are never redistributed.

## Source and maintenance

- Registry base: egui/epaint 0.36.2, upstream commit `49682f8baa058bf49e011035cfbd6e825f88a5ef`.
- Initial bidi and cluster design: [upstream PR 8577](https://github.com/emilk/egui/pull/8577), commits `c55da0337f67616ee7be4bfb06b73fdd71ef7f91` and `c93e8863e958507056415e2a6dace56e23ba5f3a`.
- Visual selection stretches and cluster refinements: [upstream PR 8818](https://github.com/emilk/egui/pull/8818), reviewed at `87476c112a338f8940c64979dc6b46b90612e195`.
- Local additions include the 0.36.2 backport, script boundaries, per-line whitespace reset and native verification. The upstream proposals are drafts; this is a Yamantaka patch, not an upstream release claim.

Both crates retain their original MIT and Apache license texts. Their normalized Cargo author metadata keeps the publisher's name. An upstream sample user path in `callstack.rs` uses a neutral workspace path instead. No registry cache, font binary, target directory or downloaded executable belongs in these source directories.

The Windows packaging inventory hashes the patched Rust sources, manifests and license files. The public-checkpoint guard permits the publisher's email only in the two exact upstream MIT license blobs, preserving attribution without accepting arbitrary personal data.

The egui override also provides opt-in immediate wheel completion and a kinetic-scroll switch for the [electronic-ink reader profile](../user-guide/reader-profiles.md). Default wheel smoothing and drag inertia remain enabled. These input additions are local Yamantaka changes, with native positive-control evidence in the profile validation report.

Replace these patches when a stable compatible upstream version passes the same native cursor, copying, mixed-script, wrapping and Latin rendering checks. Do not remove logical character mapping or replace it with reordered stored strings.

The [native validation record](../validation/rust-bidirectional-text-2026-11-03.md) defines the executed scope. Full Unicode conformance, IME interaction, physical Narrator and wider script/font coverage remain separate acceptance work.
