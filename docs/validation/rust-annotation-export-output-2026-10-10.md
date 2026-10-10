# Governed and cancellable annotation export output

The connected export service now groups formatter fragments in a fixed 64 KiB output buffer before writing to its destination. It reuses the domain's fallible TrackedBytesBuilder and shared memory governor. Actual allocated buffer capacity is counted as Other/CPU memory and contributes to the global limit while the buffer exists. Buffer admission failure returns MemoryBudget before streaming records.

The output adapter checks cancellation before accepting fragments, between bounded destination writes and around final flushing. It handles partial writes and retries Interrupted results while still checking cancellation. A zero-byte write or destination failure poisons the output, preventing later retries from repeating an already written prefix. It intentionally does not flush pending bytes when dropped.

The application formatter still returns the destination writer only after successful traversal and explicit finalization. Its error boundary turns a cancellation observed during output into the typed Cancelled result, while ordinary output failures retain the existing redacted storage error. The staging layer consequently discards failed or cancelled output instead of publishing a completed file. A published success is not reclassified by this error mapping.

## Verification

Five output regressions prove:

- Fragmented Unicode input produces exactly the original bytes. The fixture contains 400,000 UTF-8 bytes supplied as character fragments; the instrumented destination receives seven writes, each at most 65,536 bytes. This counts calls to the test destination and does not measure elapsed disk performance.
- Denied output admission and cancellation before entry retain no buffer memory.
- Cancellation after the first partial destination write stops the drain, poisons the adapter and prevents replay of the prefix.
- Dropping unfinished output writes no pending bytes.
- Interrupted writes resume; zero writes and flush failure poison subsequent operations without additional writes.

Two new file integration regressions use the connected SQLite visitor, formatter and staged publication boundary. Both JSON and Markdown cancel during the first 128,000-byte note, after some bytes reach the temporary file but before that note completes. No final file or temporary entry remains, and all governed source/output memory is released. Denying Other memory also leaves no file, and retry with an available governor successfully publishes all 181 records.

The existing complete-format, malformed-final-record, runtime, UI-state and repository regressions remain passing. Formatting, warning-denied workspace/all-target Clippy and the full locked workspace test suite pass. Local log: `build/verification/annotation-export-output-tests.log`.

## Remaining scope

Cancellation checks do not interrupt a blocked operating-system write or sync call. SQLite decoding and other opaque native work still need latency/lock-contention acceptance. The buffer is governed, but complete SQLite/output resource accounting, a transactional metadata snapshot, crash cleanup, native save-dialog and error/cancel interaction acceptance, replacement policy, per-record progress and broader annotation parity remain pending. No new native visual pass is claimed for this backend-only change, and no end-to-end performance improvement or final release parity is claimed.
