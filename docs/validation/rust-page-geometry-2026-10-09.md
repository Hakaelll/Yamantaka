# Shared native page geometry

PdfInfo now contains shared, read-only PdfPages instead of a vector of sizes.
Each validated page record pairs its displayed size with the native coordinate
space for its crop and intrinsic rotation. This avoids keeping independently
indexed size and geometry vectors. The native outline parser and document
inspection use one DeviceToPage extraction function.

PdfPagesBuilder reserves the expected vector capacity and shared-object overhead
in the global CPU/Other memory ledger before allocating. It uses fallible vector
reservation, accounts for retained capacity, refuses extra records, and refuses
to publish an incomplete inspection. Failed or cancelled builders release their
lease. PdfInfo clones share the page payload rather than clone a large vector;
the lease survives native document close and releases on the final shared drop.
Other metadata strings and opaque PDFium allocations are outside this page-vector
accounting claim.

Native inspection still yields after sixteen pages. Every Rust consumer has been
migrated to the paired records: actor and index page counts, storage import,
reader layout, controls, bookmarks, password opening, native tests and controlled
engine fixtures. The old page_sizes field has been removed.

ReaderLayout can now recover the viewport's top-left PDF point from a frame,
selected page and native coordinate space. It uses the left edge, not the
horizontal-center anchor, and reverses user rotation before converting back to
PDF points. Blank page margins retain out-of-page coordinates. This supplies the
context needed for null destination parameters without assuming a zero crop
origin or treating the same fraction on different pages as the same PDF point.

Two domain tests cover sharing and lease lifetime, overfull/incomplete builders,
checked-size overflow, memory refusal, cancellation by dropping an unfinished
5,000-page builder, and invalid page dimensions. A layout test checks literal
viewport PDF points for all four user rotations and blank margins. The native
rotated-crop test now confirms inspection and outline geometry agree, verifies
shared payload identity, and retains geometry across native close until the last
clone drops. The existing real 5,000-page and actor suites also passed.

Validation on 2026-10-09:

```text
cargo test --workspace --locked                              158 passed
cargo clippy --workspace --all-targets --locked -- -D warnings passed
cargo fmt --all -- --check                                   passed
cargo build -p yamantaka --locked                            passed
```

The current debug executable was launched with --demo at the user's request.
Its Yamantaka window reported Responding and was brought to the foreground. It
uses the synthetic demo library. This is launch evidence, not a new visual or
accessibility acceptance capture. The outline tab and applying resolved
destinations in the actual reader viewport remain pending. Native geometry
inspection latency and the complete original acceptance matrix still need their
remaining gates.
