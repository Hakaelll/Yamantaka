# Automatic cover selection in the native library

The library actor now requests native detection when a document has no stored
cover decision. It persists the selected source page, confidence and algorithm
version, then sends the image through the existing scoped texture flow. Saved
choices bypass detection, including manual decisions copied from Flutter.

The first six pages are scored using the
[native policy](rust-cover-detection-2026-10-13.md). The detector writes the final
selected image to the [bounded pixel cache](rust-cover-cache-2026-10-13.md) when
the source identity remains stable and the PDF is a temporary unprotected open.
Existing native handles bypass disk caching. Restarting the actor uses the saved
page and the normal cover cache path without running detection again.

## Persistence and competing choices

Automatic completion validates the document/page/generation boundary, finite
score and confidence, complete unrotated raster, bounded dimensions and pixel
length. The application checks the expected generation before constructing it.
The aspect stored in the legacy schema is clamped to its supported range.

SQLite verifies the current catalogue path, fingerprint and page count. It
inserts only when no cover row exists, clears recreatable document thumbnail
references and reads back the full resulting metadata before committing.
Progress and all existing cover choices remain unchanged on a rejected write.
Ignored, aborted or mutated writes roll back together.

If a user saves a manual choice while detection is pending, the automatic write
returns without replacing that row. The actor reads the winning source page and
renders it with the original request scope. If it is already the detected page,
the image can be reused while retaining all winning metadata. Cancellation before
completion prevents persistence. An accepted atomic write retains its real
outcome; later presentation cancellation is handled by the existing UI scope.

## Executed verification

- 437 workspace tests passed, including all 95 UI-crate tests.
- One new domain test rejects mismatched pages/documents, malformed dimensions,
  nonfinite scores/confidence and inconsistent confidence.
- Three storage regressions verify exact automatic metadata, unchanged progress,
  refusal to overwrite either manual or automatic decisions, catalogue identity
  changes and ignored/aborted/mutated transaction scenarios.
- Three actor regressions exercise real PDFium and file-backed SQLite. They check
  automatic persistence, restart through the saved-page/cache route without
  another detection request, exact pixels, unchanged source PDF and released
  memory. A controlled delayed native reply proves that a newer manual choice
  wins and its page is rendered. Cancellation and rejected storage preserve the
  original selection state and thumbnail reference.
- Workspace/all-target Clippy with locked dependencies and warnings denied and
  `cargo fmt --all --check` passed.
- Three native synthetic captures cover dark automatic selection, Spanish
  light/reduced motion and an existing manual override. Each installs five
  visible covers, has zero pending cover replies and writes five valid pixel
  cache records. The generated seven-page fixture selects page two automatically;
  the manual case keeps its seeded page-one decision and version-4 metadata.
- Independent verification reads the actual isolated SQLite files, verifies their
  modes, versions, confidence and source pages, checks the cache records and
  matches all 67 UI source hashes, executable, Cargo.lock, PDF fixture and capture
  hashes. Published PNG pixel chunks match the originals. No personal library,
  source PDF or cache binary was published.

Executable SHA-256:
`6292946632B7919088A4A1BD30665CE689F3C9DCBE20E20A322940852DA6BA24`.

See the [capture manifest](../screenshots/rust-automatic-cover/capture-manifest.json),
[automatic selection](../screenshots/rust-automatic-cover/automatic-cover-dark.png)
and [preserved manual choice](../screenshots/rust-automatic-cover/automatic-cover-manual.png).
The first book's artwork is the fixture's actual second PDF page. The original
first page contains dense synthetic contents text.

```powershell
python tool/verify_rust_automatic_cover.py build/rust-visual/automatic-cover build/verification/yamantaka-automatic-cover.exe --published docs/screenshots/rust-automatic-cover
```

## Remaining acceptance

Existing saved automatic decisions currently remain authoritative. Source or
algorithm invalidation, an explicit Restore automatic action, retry/cleanup
controls, small/large thumbnail parity and broader protected/changed-file flows
remain pending. Catalogue snapshot checks do not replace external file-change
revalidation. Candidate resolution and native scheduling still have the limits
documented in the detector report.

The captures prove connected selection and persistence, not pointer, keyboard,
physical DPI or Narrator acceptance. Large-library and competing-reader latency,
paired Flutter/Rust performance, whole feature parity and final-release gates
remain unverified.
