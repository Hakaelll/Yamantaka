# Connected native page-text panel

Date: 2026-10-09

## Connected behavior

The application coordinator now accepts `Action::TextGeometry` and returns typed success/failure updates with document, requested page and cancellation identity. It waits on ticket readiness rather than blocking native work, rechecks cancellation after completion and rejects a payload from another page. Immediate queue-admission failure produces an identified terminal update. Shutdown cancels and detaches pending tickets.

The reader's Text tab now requests this governed result and retains its Arc instead of copying the extracted page into an ungoverned panel String. Page/document changes cancel previous requests and release results. Closing the panel, leaving Text or leaving the reader clears its state. Requests are keyed by document/page, so zoom alone does not restart extraction. Late, foreign and duplicate replies do not replace current text.

The panel exposes extraction progress, failure with explicit retry, localized no-extractable-text feedback and Copy text for nonempty content. Copy uses the existing egui platform clipboard command; no direct PDFium or SQL call occurs in UI. At compact width, title and copy action share a row to keep the action visible at 200% text zoom.

## Tests

Three application tests cover five ticket completion paths (success, late cancellation, wrong page, native failure and disconnection), identified immediate admission failure, coordinator responsiveness to a real SQLite catalogue query while the native ticket is held, Unicode text/rectangle preservation and shutdown without waiting on native completion. Each path produces one terminal result and releases tracked Text memory.

Three UI-controller tests cover request coalescing, page/document changes, cancelled/foreign/duplicate replies, payload page validation, explicit retry state and cancellation/resource release on clear/drop. These are lifecycle tests, not OS pointer or clipboard tests.

All 220 workspace tests, formatting checks and warning-denied workspace/all-target Clippy pass. Local logs are `build/verification/text-geometry-actor-tests.log` and `build/verification/text-geometry-actor-clippy.log`.

## Native presentation evidence

Four final native wgpu captures are in `build/rust-visual/text-native`. Manifest SHA-256: `C7084C7FE6CD9E26F39B8D957E446D7B693FFD1A76AE6825D1B8B06B0BF73E70`. Verification checked the executable, Cargo.lock, UI source records and every PNG/control hash.

| Capture | Verified result |
| --- | --- |
| reader-text | Dark panel; 47 native characters; Copy text inside its clip |
| reader-text-light-es | Light Spanish panel; 47 native characters; Copiar texto inside its clip |
| reader-text-small-200 | 640 x 480 at 200% text zoom; copy action fully visible in compact header |
| reader-text-empty-es | Synthetic scanned page; zero native characters; Spanish empty state and no copy action |

All captures use isolated demo stores and synthetic PDFs. Initial views of dark, compact and empty states found the compact copy action outside its initial clip. One fix batch moved it beside the title. Three final views confirmed compact, light Spanish and empty presentation. The visual review is finished for this change.

Tracked Text bytes at capture are 3,276 for dark/light, 2,540 for compact and 168 for empty. These include other resident reader text payloads and are snapshots, not general process-memory/performance measurements. The source manifest covers UI files; actor/native changes are covered by their tests and Git checkpoint. No clipboard action, OS keyboard, physical DPI or Narrator was exercised by the capture script.

## Remaining acceptance

Pointer/keyboard text-range selection over the PDF, multi-page selection, selection presentation, annotation creation/editing/review/export and the page-note fallback still need connecting and verifying. Native/OS clipboard execution, long-page text-panel performance, broader rapid-interaction stress, accessibility, physical DPI and full release acceptance remain pending. Page text availability does not establish complete annotation parity.
