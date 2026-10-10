# Reader sessions and recording privacy

The native reader now instantiates the retained session lifecycle and sends its writes through the application actor. Opening an accepted document finalizes the previous session before replacing its page/context. Failed or cancelled opens do not finalize the current session. A new session starts after that document's reader preferences are available. UUID identifiers distinguish sessions without embedding a document path or title.

Input and viewport observations feed the monotonic activity policy. Qualifying time requires the reader surface, window focus and a non-minimized viewport; pointer, wheel, key, text and touch events refresh the inactivity window. No periodic reading timer or idle repaint was added. Session timestamps use a UTC origin plus monotonic elapsed time, preventing a backwards system-clock adjustment from invalidating a closure within this application run.

The reader caches its page and live context before an opening resets presentation state. Context records the actual displayed zoom, effective layout, reading mode/theme, vertical viewport offset and optional panel. Scroll observations update in-memory context without writing SQL for each frame. Closure values remain immutable during retries and document switches.

## Privacy and failure handling

Privacy now exposes a themed reading-time card and a recording checkbox. It reads the imported `recordReadingSessions` setting, including an explicit legacy false value. The missing-key default remains true, matching Flutter. An unknown non-boolean value disables recording while preserving the original preference. A Rust override is written atomically; failure restores the previous in-memory override. Changing this setting does not remove earlier history.

Disabling recording finalizes the current session and prevents another start. Enabling it permits a new session for the current document. Temporary actor admission pressure retains the write and schedules a bounded 100 ms retry repaint only while that write is waiting. Storage failures retain the session and expose an explicit retry control. Opening another document is rejected when retained-session capacity is exhausted.

The close-request path finalizes activity and waits for the actor's persistence barrier. Failure sends `CancelClose` and keeps the lifecycle for retry. Successful persistence clears retained state. `on_exit` repeats the barrier as a fallback before renderer shutdown. Crash termination is a separate pending recovery requirement; this path does not claim durability against forced process termination or power loss.

## Verification

Two presentation regressions verify imported privacy, unknown-value handling, preference rollback and preservation of unrelated legacy settings; and context mapping for effective Book layout, live zoom, scroll, theme and annotations panel. The existing lifecycle, SQLite and saturated-actor regressions remain green.

All 338 locked workspace tests pass. Warning-denied workspace/all-target Clippy and formatting pass. After the compact layout correction, all 69 UI tests were rerun successfully. Local logs are `build/verification/reader-session-integration-tests.log` and `build/verification/reader-session-ui-final-tests.log`.

Five final native wgpu captures use isolated synthetic stores in `build/rust-visual/reader-sessions-final`: Privacy in dark English, light Spanish with recording disabled/reduced motion, compact 640x480 with text at 200%, and readers with recording enabled/disabled. The initial compact capture put the checkbox below the viewport. The correction removes duplicate headings and places the recording control first; the final compact image shows the entire control. Long explanation text remains scrollable.

The recorded reader closes successfully and an independent SQLite connection observes exactly one completed session. Its document, page and every context value match the captured reader snapshot. The disabled reader has zero session rows and no session-context metadata. All five exit codes, PNG/control hashes, UI-source hashes, lockfile hash and executable hash were checked against `capture-manifest.json`. The final executable SHA-256 is `CA2525E85DE853F84C270DD66FE7EE0338D72225687066E3A0129D79802E20C6`.

## Remaining acceptance

Physical focus/minimize/input timing, interactive privacy toggling, injected native close-failure/retry, rapid real-document switches, long reading, Narrator, physical DPI and multi-monitor acceptance remain pending. These captures verify text zoom rather than physical monitor DPI. History queries/presentation, trimming and cleanup, goals, recently closed documents, restoration, workspaces and crash recovery remain open. Full idle/resource/performance comparisons and the complete Windows Release gates are unfinished.
