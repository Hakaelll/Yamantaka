# Native reader tab selection

The reader now connects the domain tab controller to native document opening. Its visible selector supports selecting readings, duplicating a reading with independent context, pinning up to three new readings, and moving tabs left or right from the context menu. The compact selector exposes those actions in its menu. Ctrl+Tab and Ctrl+Shift+Tab cycle readings while preserving modal focus routing.

Default library opening reuses a matching tab. A pending cross-document selection first resolves the current catalogue record using a scoped lookup, then submits a native opening with a separate request token. The tab plan becomes active only after a matching successful native opening, valid navigation and admission of the outgoing reading-session completion. Failed, cancelled, foreign or superseded candidate replies retain the previous tab model and reader. Leaving the reader cancels an unadmitted lookup; an admitted native handoff can complete without forcing the reader surface back into view.

Same-document duplicate selection shares native ownership while retaining separate page, scroll, panel and complete ReaderPreferences snapshots. Restoration preserves fit-width/fit-page mode, rotation, margins and the other stored fields. It does not reduce a tab to the partial historical-session context format. If the outgoing reader scrolls while a candidate is opening, adoption copies its latest snapshot back into the staged plan. Accepted preference restoration uses the exact candidate preference token once.

A note editor blocks switching at request admission and again at native acceptance, covering an editor opened while the candidate was pending. Candidate lookup/preference failures release their own presentation intent. Captions are limited to 512 Unicode scalar values each and retained only for live and remembered closed tabs. Tab context itself remains in memory at this checkpoint; persistence is still pending.

## Verification

All 392 locked workspace tests pass. Four new tab-state regressions cover lookup/native/preference identity, exact-once full-context restoration, failed and superseded selections, independent duplicate positions, latest outgoing scroll and leaving the reader during lookup versus admitted native opening. A fifth regression covers both tab keyboard shortcuts outside the reader and inside a modal. Formatting and warning-denied workspace/all-target Clippy pass. The workspace log is `build/verification/reader-tabs-ui-workspace.log`.

Seven final native synthetic captures in `build/rust-visual/reader-tabs-final` completed successfully. The real traversal opens a second document, duplicates its reading, pins and reorders that duplicate, then returns to the first document. The recorded model contains three unique tab identities, two documents, one pinned reading, and the first reading restored to zero-based page 1, custom zoom 1.75, continuous sepia layout and scroll offset 1500. Both readings of the second document retain fit-width mode, 90-degree rotation and horizontal margin 31. The final dark, light Spanish with reduced motion, and compact doubled-text screenshots were inspected. Protected opening/unlock, failed replacement and historical restoration also completed.

The capture manifest independently verifies all 55 UI source files, Cargo.lock, seven PNGs, seven control snapshots and successful process exits against executable SHA-256 `8EFBDC64ECD0843C27C5422275E57A0722D108B970171054DBE85A018A27150F`. Source inventory equality is checked as well as hashes, so newly added UI modules cannot be omitted. The wide bar marks the selected reading with a violet underline; the compact window displays the selected document without overflowing its closed selector. Physical menu interaction remains pending. Doubled text in a synthetic screenshot is not certification of physical DPI or Narrator.

## Remaining acceptance

Connect visible closure/reopen and their scoped native outcomes, persisted recent closures, startup/workspace restoration and queue/focus/compare flows. Verify replacement-password cancellation with an existing reader, physical mouse/keyboard/focus interaction, rapid switching, tab resource lifetimes, Narrator and monitor DPI. Domain resource classifications are not proof of the native warm-handle policy. The current worker still enforces its existing two-document bound.

No final tab parity or Release completion is claimed. Visible closure/reopen, persistence and the remaining acceptance above must be completed before shipping the rewrite.
