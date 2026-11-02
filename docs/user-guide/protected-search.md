# Search inside a password-protected PDF

Open **Local search**, enable local search, then choose **Index library**. Indexing is optional. When a protected PDF needs unlocking, enter its password and choose **Index PDF**. An incorrect password shows a retry message.

The password is held temporarily for that opening. It is not saved in the catalogue, index, preferences, backup or session. If the native document is unloaded to make room for reading, indexing keeps its committed page cursor and asks again when it can reopen the PDF. Opening it for reading remains a separate operation.

**Indexing stores the extracted text unencrypted in the local index.** PDF password protection does not encrypt that copy. The unlock dialog explains this before you submit the password. Use **Manage index → Clear index** to erase indexed text, or exclude the document to remove it from search. Disabling search stops indexing and keeps the existing index.

**Cancel indexing**, closing the password dialog, or **Stop** ends the current indexing run. Pages already committed remain available. Choose **Index library** again to resume unfinished work without rebuilding those pages. A new opening requires the password again. A rebuild clears progress while preserving document exclusions.

The original PDF stays unchanged. Source identity and page geometry are checked before newly opened native documents supply text. The inherited fingerprint is a bounded identity check, not a full-content integrity guarantee.

See the [validation record](../validation/rust-protected-search-2026-11-02.md) for executed cases and remaining acceptance work.
