# Find identical copies

Open **Tools → Identical copies**, then choose **Find copies**. Yamantaka checks candidate PDF files in the background. You can continue using the application or choose **Cancel search**. Leaving this view cancels the search and releases its results.

Choose **Review group** to see the catalogue entries and locations of identical copies. **Open reading** opens an entry through the normal reader. Results describe the files when the search ran. Run the search again after changing files or library records.

The search narrows candidates by their inherited fingerprint and size, then compares freshly calculated complete SHA-256 hashes. Matching filenames or a matching partial fingerprint alone cannot establish identical contents. Missing, changed or inaccessible sources appear in the unverified count. A failed search does not present incomplete groups as a completed result.

## Consolidate library records

Select the entry to keep, then choose **Review consolidation** for another member of its group. Close readings of both copies from Tools first. Finish pending changes and resolve recovery drafts before continuing; saved reading positions are retained.

The review identifies both records, shows affected annotations, bookmarks, sessions, collections and workspace readings, and lets you choose between conflicting titles, authors and library notes. The kept entry retains its source identity. Existing kept bookmarks and workspace readings win when entries overlap; manual covers take priority. Choose **Cancel** to leave the records separate or **Consolidate** to confirm the reviewed choices.

Yamantaka rechecks the records and complete source hashes, creates a backup, then combines library records in a transaction. Background work can temporarily prevent confirmation: finish or cancel that work and confirm again. The original PDF files are never moved, deleted or rewritten.

After completion, **Undo consolidation** opens a hash-checked review of the backup. This restores the **entire library** to its state before consolidation, including any later library edits. Review the counts before choosing **Restore and close**. The current library is retained in a safety folder.

The immediate Undo action is available in the current application session. After restarting, open **Tools → Identical copies → Consolidation history**. The history reads compatible completed native consolidation receipts from disk, in pages of up to 80 entries. Choose **Review backup** beside a date to validate and review that backup before restoring it. Missing or modified backups cannot pass the recorded hash check.

**Previous**, **Next** and **Hide history** control the history list. Historical Flutter merge records without a compatible native hash receipt do not appear here. Do not treat either Undo or history restore as a selective reversal of one record.
