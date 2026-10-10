# Native reading goals

The document inspector now reads and edits optional page and minute targets. The editor identifies the selected book, accepts either target independently, treats blank fields as absent and offers Save, Clear and Cancel. Page progress follows Flutter's existing current-position convention, capped at the target. Minutes remain a target rather than an invented session-progress measurement.

## Compatibility and persistence

`reading_goals_v1` remains in schema-v11 app metadata. The adapter reads Flutter's nullable integer fields and clamps historical values to 1–10000 pages and 1–1440 minutes without rewriting them during a read. New targets must already fall within those bounds. Malformed roots, duplicate target/known keys and invalid known types produce a recovery error and preserve the original record.

A write uses a SQLite transaction and parameterized document-key matching. Updating a target preserves its unknown fields and all unrelated metadata entries, including non-object JSON values. Clearing removes only that document's entry; the metadata record disappears when the map becomes empty. Cancellation, aborting triggers and ignored writes roll back the value and timestamp. The repository verifies the stored outcome before committing. No JSON path is constructed from a document ID, and PDF originals are untouched by this command.

## Connected interface

Commands run through the bounded application actor. The interface keeps one selected document and one admitted write, matches document/change/cancellation identity on replies and rejects superseded reads. A selection change does not retarget or cancel an admitted write. A failed write keeps its draft available for retry; an error received after selecting another document appears in the application error area.

The inspector retains the goal summary while a separate editor provides labeled inputs and the same application controls used elsewhere. Large windows show explanatory text and the document's editorial title. Compact windows prioritize the title and fields, keep actions outside the scrolling area and reserve space for them at 200% text. Both fields expose their visible label through egui accessibility semantics. Physical keyboard/Narrator acceptance is still pending.

## Verification

Ten new regressions cover strict new bounds and legacy clamping, quoted/Unicode document IDs, unknown-field and JSON-type preservation, no-op clearing, malformed/duplicate metadata, cancellation, unavailable documents, late abort/ignore rollback, removal of the last metadata record, scoped actor replies while native PDF opening is held, stale presentation replies and draft retention after failure.

The locked workspace passes 375 tests. The final executable state tests were rerun after the presentation/module changes, with 79 passing. Formatting and warning-denied workspace/all-target Clippy pass. Logs remain in `build/verification/reading-goals-final-tests.log` and `build/verification/reading-goals-ui-final-tests.log`.

Five native DX12 captures cover the dark English editor, light Spanish/reduced-motion editor, compact 640×480 with 200% text, and successful save/clear outcomes. The compact capture keeps all three action bounds within the window clip; the field area can scroll. Synthetic preview commands use the same state/request methods as the visible controls. Independent read-only SQLite checks confirm the saved 25-page/45-minute target, absence of the cleared document entry, retention of the other document's target and unknown fields/JSON values, and no generated reading sessions.

The final manifest is `build/rust-visual/reading-goals-final/capture-manifest.json`. Executable SHA-256: `8C9DA5904D35CA1C138BB94DC061BD695F8187046A17F2D9390695753E601EBF`. The executable, lockfile, recorded UI sources and five PNG/control pairs were independently checked. Every capture process exited successfully. Generated captures and stores remain outside the public checkpoint and use isolated instance identities.

## Remaining acceptance

Physical input, focus order, Escape behavior, scrolling and screen-reader operation, external concurrent goal changes and native error/retry interaction remain pending. The full rewrite still requires the remaining continuity features, broader parity, physical Windows/accessibility validation, paired performance measurements and the complete Release deliverables. This checkpoint does not certify those gates or replace the Flutter application.
