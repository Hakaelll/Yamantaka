# Reader composition checkpoint

The native reader now groups page navigation and zoom controls on separate
raised surfaces. The document title uses the bundled editorial face. The paper
shadow is softer, retaining a small offset so the sheet remains distinct from
its reading environment. A dedicated toolbar module replaces the inline
composition; navigation, zoom, fit, options and panel commands are unchanged.

| Before | After | Why |
| --- | --- | --- |
| Page and zoom controls mixed in one undifferentiated row | Two compact control groups | Make their purpose and boundaries easier to scan |
| Document title styled like a utility label | Source Serif 4 Display at 22 points | Give the current document a clear identity |
| Heavy paper shadow | 3-point offset, 12-point blur, lower opacity | Keep the paper distinct without a dark rim dominating the reading area |

Verification: 93 native UI tests passed; workspace Clippy with warnings denied
and `cargo fmt --all --check` passed. Six native synthetic captures cover tabs,
the default reader, a Spanish light-theme text panel, facing pages, options and
640×480 at 2× logical text scale. The source hashes, executable hash and capture
conditions are recorded in the [manifest](../screenshots/rust-reader-composition/capture-manifest.json).
Published PNGs embed their synthetic origin. This is a visual checkpoint, not
proof of physical DPI, Narrator, full keyboard interaction, performance or
completion of the rewrite. It adds no animation to repeated navigation actions.

The next library requirement is to display PDF-derived cover imagery on the
physical 3D books, matching Flutter's connected cover-image path. Procedural
jackets are a fallback; they must not replace available PDF covers.
