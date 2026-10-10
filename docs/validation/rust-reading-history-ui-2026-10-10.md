# Native reading journal

Tools now opens the Reading journal destination. It queries completed sessions through the bounded actor and keeps one immutable, governed page of up to 80 entries. Refresh, previous/next pages, selection and session details are connected. Source opening uses the existing canonical document lookup and saved end-page navigation. Missing sources are disabled, and an unfinished reader note blocks opening another reading.

The journal uses the established graphite/warm-paper palette, bundled editorial serif, a chronological session list and a selected-reading detail surface. Wide windows show list and detail together; compact windows place the selected reading and its opening action first, with timestamps in expandable details. A 140 ms selection emphasis respects reduced motion. Dates display in local time; active durations retain seconds and historical values. Loading, failure, empty-history and a page made empty by concurrent removal have explicit states. The latter can return to the first page.

Requests retain both query and cancellation identity. A newer request cancels the previous read, stale or mismatched replies are ignored, and navigation away cancels pending work and releases the resident page. A failed refresh retains its previous readings and offers another refresh. This destination does not write session rows or context.

## Verification

Two presentation-state regressions exercise superseded replies, a reply with the current token but wrong offset, navigation away before completion, failure/retry while retaining a real session page, and final governed-memory release. The locked workspace suite passed 350 tests before the visual correction; all 71 executable tests passed after the correction and stronger memory assertions. Final formatting and warning-denied workspace/all-target Clippy pass. Logs are `build/verification/reading-history-ui-tests.log` and `build/verification/reading-history-ui-final-tests.log`.

Five native DX12 captures use an isolated synthetic collection: dark English, reduced-motion light Spanish, offset 80, empty history and 640×480 with 200% text scale. The seeded history has 181 completed sessions across two documents and one unfinished session; the resident pages contain 80 rows and exclude the unfinished reading. The empty case contains no rows. The compact correction makes the selected title and opening action visible before scrolling.

Final capture evidence is `build/rust-visual/history-final/capture-manifest.json`. Executable SHA-256: `B56525F5FDDA626893B6B75953B691B211F9C82D042B50F987862850223F750F`. The executable, lockfile, all recorded UI sources, PNGs and control files were independently checked against the manifest. All five processes exited successfully. Captures and generated binaries stay outside the public checkpoint.

## Remaining work

Full context restoration is not implemented here. Native activation of the source-opening control, physical mouse/keyboard navigation, compact scrolling, error/retry interaction and live retention changes still require acceptance. The next-page fixture submits a scoped offset-80 request; it does not simulate a physical click. Text scaling is not physical monitor-DPI certification. Large-history SQLite latency, opaque allocation profiling, category cleanup, goals, recents/closed documents, workspaces, crash recovery and the complete Windows Release gates remain open.
