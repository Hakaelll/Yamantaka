# Native PDF source coherence

The PDF worker previously checked source revisions when opening and preparing covers, but reader operations and cache hits could continue using an older loaded document after its file changed. The synthetic baseline accepted stale raster and text cache entries, outline, text geometry, links and search. It also accepted cached text after replacing the path with another physical file.

The worker now acquires a short Windows read lease before those operations, checks the loaded file's physical version and verifies the lease again before returning an owned result. This includes source-revision queries, chunked page inspection, raster/tile caches, text caches and the other native read paths. Cover detection and rendering use the same source guard. Cancellation, epochs, independent render consumers and the two-document admission policy remain in place.

Opening passes the exact file used to capture the source stamp into `load_pdf_from_reader`. A later path lookup cannot substitute another file for that native reader. Temporary cover opens also bind their reader to the expected stamp. The read lease lives only through the operation; the PDF document does not keep a deny-write lease for its entire lifetime. Cache-hit validation reads physical-version metadata and does not hash the whole PDF or allocate a PDF-sized copy.

## Executed evidence

The [result artifact](artifacts/pdf-source-coherence.json) binds the baseline and corrected probes to their executable hashes and records the runtime source hashes. The probe generates its own valid PDF with text after the first 64 KiB. Replacement changes that text while preserving file length, the entire first 64 KiB and the original modification time. All six routes accepted results before the correction. All six now reject them specifically with `PdfSourceChanged`.

The direct worker probe reopens the same document ID, reads the replacement text and produces different pixels. The application actor requires closing a visible reading before reopening its native owner; [reader source ownership](rust-reader-source-ownership-2026-11-03.md) records that admission check. Previously delivered pixels retain their digest. Renaming the open file and replacing its path is allowed between jobs, and the old cached text is then rejected. A separate lease check blocks write access while held and permits it after release. These checks use only generated temporary PDFs.

All nine retained native PDFium tests passed, covering covers, two hot documents, cancellation and shutdown, protected/damaged/scanned/vector files, mixed and rotated pages, stitched tiles, live buffer ownership and 5,000-page inspection. One existing assertion was updated to the typed source-change failure; no standalone Rust tests were added.

Three serial application captures passed: the wgpu reader, the OpenGL electronic-ink reader and a settled gallery retaining ten PDF covers. Both reader images match the preceding control checkpoint pixel for pixel using RGBA byte comparison. All 42 generated fixture PDF hashes remain unchanged. Locked offline builds, workspace formatting and workspace all-target Clippy with warnings denied passed. Raw captures and manifests stay local under `build/verification/source-coherence`.

## Reproduction and limits

```powershell
cargo run --locked --offline -p yamantaka-pdf --example source_revision_probe -- `
  build/native/pdfium-7881/bin/pdfium.dll build/verification/pdf-source-review.json
```

Choose a fresh `.json` output path; the probe refuses to overwrite an existing file. The earlier baseline used a frozen pre-correction executable with `--observe`; that mode records outcomes and does not require rejection. The corrected run requires the typed failures, new-content reopening, retained old pixels and lease release.

Physical source versions are metadata-based change hints, not complete content digests. This does not certify every filesystem, forged metadata, every concurrent edit or GUI recovery race, or Release interaction latency. Full-content SHA-256 requirements for duplicate confirmation and indexing remain separate. Existing page images remain available after a source failure; reopen the document from the library to read its new version. The broader acceptance matrix remains Partial.
