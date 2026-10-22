# Two-reader admission checkpoint

Opening confirmation now accepts the complete visible document set atomically. It retains at most two unique documents, requires the newly opened document, and rejects references to documents outside the previous visible set. Cancellation restores the previous set; stale confirmations remain harmless. The single-reader UI keeps its existing behavior.

The existing native import-preemption scenario now admits its second reader with both document IDs in the confirmation, without a later visibility repair. The focused run passed: `cargo test -p yamantaka-application --test import_resumption_runtime native_reader_preemption_resumes_the_interrupted_page_and_keeps_prior_evidence -- --exact`. It exercises two protected readers while interrupted PDF analysis waits and subsequently resumes.

This checkpoint supplies the ownership boundary for native split reading. A second viewport, pane-specific interactions, synchronization and saved split restoration remain pending; no split UI or complete parity is claimed.

## Independent rendering state

`ReaderSurface` now owns each viewport's page images, layout, scale, render generation, requests and scroll restoration flag. Exact render-key and cancellation-token matching is checked by the owning surface before installation. Resetting requests retains existing images; clearing or dropping a surface cancels its consumers before releasing images. The shared presenter and memory governor remain application services.

The native binary compiled. One existing synthetic scenario, `reader-preserved`, completed after the extraction: a replacement failed, the earlier reader remained on page 2 at its previous zoom, and its native page image stayed visible. The PNG and control record are local in `build/rust-visual/reader-surface`. No broad test suite was run. This is the rendering-state extraction, not an enabled second reader.
