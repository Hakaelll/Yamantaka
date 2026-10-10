# Global review export worker

The existing independent annotation export worker now accepts an owned typed source. Document export carries document metadata and streams the complete document. Review export carries the chosen filters, grouping and page offset, captures the coherent review snapshot on the worker's read-only SQLite connection, and writes its bounded global format.

Replies carry a typed Document or Review scope and the original request token. Queue admission errors, worker disconnection and normal completion retain that scope. The reader's export state accepts only its expected Document scope and matching token, so a Review reply cannot complete it even if a token matches.

Both sources share the one-job admission limit, cancellation/shutdown handling and existing safe file publication. The global path does not add another thread pool or bypass occupied-file/original-PDF protection. A failed snapshot, writer or publication removes staged output through the same existing publication contract.

## Verification

A new file-backed runtime integration regression holds a controlled PDF engine's open ticket while exporting global review results. JSON and Markdown publish successfully before that ticket is released. The test checks the 80-record first page, 21-record final page, two-note text filter, manual metadata, the global format and request identity. It also verifies that no additional PDF work is requested, occupied output bytes remain intact, denied memory/cancelled requests/mismatched extensions publish no file, and governed memory returns to zero.

The reader export-state regression now explicitly rejects a Review reply with its active token. Existing document worker, queue, cancellation and publication tests exercise the migrated typed contract. This is controlled runtime concurrency evidence; it is not a new real-PDFium latency measurement.

Formatting and warning-denied workspace/all-target Clippy pass. All 321 locked workspace tests pass. The result is recorded in the local ignored log `build/verification/annotation-review-worker-tests.log`.

## Remaining acceptance

Global save controls and their native selector, cancellation/recovery UI and synthetic captures remain pending. Physical native dialog interaction, concurrent mutation stress and process interruption acceptance remain open. The full rewrite and Windows Release gates remain incomplete.
