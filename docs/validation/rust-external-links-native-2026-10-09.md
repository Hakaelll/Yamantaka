# Native external-link confirmation and rejected-target feedback

Date: 2026-10-09

## Implemented behavior

Rejected URI actions now retain their normalized active regions with a typed reason rather than disappearing from the reader. Activating them displays an ES/EN explanation for an invalid or disallowed destination, or a destination that exceeds the size limit. Rejected targets contain no URI string and cannot reach the Windows opening adapter. Permitted destinations still require the explicit confirmation dialog.

The confirmation now keeps its heading and action footer outside the scrollable destination text. This fixes clipped Cancel/Open controls in the native 640 x 480, 200% interface-text capture. Initial keyboard focus remains on Cancel.

A real PDFium regression verifies aggregate URI capacity exceeds the 512 KiB work limit deterministically, releases partial result memory and leaves the worker able to answer the next page request. Existing rejected URI cases now assert their regions remain present. Original-file immutability and URI validation continue to run against the patched native dependency.

## Native captures and provenance

Four final captures are under `build/rust-visual/external-links-native`, with an exact executable/source/Cargo.lock/image/control manifest. Verified manifest SHA-256: `FA1BD2D07FDA9877FD7C3F5971040BC273387D1D22CCCEC130E149A99607828A`.

| Capture | Observed terminal state |
| --- | --- |
| external-link-dark | Allowed external region activated; confirmation remains open with Cancel focused |
| external-link-light-es | Spanish light confirmation with normalized synthetic destination |
| external-link-small-200 | Compact confirmation; both enabled action controls fit completely within their clips |
| external-link-rejected-es | One rejected local-file region; Spanish explanation visible and no confirmation open |

Each capture uses an isolated demo store, synthetic PDF and native wgpu presentation. The production region control is activated by the capture-only egui Enter event. No OS keyboard, pointer or external browser/mail handler was exercised. Capture mode never dispatches to Windows external opening. URI text is absent from control records; the only visible URI in the allowed captures is the synthetic example.invalid destination.

Allowed captures retain 741 tracked Text bytes and the rejected capture 712 at their respective capture points. These are synthetic fixture measurements, not general memory/performance acceptance. Hash verification checked all source records, the executable, Cargo.lock and every PNG/control record. All expected enabled link/action controls lie within their clips.

Three initial views identified the compact action clipping. One fix batch moved the actions outside the scroll area. Three final confirmation views inspected light Spanish, compact and rejected states; the visual review is finished for this change.

All 205 workspace tests, warning-denied Clippy and formatting checks passed. Local logs are `build/verification/external-links-native-tests.log` and `build/verification/external-links-native-clippy.log`.

## Remaining acceptance

Actual Windows browser/mail-handler opening, failure/retry, long-destination scrolling, repeated cancel/Escape/open sequences, OS pointer behavior, rapid document changes during confirmation and broader accessibility/performance acceptance remain pending. These captures do not certify physical DPI, Narrator, multiple monitors or the full rewrite/release.
