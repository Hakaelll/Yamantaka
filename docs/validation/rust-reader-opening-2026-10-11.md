# Reader preservation during native opening

Opening another PDF now retains the current reader until the candidate's scoped native reply is accepted. Candidate opening and preference tokens have independent lifetimes from the current page rendering and preference tokens. Superseding, cancelling or failing a candidate releases its own scopes without clearing the current layout, page images, navigation, text selection, outline or preferences.

A password challenge keeps the candidate preference scope. Each password retry receives a fresh native request identity, and cancellation discards only the candidate. Successful opening validates the new navigation and admits completion of the previous reading session before replacing reader state. Historical context restoration binds to the candidate preference scope and survives the accepted handoff. Late or mismatched replies cannot replace the current reader.

## Verification

All 387 locked workspace tests pass, including three new opening-state regressions. Formatting and warning-denied workspace/all-target Clippy pass. The regressions cover superseding requests, stale and foreign replies, exact-once preference adoption, password retry identity, cancellation and failure, and accepted scopes surviving opening-state destruction. Workspace evidence is in `build/verification/reader-opening-tests.log`.

Six native synthetic captures in `build/rust-visual/reader-opening-final` cover failed replacement in dark, light Spanish with reduced motion, and 640 by 480 with doubled text; protected-document challenge and successful unlock; and historical restoration. Each failed replacement retains document `demo-0`, zero-based page 1, custom zoom 1.75, continuous document layout, sepia theme and scroll offset 1500. The attempted replacement uses a separate unavailable synthetic path. It does not modify the current PDF.

The final capture manifest verifies all 50 UI source hashes, Cargo.lock, PNGs, control snapshots and successful process exits against executable SHA-256 `B32D320CFBBF651D25CAFAF59D19A4B18CCBE7F032739F00D7CE59F6ADF85DEE`. The ordinary dark screenshot visibly retains the rendered current page after the PDFium error. The compact doubled-text screenshot retains the recorded reader context, but the error banner consumes substantial viewport height and leaves little paper visible. This is a remaining interface issue, not complete compact-layout acceptance.

## Remaining acceptance

Visible tab selection, closure/reopen and persisted tab context are still pending. Physical rapid switching, replacement-password cancellation while an existing reader is visible, preference-load failures, Narrator and monitor-DPI acceptance require further verification. The synthetic screenshot text scale does not certify physical DPI or accessibility. This checkpoint does not complete the rewrite or qualify a final Release.
