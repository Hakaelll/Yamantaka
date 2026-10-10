# Incremental annotation export writer

The application crate now writes annotation Markdown and versioned JSON incrementally from borrowed repository records. This is the format implementation for the export flow. The save dialog, coherent repository traversal, actor command and atomic file publication are not connected yet.

JSON retains the `yamantaka-annotations` version 1 envelope and `normalized-page-v1` coordinate declaration used by Flutter. It preserves identity, page, type, color, note, selected text, surrounding context, rectangles, rotation, source fingerprint, attachment state, tags and creation/update times. Optional origin fields and the separate editorial review state are included as additional fields. Optional absent text fields remain absent. Export and record timestamps are validated as RFC 3339 and normalized to UTC. Progress is finite and clamped to the legacy range.

Markdown retains Flutter's Spanish export labels, page order, colors, tags, quoted passages, notes and page links. It escapes Markdown syntax and user-supplied HTML characters. One numeric anchor per page makes the generated page links resolve, including multiple annotations on the same page.

The writer accepts records in nondecreasing page order for one document. It holds no collection of records or complete output string. The caller owns the output writer and its buffering policy. Cancellation is checked before the header, each record and finalization. An invalid record, wrong document, reversed page order or write failure poisons the stream so later calls cannot return a successful completion. Error messages omit destination details. Finalization also checks flush failures.

## Verification

Five application integration regressions check:

- Unicode text, geometry, tags, identity, review/origin fields, maximum page index, UTC normalization and progress clamping in parsed JSON.
- Escaped Markdown/HTML content, quoted multiline Unicode text and one anchor per page.
- Prior and final cancellation, invalid metadata, wrong-document/reversed-page rejection and poisoned stream completion.
- 181 annotations supplied across three real SQLite repository pages, with the final record present, optional fields absent and source snapshot leases released between pages. An empty export is valid JSON.
- Late output and flush failures, redacted destination errors and invalid timestamps.

Formatting, warning-denied workspace/all-target Clippy and workspace tests pass. The full local test log is `build/verification/annotation-export-writer-tests.log`. The application crate reuses the already pinned workspace chrono dependency; no dependency version changed.

## Remaining scope

The paged fixture is static and does not prove a coherent snapshot while another process writes. The export command still needs repository traversal in one read transaction, actor routing, cancellation/error presentation, a native save dialog, original-PDF protection and atomic destination publication. A failed writer can leave partial bytes in its destination, so the connected flow must write to a staged file and publish only after successful completion. File overwrite confirmation, native export acceptance and large-export latency/resource checks remain pending. This checkpoint does not establish complete annotation export parity or release readiness.
