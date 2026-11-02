# Protected PDF full-text indexing

Follow-up: [guarded full-source indexing](rust-search-source-integrity-2026-11-02.md) adds full hashes, source read guards and durable cursor proofs. The evidence and remaining work below describe the earlier unlock checkpoint.

## Connected behavior

The existing opt-in index controller now uses the application's bounded, one-shot password broker with an indexing purpose. It pauses on a protected open, reports an incorrect credential through the same masked dialog, and submits the owned zeroizing password for that native opening only. The controller does not retain a reusable credential.

Cancelling the unlock prompt stops the indexing run and closes its native owner. Yielding the native slot instead cancels that owner's work and requeues the document after confirmed closure. The repository's committed page cursor is retained. Reopening after native owner loss requests a new password, checks page geometry and the native source revision, and continues at that cursor. The text worker now reports a missing native owner through `NativeDocumentUnavailable`, matching the recovery contract instead of returning an undifferentiated PDF error.

Initial source admission compares the native legacy fingerprint with the index session. Reopening the same active job also compares its source revision and page geometry. These are bounded checks; this checkpoint does not add full-file hashing or a lifetime source read guard to indexing.

The unlock dialog identifies **Index PDF** and **Cancel indexing**. It explains that extracted text is persisted unencrypted, while the password is never saved. At compact sizes this disclosure stays outside the scrolling input body and the shorter **Cancel** action keeps both actions on one row. Existing local-index consent, exclusions and deletion remain in effect. No renderer priorities or hot-document limits changed.

## Executed evidence

- The retained application test `native_reader_preemption_resumes_the_interrupted_page_and_keeps_prior_evidence` passes with real bundled PDFium. Its protected-index helper covers a wrong password followed by a correct one, prompt cancellation, native closure after the first committed page, slot yielding and re-admission, and the actual runtime password broker.
- Both recovery cases retain the committed cursor, request a fresh credential, complete without failed documents, and store page text identical to extraction from a separately opened reference document. The runtime case finds a literal fixture word through page-text FTS search.
- The parent protected-import helper still checks that original PDF bytes are unchanged and neither synthetic credential appears in persistent files. No standalone test was added.
- Workspace Clippy with all targets and warnings denied passed. The native executable built offline. UI-only compact-dialog adjustments then passed focused UI Clippy and a fresh native build.
- [Two native synthetic captures](../screenshots/rust-protected-search/capture-manifest.json) exercise the actual indexing challenge: dark English at the regular size and light Spanish at 640×480 with text scaling at 200 percent. Enabled submit/cancel bounds are checked against their clip and viewport. Both images were visually inspected. The small layout was revised after the initial images hid the disclosure or displaced the input. The final compact body starts with the password field and keeps its disclosure above the scroll area.

## Remaining acceptance

This fixture-level checkpoint does not prove whole-rewrite parity. Physical keyboard/Narrator/DPI interaction, protected indexing across forced process exits, source replacement races, source guards and full-digest baselines for resumed indexing, corrupt/large PDF corpora, complete memory accounting and paired performance measurements remain pending. The existing queue-full and cleanup-failure paths were reviewed, but this run did not inject those failures. It does not authorize a replacement Windows Release.
