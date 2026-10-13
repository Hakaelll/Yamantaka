# Native automatic cover scoring

Rust now has the cover policy and an asynchronous native detection operation.
The library actor does not yet invoke or persist that operation. The connected
gallery still uses a saved source page or the first page when no selection is
stored. This checkpoint establishes the detector, not complete automatic-cover
behavior in the application.

## Policy and native ownership

The domain policy ports algorithm version 3 from Flutter's `CoverSelectionPolicy`.
It preserves the first-page position weights, uniform-space and edge thresholds,
contrast reward, text-length and line-count penalties, contents penalty and
confidence calculation. It keeps the first page unless another candidate has
sufficient confidence or a clear score advantage. Inputs must be finite,
ordered first-page metrics for one to six candidates.

RGBA sampling uses the same luminance coefficients, approximately 48 samples
per axis, uniform/edge thresholds and variance calculation as Flutter's
`ThumbnailService._measure`. Text length uses UTF-16 units, matching Dart's
length convention. Contents markers use whole Unicode words without allocating
a lowercase copy. This deliberately handles accented uppercase Spanish words
more consistently than the legacy regular expression's ASCII word boundaries.

The native ticket uses the worker's existing bounded response reservation and
background priority. PDFium stays on its owning thread. Detection opens a
temporary document when needed, evaluates at most six pages, releases each
governed preview and extracted text before the next, and renders the selected
page for the returned `DetectedCover`. Cancellation and memory failures remain
explicit failures. Text extraction errors can fall back to visual scoring for
scanned pages. No source PDF or stored cover metadata is modified.

Candidate images currently use the bounded 512-pixel native cover renderer.
Flutter uses scale 0.16 for candidates, so these image measurements are not
pixel-identical to Flutter's raster inputs. The detector reuses the governed
text-geometry reader, including its character work limit. It processes the six
candidates in one native scheduling unit; finer interleaving and scoring-only
text extraction remain part of performance acceptance.

## Executed evidence

Six domain regressions check literal weights/confidence, threshold endpoints,
first-page preference and clear improvements, pixel metrics, UTF-16 lengths,
Unicode contents markers, invalid inputs and cancellation.

The final workspace run passed 430 tests. Workspace Clippy with all targets,
locked dependencies and warnings denied passed. `cargo fmt --all --check` passed.

Three real PDFium integration tests use a deterministic seven-page synthetic
fixture. They check selection of its second page, exact equality with directly
rendered pixels, request identity, original PDF bytes, cancellation, memory
denial and full release of governed memory after dropping results and workers.
The admission regression retains 128 unread detection tickets, verifies rejection
of another data request, closes a document through the separate control capacity,
then consumes the tickets and verifies new data admission.
Only the fixture generator is public; the generated PDF stays under `build`.

Synthetic PDF SHA-256:
`996C82E1DEB83AFD8C13DE7834FAE7603E4F9FE23109DC08E4A5B781C1047609`.

Reproduce the focused checks with:

```powershell
python tool/create_rust_auto_cover_fixture.py
cargo test -p yamantaka-domain cover_policy --locked
cargo test -p yamantaka-storage --test cover_detection_native --locked
```

## Remaining integration

The library actor must request detection when no usable selection exists,
persist confidence and version with catalogue identity checks, and refuse to
overwrite a newer manual choice. The selected result must then feed the same
scoped library texture flow and persistent cache. Automatic restoration controls,
algorithm/source invalidation and native UI captures remain pending.

The operation retains the existing two-handle admission gate. Broad scanned,
protected, multilingual and malformed candidate comparisons, responsiveness
under competing reader jobs, large-library performance and whole-rewrite
acceptance remain unverified. No speedup or completed automatic-cover parity
is claimed.
