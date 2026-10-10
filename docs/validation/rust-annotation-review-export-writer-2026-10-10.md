# Global annotation review export writer

The connected Flutter global-review callback in `lib/app/app_shell.dart` exports the loaded review page, capped at 80 annotations. It groups those annotations by document. JSON uses the `yamantaka-global-annotations` version 1 envelope with nested versioned document exports; Markdown joins the existing per-document output with a horizontal separator. This differs from the reader's complete-document export, which must traverse every annotation.

The application now has a separate bounded review writer that preserves this contract. Its inputs borrow frozen records and document metadata. It validates document identity, nonempty groups, duplicate documents/annotations, finite progress and the 80-record limit before writing. A fixed stack array orders each group's records by page, preserving incoming order within a page without allocating an annotation copy. The existing document formatter supplies both nested JSON and Markdown sections. All nested documents use the same normalized UTC export time.

The global wrapper shares the existing governed 64 KiB output buffer, partial-write checks and cancellation handling. The function returns a count only after all document sections and the enclosing output finish successfully. No new file-publication path is introduced by this formatter checkpoint.

## Verification

Four new application regressions cover:

- Two documents with Unicode passages, tags, metadata and geometry. Parsed nested JSON matches independently generated document exports; progress is clamped consistently, pages are ordered and the global envelope retains its version and UTC timestamp. Markdown contains both documents and preserves quoted Unicode line boundaries.
- Duplicate annotations, more than 80 records, stale document identity and duplicate document groups are rejected before any output bytes are written.
- Empty JSON review output is valid, cancellation before entry writes nothing, and output memory is released.
- Cancellation after the first document flush returns Cancelled, leaves an incomplete global JSON envelope and releases the output buffer. A partial stream is not reported as a successful export.

Formatting and warning-denied workspace/all-target Clippy pass. The full locked workspace suite passes with 316 tests and no failures. Verification is recorded in the local ignored log `build/verification/annotation-review-export-tests.log`.

## Pending integration

This checkpoint supplies the compatible formatter. The global screen does not yet call it. A coherent storage snapshot must supply frozen page records and catalogue metadata, and an owned worker request must route the snapshot through the existing safe publication path and native save selector. Global export controls, cancellation/recovery captures and their interaction acceptance remain pending. Existing reader exports still use complete-document traversal. This report does not claim that global export or the rewrite is complete.
