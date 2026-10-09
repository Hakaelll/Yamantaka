# Bounded native PDF URI reading

Date: 2026-10-09

## Problem and implementation

The pinned pdfium-render 0.9.4 URI getter queries PDFium for a buffer length and allocates that length without a caller limit. Its native handles are private, so an application-side length check after `uri()` would occur after the allocation. Yamantaka now carries a narrowly patched local copy and exposes `PdfActionUri::uri_with_max_bytes()`.

The bounded getter rejects a native length greater than the payload limit plus its terminating NUL before creating the buffer. The existing getter retains compatibility by delegating with an unlimited caller limit. A changed second-call length returns a typed buffer mismatch instead of panicking. No ABI or handle-layout assumptions are introduced.

The local source comes from the registry archive whose SHA-256 matched the original Cargo.lock checksum: `8948a803616a9e936b15a6637af2cd48c5fb8ae0fcdeb3c32eac3a540e255a19`. `third_party/pdfium-render/UPSTREAM.json` records 144 upstream file hashes and four explicitly patched files. `python tool/verify_pdfium_vendor.py` verifies all included source files against those records and rejects unrecorded sources.

The local manifest retains the tested libloading 0.9.0 version, builds only the Rust library needed by the application and excludes the unused optional binding-generation headers. The upstream content-mark struct has narrow dead-code attributes so application checks stay warning-free. The vendored crate is excluded from workspace formatting and test membership; it remains a normal compiled dependency.

Original README, copyright comments and license metadata are retained. The complete Apache 2.0 text was obtained from [the Apache Software Foundation](https://www.apache.org/licenses/LICENSE-2.0.txt), with SHA-256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`. No global registry cache was modified.

## Executed verification

A real PDFium test reads a URI from a synthetic link annotation. Zero and one-byte-too-small limits return `UriLengthExceedsLimit`; exact, larger and unlimited limits return the original URI. The previous API returns the same value and the source PDF SHA-256 remains unchanged. Existing native link, destination, geometry, search, renderer and worker tests run against the patched dependency.

All 199 workspace tests, warning-denied Clippy and formatting checks passed. The provenance verifier passed for 144 included upstream files and four recorded local patches. Local logs are `build/verification/pdfium-bounded-uri-tests.log` and `build/verification/pdfium-bounded-uri-clippy.log`.

The public-checkpoint privacy guard was also run over every vendored file and reported zero findings. Test PDFs and native binaries remain outside committed artifacts.

## Remaining work

The production link extractor currently returns internal targets and counts external targets as unsupported. Its next step is to call the bounded URI getter, apply the same allowed http/https/mailto policy as Flutter and connect an explicit confirmation dialog to Windows external opening. This dependency checkpoint does not claim external-link parity or complete rewrite/release acceptance.
