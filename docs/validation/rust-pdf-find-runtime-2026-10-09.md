# Native PDF page-search foundation

The native worker can search an open PDF page asynchronously and return bounded
match batches. This checkpoint provides the domain, PDFium and application actor
contracts. The reader search panel, document-wide scan coordinator, highlights,
and next/previous shortcuts are not connected yet. In-document search remains
Partial in the acceptance matrix.

## Contracts and ownership

Queries preserve Unicode, with independent case-sensitive and whole-word options.
They reject blank input, NUL and input above 1,024 UTF-8 bytes. Results contain at
most 80 matches, an ordinal continuation cursor, bounded snippets, and normalized
rectangles in the canonical crop/rotation coordinate space. Nonvisual matches do
not invent a navigation anchor. PDFium text traversal order can change with
intrinsic rotation; match ordinals belong to that page's native traversal.

Search executes on the existing serialized worker and preserves cancellation
identity and document epochs. The actor accepts unresolved search tickets without
blocking catalogue commands. Cancelled results are rejected even if the native
ticket later succeeds. Native handles remain on the worker.

Each request reserves 2 MiB of CPU Text working capacity before extracting text.
Accepted results retain a shared lease reduced to their owned payload; final drop
releases it. Requests reject pages above 128,000 characters, searches above 10,000
match visits, and matches above 64 rectangles. These limits report failures instead
of presenting incomplete work as complete. Opaque SDK allocations and total
process memory are outside this payload-accounting claim.

## Verification

Two domain tests cover query validation, literal normalized geometry, anchors,
continuation validation and shared lease release. Three real PDFium tests cover
80-result pagination, case and whole-word options, Unicode, empty results, invalid
pages and cursors, cancellation, memory rejection and recovery, source hash
preservation, and result ownership after document closure. Rotation checks compare
the complete native-space match geometry across all four intrinsic rotations,
rather than assuming the first textual match has the same traversal order.

One SQLite-backed actor integration test verifies that a catalogue query completes
while a native search ticket is held. It checks result/cancellation identity and
rejection of a late success after cancellation. Native limit stress cases and
interactive reader acceptance remain pending.

```text
cargo test --workspace --locked                               172 passed
cargo clippy --workspace --all-targets --locked -- -D warnings passed
cargo fmt --all --check                                       passed
```

The suite and lint logs are retained locally under ignored `build/verification`.
All search PDFs are generated synthetic fixtures. No personal documents or search
queries are included in this checkpoint.

## Independent development preview

`--demo --demo-window` opens a separate synthetic demonstration instance using a
process-specific preview store. It allows reviewing a new executable while an
older demonstration remains open. The current development executable was launched
successfully and its native Yamantaka window reported responsive. This is a launch
check, not visual, accessibility, or complete application certification.
