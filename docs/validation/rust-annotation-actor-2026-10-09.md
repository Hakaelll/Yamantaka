# Annotation actor checkpoint

Annotation reads and changes now run on the existing application actor. The presentation layer can submit an owned request without calling SQLite or PDFium directly. The visual annotation editor, marks and global review are still pending.

Requests own their query, cancellation token and a clone of the caller's shared memory governor. Creation and snapshot restoration retain existing governed payloads through Arc, without cloning their strings or geometry. Other mutations own their command values and borrow them into the domain repository call. A page can transfer its records to a caller without duplicating them, so mutation history can retain an original snapshot.

The actor derives search access from its current indexing consent. A terminal reply retains the document identity and original request token, including failures. These let the future editor match its pending request and reject stale replies. The actor checks cancellation before entering the repository; the repository retains transaction cancellation and rollback checks. Cancelling after a transaction commits does not undo the accepted write.

Annotation commands use the existing 128-command data admission limit. Each reservation follows its command through the terminal response. Delivery does not block when the UI stops consuming results. Dropping queued requests or responses releases their governed snapshots. The native worker has no annotation storage calls, and outstanding PDF work does not prevent the actor from handling annotation SQL.

## Verification

Three application integration tests use the actual schema-11 SQLite repository and the existing asynchronous native-engine fixture:

- Create, edit, add tags, review, check attachment, delete and restore the original snapshot while native PDF indexing holds an unresolved open ticket. The test checks Unicode text, identity, original timestamps, immediate annotation search, independent snapshot ownership and memory release. Native cleanup receives its terminal response before shutdown.
- Reject an unconfirmed fallback and a cancelled creation with the original request token, then accept an explicitly confirmed page comment. A denied Text-budget query returns a typed failure and releases its reservation.
- Hold 128 annotation reads/results, reject a further data command, admit a new command after consuming one response and release all queued payloads on shutdown.

Formatting, warning-denied workspace/all-target Clippy and workspace tests pass. The local full test log is `build/verification/annotation-actor-tests.log`. This checkpoint changes no rendered UI and adds no native visual acceptance evidence.

## Remaining work

The reader still needs an editor/controller that submits these requests, tracks pending tokens, preserves accepted write failures across document changes and displays loading, retry and confirmation states. Painted annotations, mutation history UI, global review, exports and the remaining selection, Windows, accessibility and release gates stay open.
