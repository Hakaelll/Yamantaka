# Global library notes

The Tools menu now opens a global Notes screen with a list and passage detail. It queries the independent Rust store through the existing annotation actor. The screen combines text, exact tag, six mark kinds, five colors, editorial review state and recent-update filters. Apply commits the draft filters; Reset restores the default document order. Previous and Next load at most 80 records per request.

Document, page and review ordering is applied in SQLite before pagination. Document labels use the catalogue's manual-title precedence. Presentation titles are included in governed result memory and are not written into annotations or added to export traversal. Annotation pagination no longer stops at an arbitrary 100,000-row offset.

Each selection freezes both document and annotation identity. Review and tag actions reuse the previously verified atomic cross-document transaction. Failed writes retain the selection and tag draft. Leaving the screen cancels reads, releases their results and preserves an accepted write until its scoped completion. A write completing after departure invalidates the reader's cached records and painted marks. A failure after departure is surfaced through the application's error state. An open reader editor prevents global mutations and source opening from discarding its draft.

Open in document uses the existing canonical lookup/open intent with the annotation's page. It does not introduce another PDF navigation path. Source opening retains missing-file recovery through the existing document lookup.

## Verification

- Formatting and warning-denied workspace/all-target Clippy pass.
- The full locked workspace suite passes, with 312 passing tests and no failures. After correcting capture readiness, the affected UI suite passes again with 66 tests. Logs are local ignored artifacts at `build/verification/global-notes-tests.log` and `build/verification/global-notes-ui-final.log`.
- Three new storage regressions verify manual titles, page/review order, grouping before 80-row pagination, governed metadata, unchanged complete export traversal and access to records beyond offset 100,000 in a 100,123-note fixture.
- Three new SQLite-backed UI regressions verify stale replies and document scope, frozen cross-document targets, writes completing after departure, failure recovery, combined typed filters with a UTC seven-day threshold, cancellation and memory release.

Seven native synthetic captures were generated at `build/rust-visual/global-notes-native-final`. Their manifest records the executable, Cargo lockfile, every UI Rust source and each image/control hash. All recorded hashes and functional assertions were checked against the actual files. Executable SHA-256: `D60CECBC649183DB3DB0FBC13322F851D4837A89B636CFF16CDE03FBB5FE554B`.

| Case | Evidence |
| --- | --- |
| Dark list and expanded filters | 181 total notes; 80 resident records covering two documents; native wgpu presentation |
| Light Spanish review | Three targets submitted across two documents; saved state; two accepted records remain in the first page after canonical update ordering moves the third later in its document group |
| Tag addition | Three resident records have Study; saved state; selection cleared |
| No matches | Zero total and resident records with a completed query |
| Open in document | Reader reaches source page zero; one ready painted page contains the source document's two marks |
| 640 x 480 with text at 200 percent | Screen renders and the completed data query is retained; initial viewport shows the heading and beginning of search, not all actions |

The first capture attempt timed out because the harness waited for an invisible reader annotation panel on a non-reader screen. The capture condition now requires reader-panel readiness only on the Reader screen. The corrected final batch completed all seven cases. This correction does not relax the global query or PDF source-opening readiness requirements.

## Remaining acceptance

These are synthetic native captures and controller/storage regressions. They do not prove physical keyboard traversal, scrolling through the compact screen, Narrator, monitor DPI changes or multi-monitor operation. The global screen still needs deeper visual refinement and complete interaction acceptance, including selection/actions with long real titles and passages. Global export controls, mutation history and the remaining annotation parity work are not completed by this checkpoint. The full rewrite and Release acceptance gates remain open.

## Reading presentation follow-up

The list/detail rendering now lives in a dedicated `global_notes/views.rs` module. The current row has a quiet surface treatment and a labelled checkbox beside its document control. This preserves separate passage focus and batch selection. The detail uses the existing bundled Source Serif Display family for selected passages, with sans-serif document metadata and user notes. Apply is now a primary action in the grouping row, and narrow screens use a smaller heading.

Current-row emphasis interpolates over 140 ms using egui's bounded animation. Reduced motion applies its final state immediately. This adds no unconditional repaint loop; physical interaction timing and idle CPU measurement are still pending.

Seven follow-up native captures at `build/rust-visual/global-notes-polish` cover an original synthetic passage in dark and light Spanish themes, reduced motion, expanded filters, review, tagging, source opening and compact text zoom. Every executable, lockfile, UI-source, image and control hash in the manifest was verified. Executable SHA-256: `08EE6D3996E7D5AFFD92BD75F3DD259A8FB1E6A7E6BBEAB8DF7B0704AB8EF948`. The functional assertions retain 181 total notes, 80 resident records across two documents, three Study tags after tagging, two accepted records in the refreshed first page and two source-page marks after opening. The UI suite passes with 66 tests, and workspace/all-target Clippy passes without warnings. These captures supplement the initial checkpoint; they do not replace the remaining acceptance gates above.
