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
