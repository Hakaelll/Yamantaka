# Shared PDF destination and geometry contracts

Destination interpretation and normalized page rectangles now have canonical
modules that can serve contents, search, links and future annotation tools.
This checkpoint prepares internal-link integration; it does not provide clickable
PDF links yet.

## Changes and compatibility

`pdf_destination::NavigationDestination` replaces the outline-specific destination
name. Destination view, native point coordinates and content bounds retain their
existing values and validation. `pdf_geometry::PageCoordinateSpace` owns crop and
intrinsic-rotation conversion. `NormalizedPageRect` replaces the search-specific
rectangle name, retaining finite, positive-area normalized bounds.

All Rust callers and tests were migrated, and the former public names were
removed. There are no compatibility aliases or duplicate destination models.
These values are transient native/navigation contracts; persisted database and
preference formats did not change.

The PDF crate's common destination reader now owns the existing PDFium conversion,
all destination view modes, cancellation checks, crop geometry and bounded content
object traversal. The contents reader calls it rather than retaining a private
copy. Native document and destination handles remain on the serialized worker.

Canonical geometry now converts native PDF rectangles, validates malformed or
non-finite bounds, clips to the page crop, and rejects wholly outside or zero-area
regions. Its rotation method transforms all corners. Native search extraction and
UI highlight presentation use those shared operations. This removes their separate
rectangle-transform loops and provides the same rules for upcoming link hit areas.

## Executed validation

A new geometry test covers literal crop conversion, clipped and wholly outside
regions, degenerate and malformed rectangles, non-finite input and quarter-turn
bounds. Existing domain coordinate and viewport tests, all destination-resolution
tests, real PDFium outline/search cases, actor ownership tests and UI highlight
coordinate tests remain passing after migration.

```text
cargo test --workspace --locked                               186 passed
cargo clippy --workspace --all-targets --locked -- -D warnings passed
cargo fmt --all --check                                       passed
```

Logs remain under ignored `build/verification/pdf-shared-contracts-*`. This is
contract and behavior validation, not a new screenshot, physical accessibility or
paired performance claim. Internal-link extraction, clickable and keyboard link
presentation, external-link policy parity, and all other outstanding rewrite and
Windows Release gates remain pending.
