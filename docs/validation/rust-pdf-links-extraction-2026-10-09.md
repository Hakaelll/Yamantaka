# Native internal PDF link extraction

Date: 2026-10-09

## Implemented behavior

The PDF worker now exposes asynchronous, independently cancellable page-link extraction. It returns owned internal destinations and normalized active regions; PDFium handles remain on its worker thread. Direct destinations and local GoTo actions use the same destination interpreter as the outline.

Multiple annotation quadrilaterals remain separate regions. Each region passes through the canonical crop and intrinsic-rotation coordinate conversion. Regions outside the visible crop and zero-area regions are omitted. Non-finite quadrilateral coordinates fail extraction.

Work is bounded to 10,000 annotations, 512 link annotations and 64 regions per link. Fit-bounds destinations share the existing 100,000-object inspection allowance across the request. Extraction reserves 2 MiB from the global Text budget before loading page geometry or annotations. Retained payloads charge vector capacities and release their lease after the final owner drops. Opaque PDFium allocations remain outside the application's accounting.

The worker checks cancellation and document epoch throughout extraction and again before publication. Unsupported targets, including URI actions, are counted without opening them or retrieving their URI strings. No original PDF is modified.

## Executed verification

Three native PDFium tests cover direct XYZ and local GoTo FitH destinations, separate quadrilateral regions, crop clipping, all four intrinsic rotations, empty pages, unsupported target counts, original SHA-256 preservation, retained payload lifetime, cancellation, invalid page numbers, memory admission failure and annotation/link work limits.

One domain test rejects empty or excessive regions, duplicate or descending annotation indices and excessive payload entries. It also checks shared ownership and memory release.

Workspace tests, warning-denied Clippy and formatting checks passed. Local logs are under `build/verification/pdf-links-tests.log` and `build/verification/pdf-links-clippy.log`. Fixtures are synthetic and created in the temporary directory.

## Application actor routing

PageLinks commands now travel through the existing bounded actor queue and asynchronous pending-ticket selection. Success events carry the document ID, owned page payload and original cancellation token. Failure events also retain the requested page number. The actor checks cancellation again after a native ticket completes and rejects a payload for a different page before emitting success.

Two application tests exercise five completion paths with a deliberately held native ticket: success, late cancellation, wrong-page payload, native error and disconnected response. A real in-memory SQLite library query completes while the native ticket is held. Each completion produces one terminal event and releases payload memory after consumption. A separate shutdown case proves dropping the actor cancels and detaches a held ticket without waiting for its native response.

## Visible reader controller

The reader requests links for every intersecting page, with at most two link requests pending concurrently, and retains only intersecting pages. Completed requests free admission for the next visible page; there is no 16-page coverage cutoff. Leaving a page, changing the document, leaving the reader or shutting down cancels pending consumers and drops retained payloads. Current document, page and cancellation identity must match before the controller accepts a result. Failures expose an explicit retry control rather than repeatedly submitting work each frame.

Internal regions are interactive native egui links with destination-page labels, pointer feedback, keyboard focus and a focus outline. Separate regions retain their gaps. The existing destination resolver is now a reader module shared by outline selection and link activation. Search highlights and link regions share the same normalized-to-paper transform, including user rotation.

Five UI state tests cover stale/cancelled replies, page removal, payload release, successful and failed completion, document changes, cancellation on drop and eventual coverage of 40 visible pages under the two-request admission limit. One additional input test uses actual egui pointer press/release and Enter/Space events against the production link control, including a click between disjoint regions that must not activate the link. Existing search rotation tests exercise the shared presentation transform. Native pointer/keyboard captures and full clickable-reader acceptance have not yet been executed for this checkpoint.

## Remaining acceptance

Internal reader controls are implemented but still require native pointer and keyboard acceptance captures. The admission controller now covers all visible pages, but very small continuous-zoom behavior and latency still require native acceptance. External URI policy and confirmation, navigation-history interaction evidence and broader performance/accessibility acceptance remain pending. No release or full rewrite completion is claimed.
