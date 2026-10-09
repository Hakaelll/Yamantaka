# Native PDF outline extraction

The PDF worker can now extract a document outline through a nonblocking ticket.
The application actor exposes `Outline` and returns a document ID, immutable
shared outline, and request token. `OutlineFailed` preserves that identity on
failure or cancellation. The reader panel does not consume these results yet.
This checkpoint proves the native and actor path, not completed outline UI.

Entries preserve Unicode titles, depth-first order, and parent indices. Local
destinations retain their page and PDF view settings, including optional point
coordinates, zoom, width fitting, and rectangle fitting. External actions and
headings without a local destination have no navigation target. Extraction does
not execute actions or write the source PDF.

The worker checks cancellation and document epoch before extraction, between
entries, and before response delivery. Closing or reopening a document makes an
older request obsolete. Outline tickets use the existing bounded admission and
readiness channels. The actor continues to serve catalogue queries while a
native outline reply is pending.

Extraction reserves 4 MiB in the shared text budget before traversal. The result
shrinks that reservation to the retained vector, string capacities, and shared
object metadata. The reservation remains alive until the final shared owner
drops. Entries are read-only after delivery. Limits reject more than 10,000
entries, a title above 4096 UTF-8 bytes, or aggregate title capacity above 1 MiB.
Exceeding a limit returns a failure rather than a partial outline.

Two tests use real PDFium and synthetic PDFs generated during the test. They
verify exact Unicode titles, parent relationships, coordinate and zoom values,
width and rectangle destinations, external-action suppression, and an unchanged
source SHA-256. They also cover an empty outline, a cyclic graph, invalid
destinations, excessive titles, 10,001 entries, cancellation, a closed document,
memory-budget refusal, and recovery after the budget increases. Shared results
remain valid after document close and release their reservation on the last drop.

An actor test holds an outline reply in flight, queries the catalogue, and then
checks the returned chapter, page, and request identity. It also checks a typed
cancelled response and release of the shared result reservation.

Validation on 2026-10-09:

```text
cargo test --workspace --locked                          144 passed
cargo clippy --workspace --all-targets --locked -- -D warnings  passed
cargo fmt --all -- --check                               passed
```

The new parser has its own PDF module. The worker retains scheduling and native
handle ownership; the domain owns the immutable returned data and its lease.
This follows the requested Boundary Discipline principle and the strict quality
review's requirement to keep feature parsing out of an existing large worker.

Remaining work includes the outline tab, destination application through canonical
navigation, broad malformed-input stress, and interaction latency during large
outline traversal. The reservation covers copied results and conservative Rust
working space. Temporary allocations inside PDFium and its title conversion are
not a measured global-allocation guarantee. No new UI captures or release claim
are part of this checkpoint.

## Destination coordinate geometry follow-up

Each local destination now carries a typed page coordinate space. The PDF worker
asks PDFium to convert three device corners to PDF points. This preserves the
visible crop origin and intrinsic page rotation without reproducing PDFium's
coordinate rules in the UI. The conversion config uses a 10,000 by 10,000 device
rectangle and allocates no bitmap. Domain functions convert between PDF points
and normalized displayed-page coordinates. Values outside the crop stay outside
instead of being silently clamped. Non-finite and singular bases are rejected.

A third real PDFium test creates a non-square crop box with a nonzero origin and
checks the same PDF destination under rotations of 0, 90, 180 and 270 degrees.
The expected normalized coordinates are literal values, with a tolerance of
0.000001. A domain test covers inverse conversion, out-of-crop points, and invalid
bases. The existing actor fixture now carries geometry through the same reply.

The complete workspace passed 146 tests. Clippy with all targets and warnings
as errors passed, and the formatter check passed. The original 144-test result
above describes the preceding checkpoint.

Reader integration remains pending. These conversions do not yet apply user
rotation, destination zoom or fit settings, or horizontal anchors to navigation.
Page loading during outline traversal also needs interactive latency validation
for large outlines with destinations. No interface or release completion is
claimed by this follow-up.
