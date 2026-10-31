# Recover a blocked reading

Open **Tools → Document recovery**. The list shows books whose automatic restoration is blocked after an opening failure, including blocks preserved from Flutter. Each page contains at most 80 records.

Quarantine is a saved library flag. Yamantaka does not move, modify or delete the PDF, its annotations, progress or organization data. Existing unknown failure codes remain stored until you explicitly remove the block or successfully retry the reading. The interface displays a general explanation instead of raw diagnostic text.

Choose **Retry reading** to make an explicit native opening attempt. If the reader accepts the document, its old opening-failure block is cleared. A PDF opening failure keeps it blocked. Password challenges, cancellation, queue pressure and the governed memory limit do not create a PDF-failure block. If the catalogue marks the file missing, locate it from the book's details before retrying.

Choose **Review block** to allow automatic restoration again without opening the PDF. The review identifies the book and explains that removing a block does not repair its contents. **Remove block** checks that the registered source, missing state and stored failure have not changed since review. If they changed, refresh the list and review again.

Blocked books are skipped by saved-tab lookup, workspace reference restoration and new automatic cover requests. Their procedural 3D representation remains available. Removing a block or successfully retrying resumes cover work; unavailable covers can regenerate when the library is visible. Existing valid cover images can remain displayed while work is suspended.

Closing the dialog cancels pending reads. A change you already submitted may still finish. Closing the application waits for tracked quarantine writes. If saving the state fails, the application reports the failure rather than silently treating the change as saved.

This flow handles reported reader-opening failures. It does not repair malformed PDFs or provide process isolation for native crashes. See [safe startup](safe-startup.md) and the [validation checkpoint](../validation/rust-document-quarantine-2026-10-31.md) for the remaining recovery limits.
