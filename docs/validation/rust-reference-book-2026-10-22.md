# Reference document checkpoint

The later [protected reference checkpoint](rust-reference-password-2026-10-22.md) adds the shared password dialog, retry and cancellation routing. The pending list below records this earlier checkpoint.

Open **Reading options → Compare pages**, then use the book button in the reference header to choose a document from the current library page. The main reading stays active while the reference opens. Its page, fit and zoom remain independent. The reference header displays the chosen title. The same reference can optionally follow the primary page.

Opening, confirmation and closure have separate request-scoped states. Admission confirms the complete two-document protection set through the shared PDF actor. Opening failure retains the previous reference; cancellation retains ownership until its matching completion. Closing a different reference waits for `ReaderClosed` before dropping it. A failed close keeps its view and permits retry. Leaving the reader waits for this closure. Closing the same-document reference releases only its view, preserving the shared native handle.

Close the reference before changing the main reading or choosing another reference document. This explicit step preserves the two-hot-document limit. The runtime, renderer, GPU presenter and memory governor remain shared.

The native binary compiled. Two focused synthetic scenarios passed: `reader-reference-book` opens catalogued `demo-1` beside preserved `demo-0` and verifies a native reference image; `reader-reference-closed` verifies one acknowledged native reference close, no remaining reference and the primary page unchanged. See the [open reference](../screenshots/rust-reference-book/reader-reference-book.png) and [recovered primary reader](../screenshots/rust-reference-book/reader-reference-closed.png). These fixtures prove distinct native document identities, not independent real-world PDF-content coverage. No broad suite or additional standalone tests were run.

Secondary password prompts, annotation and text tools, chooser search/pagination beyond the current library page, saved split restoration, bidirectional synchronization, physical interaction acceptance and failure-injection UI scenarios remain pending. This checkpoint does not establish full split-reader parity or Release readiness.
