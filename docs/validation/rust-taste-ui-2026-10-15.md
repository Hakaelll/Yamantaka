# Library hierarchy and compact reading strip

The user requested an interface review with
[Taste Skill](https://github.com/Leonxlnx/taste-skill), followed by a smaller
Continue reading block and progress bar. The main skill and its
`redesign-existing-projects` companion were installed outside the repository.
The companion applies to existing apps; the main skill explicitly targets
public web pages. Its browser inspection scripts do not evaluate this Rust UI.

## Review and changes

The review compared the existing native screenshots with the connected egui
components. The graphite/ivory palette, local interface typography, editorial
titles, PDF textures and physical book geometry already provide a coherent
identity. The useful changes were clearer catalogue captions and less space
spent on the reading shortcut.

- Catalogue titles increase from 15 to 16 logical points and retain two lines.
  A separate author line makes documents easier to distinguish. Both fields
  truncate with ellipses. Missing or blank authors use localized fallback text.
- Started books have a two-point progress track below their metadata. Unread
  books retain their numeric progress without an empty decorative track.
- The book button's semantic name includes the full title, author, page count
  and percentage, with English or Spanish page labels.
- The Continue reading miniature shrinks from 96 points in the spacious view,
  or 72 with the inspector, to 42. The title uses 20 points. Author, page and
  percentage share one secondary line. The progress track shrinks from a
  220-point cap and three-point height to 120 by two points. Insets shrink to
  16 by eight points, and the shadow is removed.
- The reading action remains alongside the summary on wide layouts and below
  it on narrower layouts. Small-window layouts retain their existing Resume
  action and scrolling catalogue. The PDF cover and its spring physics remain
  connected to the same implementation.

The shared caption height is used by the book and gallery allocations. This
keeps the added author line inside each row without overlapping the next row.
No dependency or remote asset was added to the product.

## Verification

All 95 UI-crate tests pass, including native PDF cover, cover selection,
restoration and GPU presentation tests. Formatting and Clippy with warnings
denied pass for the changed UI crate and all its targets.

Six native captures use an isolated synthetic collection: dark English, light
Spanish, inspector open, inclined cover, reduced motion, and 640×480 with 200%
text. The capture manifest records the executable, lockfile, UI sources,
image hashes, control bounds and actual native cover readiness. The published
evidence is in [rust-taste-ui](../screenshots/rust-taste-ui/).

All 68 UI source hashes, the executable, lockfile and six image/control hashes
matched. Published PNG pixel chunks match the originals after provenance is
embedded. Cover readiness is ten in each spacious gallery, five with the
inspector and one in the small viewport, with no pending cover requests. All
persistent cache records passed header, dimension and pixel-checksum validation.
The compact capture has two disk records but one resident visible cover; disk
cache contents are not an assertion of current visibility. Continue reading is
enabled and inside its clipping rectangle in each of the five spacious views.

Final development executable SHA-256:
`C0A2D915A5DFC885A11F2D7EB2B6EB76FC5AEAE808946F93F05DFCD49D267C31`.

The bounded inspection corrected alignment in the reduced reading strip.
There is no new ambient animation or repaint loop. The inclined and reduced
motion views retain the existing moving and static book presentations.

This is a targeted visual improvement. Native captures do not certify physical
DPI, Narrator, multiple monitors, the full rewrite or release readiness. The
small-window catalogue scrolls; all books and actions do not fit simultaneously.

## Reproduce

```powershell
cargo fmt --all --check
cargo test -p yamantaka --locked
cargo clippy -p yamantaka --all-targets --locked -- -D warnings
cargo build -p yamantaka --bin yamantaka-rust --locked
./tool/capture_rust_ui.ps1 `
  -Executable target/debug/yamantaka-rust.exe `
  -OutputDirectory build/rust-visual/taste-ui-final `
  -CaseNames @('pdf-covers-dark','pdf-covers-light-es','pdf-covers-details',
              'pdf-covers-tilted','pdf-covers-reduced','pdf-covers-small-200')
```
