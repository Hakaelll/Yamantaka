# Physical book materials — native visual checkpoint

This checkpoint refines the library's physical books. It is part of the Windows
Rust rewrite, not a release or complete visual acceptance.

| Before | After | Why |
| --- | --- | --- |
| A four-vertex jacket gradient and flat spine strip | A sculpted spine light profile, hinge recess and fixed cloth weave | Make the existing book volume easier to read without changing the six jacket families |
| Thin paper edge at the resting angle | A deeper volume and stronger resting perspective | Separate the cover board, paper block and back board |
| Axis-aligned selection rectangle | Selection outline and marker projected with the jacket | Keep selection attached to the book as the pointer-driven spring tilts and lifts it |

The material uses one 90-vertex base mesh and, for covers at least 170 points
wide, at most 96 weave strokes. The weave is deterministic in book coordinates;
it does not change with time. Existing title, artwork and author geometry share
the same rigid projection. Miniatures omit weave detail. The existing spring
and lift remain interruptible, stop requesting repaint after settling, and reset
in reduced-motion mode. This change introduces no autonomous entrance or idle
animation. It does not establish a new frame-time or memory benchmark.

## Verification

- `cargo fmt --all --check`: passed.
- `cargo test -p yamantaka --locked`: all 93 UI tests passed, including spring
  settling, rapid lift retargeting and reduced-motion reset.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- Native executable SHA-256:
  `39E8352928767F297E3F661B632E5C7C7F7571C03AE31BDBF5969E3015EE0C39`.
- Seven successful native captures: dark, light, details/selection, Spanish,
  tilted, reduced motion and 640×480 with a 2× logical text scale.
- Independent verification matched all 62 UI source files, Cargo.lock,
  executable, image/control hashes and successful capture exits. Publishing
  added origin metadata without changing PNG pixel chunks. Every wide capture
  keeps its enabled continue-reading action within the upper half of the viewport.
- Impeccable's source detector returned no findings; it does not certify native
  accessibility or visual quality. Committed PNGs contain synthetic origin metadata.

See [the capture manifest](../screenshots/rust-bound-cloth/capture-manifest.json)
and [dark library capture](../screenshots/rust-bound-cloth/library-dark.png).
Only the isolated synthetic collection was used. No personal library was opened.

## Remaining work

The overall interface still needs broader visual work on the reader, tools,
inspector and compact states. The small 2× capture remains a scrollable, partial
first-row view; it is not proof of complete keyboard or scroll interaction.
Physical monitor DPI, Narrator, contrast across arbitrary imported artwork,
large-library frame times and user evaluation remain pending. No whole-rewrite,
performance-improvement or final-release claim follows from these screenshots.
