# Bulk reader tab closure

Wide tab context menus and the compact reader selector now expose Close other readings and Close readings to the right. Both commands use the validated domain tab rules, retain pinned readings and stage the resulting tab plan through the existing scoped reader handoff. If the selected PDF cannot open, the current reader and tab set remain intact. Successful adoption updates the outgoing reading context before recording its closure.

Native retirement now collects every removed document that has no surviving tab, rather than only the first removed document. Duplicate tabs share a single retirement. A serial queue admits one scoped close at a time, so a bulk action does not compete with itself for the native control reservation. Its union of queued, failed, pending and newly requested document identities is limited to twelve. Invalid identity or capacity failure leaves the queue unchanged; switch-plan and individual-close admission check this bound before native work.

A failed native retirement retains its document identity and pauses the remaining queue. Retry close schedules that failed document before the remaining work. The queue does not automatically retry a permanent error or keep the UI busy while paused. Successful reopening removes that document from both queued and failed retirement, preventing cleanup from later closing a newly accepted reader. Scoped reply identity and the existing late-cancellation native-outcome rule remain unchanged.

## Verification

All 399 locked workspace tests pass, including three new regressions for serial/deduplicated retirement, failure pause and exact retry, atomic capacity rejection, reopened-document removal, and multi-tab plan retirement with a surviving pinned reading. Formatting and warning-denied workspace/all-target Clippy pass. The final workspace log is `build/verification/reader-bulk-close-tests.log`.

Eight native synthetic captures in `build/rust-visual/bulk-close` cover both bulk commands, light Spanish with reduced motion, compact doubled text, inactive closure, individual close/reopen, last closure with session recording, and the existing tab selector. The final manifest independently verifies all 59 UI source files, full source inventory, Cargo.lock, PNG/control hashes and successful process exits against executable SHA-256 `A1F6D8F56F3167AB6F16E9EC66EE9A8F8A64C0BF6242AED80E13680AD55FE9E0`.

The bulk traversal creates five readings across four documents, including a pinned duplicate. Close other readings leaves the first reading and the pinned duplicate, records three closed snapshots and confirms two distinct native closures. The first reading retains zero-based page 1, zoom 1.75 and scroll 1500. Close to the right retains the pinned duplicate and the readings to the left, selects the unpinned second-document reading, records two closures and confirms two distinct native closures. Its fit-width mode, 90-degree rotation and horizontal margin 31 remain intact. Every final capture has no queued, failed or pending native closure. The ordinary bulk-close screenshot and compact close-right screenshot were inspected.

## Remaining acceptance

Connect persistent recently-closed readings, startup tabs, workspaces, reading queues, focus and comparison flows. Verify the intended native warm-document policy, retained resources after repeated cycles, physical menu/keyboard/focus interaction, protected-document cancellation and mixed rapid switching. The queue failure/retry contract is covered by state regressions; native UI fault injection, Narrator, monitor DPI and multi-monitor acceptance remain pending. Synthetic doubled text does not certify physical DPI. This checkpoint does not complete the rewrite or qualify a final Release.
