# Protected reference checkpoint

A protected reference now uses the reader's existing password dialog. Its target identifies the reference, so retrying or cancelling does not replace the main reading or modify its tab plan. Incorrect credentials return to the same dialog with the existing rejected-password message. Cancelling retains the earlier reference. Closing or leaving the reference while a challenge is pending releases that challenge before continuing.

The password still uses the existing `PdfPassword` ownership and zeroization. The shared masked editor clears its undo history and removes its state on submission or cancellation. No credential persistence, serialization or diagnostic logging was added. Importing files or folders dismisses the challenge through the same target-aware cancellation method, preventing an invisible blocked reference after external activation.

The native binary compiled. Three focused synthetic scenarios passed:

- `reader-reference-password-retry`: a clear primary PDF stays open while a separate protected PDF rejects the first credential and accepts the second; the reference renders after exactly two attempts.
- `reader-reference-password-cancel`: cancelling before retry retains the original reference and primary page, with no credential submitted.
- `reader-reference-password-import`: production import runs while a reference challenge exists; the challenge is dismissed, its admission returns to ready, and the primary reading remains available.

The [unlocked reference](../screenshots/rust-reference-password/reader-reference-password-retry.png), [cancelled unlock](../screenshots/rust-reference-password/reader-reference-password-cancel.png) and [import recovery](../screenshots/rust-reference-password/reader-reference-password-import.png) use generated acceptance PDFs. The primary source is `textual.pdf`; the secondary source is `protected.pdf`. Capture arguments and records contain only scenario names, attempt counts and synthetic document identities. No new standalone tests or broad suite were run.

These scenarios drive request routing and completion through the native runtime; they do not certify physical keyboard/IME interaction, memory-forensics coverage, full secondary tools or saved split restoration. The rewrite remains pre-release.
