# Add a password-protected PDF

Choose **Add PDF**, drop PDF files into the library, or import a folder. When a document needs a password, Yamantaka shows its filename in the existing unlock dialog. Enter the password and choose **Add PDF**. An incorrect password returns the same dialog with a retry message.

The password is used for that opening only. It is not saved in the catalogue, preferences, import journal, backup or session. If the native document must reopen after an interruption, Yamantaka asks again. Resuming an import after restarting the app also requires the password again.

**Cancel import** stops the current import batch. Documents already confirmed remain saved. The unfinished job stays in **Pending imports**, where **Resume** continues it and **Discard** forgets the job without deleting confirmed books. Closing the unlock dialog cancels the same batch.

The import keeps the original PDF unchanged and uses the existing metadata analysis and full-content identity checks. Adding a protected document does not remove its protection. Opening it for reading can require its password again. Background features that need a closed encrypted PDF still cannot reopen it without another explicit unlock; protected-cover and full-text-index acceptance remain incomplete.

Executed scenarios and limits are recorded in the [protected import checkpoint](../validation/rust-protected-import-2026-10-31.md).
