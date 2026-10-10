# Connected native reading restoration

The journal now offers Restore reading for supported saved context. Open page remains available independently, including when a future or invalid context requires review. The selected context is decoded once per resident page/selection, and the cache is discarded on refresh or departure.

Canonical lookup transfers page and context together under the exact request token. An accepted open arms the plan for the exact document and preferences-load token. Restoring the currently active document uses its loaded preferences, or waits for their pending load. The plan applies only once after preferences are available. Another document open, leaving for a different destination, or explicit navigation clears pending restoration. Entering the reader after the accepted open preserves it.

The preference adapter supplies custom zoom, layout, mode, paper theme and vertical scroll without changing unrelated document choices. The typed panel restores to its native tab, or closes when the snapshot has no panel. The viewport consumes the vertical target once after rebuilding its geometry; egui applies its normal extent clamp. Visible-page feedback remains canonical after that clamp. Restoration does not rewrite the archived context object.

Opening now submits to the actor before clearing the current reader. Failed queue admission retains the existing state and reports an error. Restoration lookup submission failure also releases its pending intent. Native preference replies require the current request identity as well as the active document.

## Verification

Three additional presentation regressions verify exact preference-request/document matching, single consumption, clearing/cancellation, rejecting an unrelated lookup token, transferring context once, and replacing a context lookup with ordinary page navigation. All 358 locked workspace tests pass; final formatting and warning-denied workspace/all-target Clippy pass. Logs are `build/verification/reading-restoration-ui-tests.log` and `build/verification/reading-restoration-workspace-tests.log`.

Six native DX12 captures cover dark/light English/Spanish journal actions, compact 200% text, an invalid context and actual restoration in both themes/locales. The restoration captures confirm page index 1, zoom 1.75, continuous document mode, sepia environment, vertical offset 1500, closed panel and no pending plan/scroll. The preview action invokes the same restore command as the visible control; it does not simulate physical input.

The first fixture attempted zoom 1.25 and scroll 1050 on a mixed portrait/landscape two-page PDF. Native geometry legitimately clamped the offset to 970, leaving the first page at the viewport top; the strict saved-page capture criterion timed out. An ignored diagnostic snapshot established the cause. The corrected fixture uses zoom 1.75 and a coherent offset 1500 on the second page. Temporary diagnostic instrumentation was removed before final compilation. The clamp behavior was retained.

Final evidence is `build/rust-visual/restoration-final/capture-manifest.json`. Executable SHA-256: `5D57B8193E7FDC8377F22053D21FCE20D55366AF085B3E3F5C1C3A9F6247C657`. The executable, lockfile, all UI source hashes and six PNG/control artifact pairs were independently checked. All final capture processes exited successfully. Generated evidence stays outside the public commit.

## Remaining acceptance

Physical pointer/keyboard activation, restoration into an already active document, rapid open/leave races, restoration under password challenges and failed native opening, each open-panel tab, viewport-size changes/clamping, queue-admission interaction and cross-layout acceptance remain pending. Individual history deletion, category cleanup, recents/closed documents, startup/crash recovery, goals, workspaces and complete Release verification remain open.
