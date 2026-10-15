# Native cover source ownership

An open PDF now owns its requested path and a source stamp alongside its native
handle. The stamp includes the canonical path, size, modification time and
SHA-256 of the first 64 KiB. Cover rendering and automatic detection compare
the requested source with that stamp before reusing a handle.

## Behavior

A changed source under an unprotected document ID detaches the old native
handle and clears its page, tile and text caches before temporary cover work.
A protected reader keeps its handle. Native cover work for a different source
under that ID returns an error instead of rendering the reader's old file.
A validated persistent-cache hit for the requested source can still return
pixels without disturbing either protected reader.

Opening captures the stamp before native loading and verifies it afterward.
Metadata inspection verifies it again before publishing the completed result.
Cover rendering, detection and the early cache-hit path also verify the source
before delivering their result. Cancellation remains checked during bounded
header reads and before delivery.

## Regression evidence

The first three tests failed against the preceding implementation before the
worker change. All four tests in `cover_source_native` now pass with PDFium:

- Direct rendering and detection use the requested file when the same ID was
  opened for another path. The unrelated protected reader remains usable.
- Two protected readers reject native work for a changed source. A seeded,
  validated cache hit returns the expected pixels and preserves both readers.
- Appending to a source rejects reuse of a protected handle and detaches an
  unprotected handle, including its previously populated text cache.
- Changing the PDF header while preserving file size and modification time
  still detaches the old unprotected handle.

The tests compare actual raster pixels or observable text results and check
that governed memory returns to zero after dropping results and workers.
File mutation tests use temporary synthetic copies, never a personal PDF.

All 452 workspace tests passed, including 95 UI tests. Formatting, workspace
Clippy with all targets and warnings denied, and the native application build
passed. After strengthening the temporary-file test to populate the text cache
before changing the source, all four source regressions passed again.

## Reproduce

```powershell
cargo test -p yamantaka-storage --test cover_source_native --locked
cargo test --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p yamantaka --bin yamantaka-rust --locked
```

## Limits and remaining acceptance

This bounded stamp is not full-file equality. A body-only modification beyond
the first 64 KiB that preserves path, size, modification time and header hash
is outside this check. Full-file SHA-256 confirmation for exact duplicate
consolidation remains a separate requirement.

The change does not add filesystem watchers or invalidate an already resident
UI cover automatically. Reader operations other than opening and metadata
inspection retain their existing handle semantics. Physical DPI, Narrator,
paired performance traces and the remaining rewrite gates remain pending.
No UI layout or book physics changed, so this report adds no visual captures
and makes no performance or final-release claim.
