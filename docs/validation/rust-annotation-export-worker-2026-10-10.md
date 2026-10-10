# Independent annotation export worker

Annotation export now has an owned application request and a scoped terminal reply carrying the document ID, exported record count or error, and original cancellation token. The runtime admits one export worker at a time. Its response permit remains owned until the terminal event is consumed, using the existing bounded event admission rules.

The storage adapter opens a separate read-only SQLite connection to the file-backed library. That connection runs the canonical coherent annotation visitor and the existing staged file publication boundary on a named export thread. Neither the interface nor the main use-case actor streams the output. Catalogue commands and pending PDF tickets continue to run while the export is held. The runtime's blocking readiness select observes the export receiver, so completion wakes the actor without an idle polling loop.

Cancellation reaches the writer through its existing shared token. Runtime shutdown cancels and joins an unfinished export before releasing the actor. Receiving and joining a successful job does not cancel its token. An unexpected worker disconnect returns a fixed scoped storage error. The reader uses a read-only connection and cannot change the library through annotation mutation methods.

The domain repository exposes independent-reader and successful-publication ports without depending on SQLite, UI or the application formatter. Existing fixture repositories retain explicit Unsupported defaults. In-memory libraries do not have an independent export connection; they return Unsupported rather than silently copying an unbounded database into memory. The production store is file-backed.

## Verification

Three controlled application lifecycle regressions prove:

- A held export allows a catalogue query to complete, rejects another export with its own scoped QueueFull reply, and eventually completes without cancelling the successful request token.
- User cancellation produces the matching terminal reply, releases the reader and allows another export to start.
- Runtime drop cancels and joins a held worker without the UI draining results. The reader's drop notification is already available when runtime drop returns.

A SQLite-backed runtime regression publishes all 181 annotations to a Unicode JSON file while an asynchronous PDF-open ticket remains unresolved. It checks the final record, document/request identity, released governed row memory, occupied-destination byte preservation and pre-cancelled requests. The held ticket uses the existing controlled engine fixture; this test does not claim a new native PDFium run.

A storage regression attempts removal through the independent export reader and confirms a read-only error and unchanged live row. A subsequent write through the main connection remains visible to a new reader query. The fixture also checks the explicit in-memory Unsupported result and released source memory.

Formatting, warning-denied workspace/all-target Clippy and the full locked workspace tests pass. The local log is `build/verification/annotation-export-worker-tests.log`.

## Remaining scope

Native save dialog, explicit replacement confirmation/policy, progress/error presentation, UI request lifecycle and native export acceptance remain pending. The UI currently recognizes export errors, but does not yet initiate export requests or display successful completion. Metadata is frozen by the request rather than captured in the same transaction as the annotation snapshot. Worker launch/disconnection fault injection, SQLite lock contention, large-export latency, intra-row cancellation, crash cleanup and full resource accounting still need acceptance. No complete export interface or release parity is claimed.
