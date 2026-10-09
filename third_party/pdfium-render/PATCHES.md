# Local changes to pdfium-render 0.9.4

The crate source was extracted from the registry archive whose SHA-256 matches the original Cargo.lock checksum. UPSTREAM.json records every included upstream file hash. The source, build script, package manifest, README and original license are retained; upstream examples, tests and optional binding-generation headers are omitted. Yamantaka uses the existing generated PDFium 7881 bindings. The optional `bindings` feature is unsupported in this copy.

Local changes:

- Add `PdfActionUri::uri_with_max_bytes()` to check the native byte length before allocation. The existing `uri()` delegates with an unlimited caller limit to preserve upstream compatibility.
- Return `UriLengthExceedsLimit` when a caller limit would be exceeded.
- Return the existing `DataBufferLengthMismatch` error instead of panicking if the second native call reports a changed length.
- Omit registry package contact metadata from the local Cargo manifest. Copyright and license attribution remain in LICENSE.md and the upstream README.

- Apply narrow dead-code attributes to the unused upstream content-mark struct and implementation, preserving the warning-denied application build without changing behavior.

- Build only the Rust library required by Yamantaka, omitting unused upstream static/dynamic library outputs.

- Pin libloading to the already tested 0.9.0 version from the original workspace lockfile, avoiding dependency resolution changes from the path patch.

No PDFium ABI, handle ownership or threading behavior is changed. The normal application calls remain serialized by the native worker.

Git attributes preserve this directory byte-for-byte so upstream and local patch hashes remain valid on Windows and other checkouts.
