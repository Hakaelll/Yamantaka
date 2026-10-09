# Annotation geometry for every visible page

The reader now loads a separate geometry snapshot for each visible PDF page. Changing the 80-row Notes list no longer changes which marks the reader can paint. The geometry query includes every attached annotation on that page, without a pagination limit.

The domain payload owns kinds, colors and normalized rectangles. Its arrays and rectangle buffers retain leases from the shared memory governor. SQLite reads the count and geometry in one transaction and reserves memory before allocation. It does not load note text, selected text, tags or extension values for painting. Invalid geometry, cancellation and denied memory admission return a failure and release the partial snapshot; they cannot publish a truncated successful result.

Owned requests and replies pass through the application actor with its existing terminal response admission. The UI admits at most two geometry requests at once, retains only visible pages and checks document, page, cancellation and request identity before accepting a result. Leaving the reader or changing documents drops the snapshots and cancels reads. Successful annotation writes invalidate the geometry cache; paging the list leaves it intact. Failed geometry loads have details and a reload action in the Notes panel.

## Verification

Two SQLite regressions check 181 attached marks on one page, exclusion of other pages/documents and detached records, and separation from the 80-row list. Large note payloads prevent full-record admission under a 128 KiB text budget while the geometry snapshot fits. They also cover prior cancellation, denied admission, a late invalid rectangle, recovery on another document and release of all governed bytes.

An actor regression checks scoped geometry replies, cancelled requests, stored selection geometry and independence from native PDF extraction. Two UI cache regressions check wrong-document/replaced replies, visible-page eviction, invalidation cancellation, stale replies, wrong-page failures and fresh admission after invalidation.

Formatting, warning-denied workspace/all-target Clippy and workspace tests pass. The full local log is `build/verification/annotation-paint-tests.log`.

Three native captures are local in `build/rust-visual/annotation-paint-native`:

- `annotation-many-rotated` shows 181 synthetic marks at 90 degrees. State records 80 list entries and 181 geometry marks on the same visible page.
- `annotation-saved-rotated` creates a native-text highlight through the actor. The accepted write closes the editor and the refreshed geometry snapshot contains its mark.
- `library-dark` checks that reader geometry readiness does not prevent the library capture.

Manifest SHA-256 is `719EA1ED131C87CD54E4BC895DDF132E60D334164B15C70E970A0B7475762041`. Executable, Cargo.lock, source, image and control-state hashes match, and control bounds are finite. All captured documents, notes and rectangles are synthetic.

## Remaining scope

The projection removes the list-page coverage limit. A page whose complete geometry cannot fit the shared budget reports an error instead of silently dropping marks. Large annotation-page latency and memory-pressure interaction acceptance remain open. This change does not certify intrinsic/directional line placement or distinct note/marker affordances. Mutation history, tags, global review, exports, broader pointer/keyboard acceptance, physical DPI, Narrator, multi-monitor, performance comparison and final release gates remain pending.
