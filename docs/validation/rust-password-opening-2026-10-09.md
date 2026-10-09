# Protected document opening — 2026-10-09

The native reader now requests a password when an already catalogued PDF is
protected. Its modal identifies the document, focuses a masked field, explains
that the credential is not saved, and offers Open PDF and Cancel. Incorrect
credentials produce a fresh, empty retry field. Enter submits; Escape or clicking
the backdrop cancels. Destination changes revoke presentation intent; late opening responses cannot force the Reader back into view.

The application actor carries a document-specific `OpenFailed` event. A password
challenge therefore cannot be confused with an unrelated rendering or text error.
Retries keep the requested search page and the existing opening intent.

## Credential lifetime

`PdfPassword` owns a `zeroize` 1.9.0 buffer, implements redacted Debug, and has no
Clone or serde implementation. Ownership moves from the transient dialog through
the application command into the PDF worker. The worker releases that buffer
immediately after the PDFium open call, before metadata inspection. Rejection,
queue failure, cancellation and normal owner destruction also release it.

The password field clears egui's undo history after each frame and removes its
text-edit state on submission or cancellation. It uses password masking and
egui's password clipboard protection. No credential is added to preferences,
SQLite, index state, command arguments, backups or log messages.

This establishes the lifetime of the application's owned buffer, not forensic
erasure of every temporary copy inside egui, PDFium or the operating system.
The native dependency can make its own short-lived FFI copies. Passwords are not
remembered across native handle closure or application restart.

## Executed verification

- The workspace passes **119 tests**. UI has **13 passing tests**.
- A new domain test verifies diagnostic redaction and reads the still-owned
  allocation after explicit clearing to confirm its original bytes are zeroed.
- A new real PDFium/application actor test opens the generated protected fixture
  with no password and with an incorrect password, checks typed challenges, then
  unlocks it, extracts page text, closes it and compares original PDF bytes.
- Workspace Clippy with all targets and warnings denied and formatting pass.
- Five native captures use actual PDFium responses and synthetic data. The first
  inspection exposed an overly tall compact composition; one correction and one
  confirmation batch were completed.

| Capture | Evidence |
|---|---|
| [Dark prompt](../screenshots/rust-password/password-dark.png) | Initial challenge and focused password field |
| [Light, Spanish](../screenshots/rust-password/password-light-es.png) | Localized prompt and actions in the warm-paper theme |
| [Small window at 200%](../screenshots/rust-password/password-small-200.png) | Fully visible password field; lower actions remain in the scrollable dialog |
| [Incorrect password](../screenshots/rust-password/password-rejected.png) | Actual rejected PDFium attempt followed by empty retry field |
| [Unlocked document](../screenshots/rust-password/password-unlocked.png) | Correct synthetic credential opens and renders the protected PDF |

The capture manifest records executable, lockfile, source and output hashes;
images carry synthetic provenance. Source entries retain both captured-byte and
reviewed Git-blob hashes; four files differ only through CRLF/LF normalization. The automated retry captures invoke the same
retry action as the dialog, rather than injecting painted error or reader states.
They do not prove real keyboard, Narrator, physical DPI or multi-monitor behavior.

Protected-file import queue handling is still pending: this stage opens existing
catalogue entries, including those preserved by migration. It does not establish
password-aware import, background indexing of protected files, full migration
acceptance or a replacement release. The complete parity matrix remains open.
