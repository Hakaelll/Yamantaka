# Regenerate a missing cover

Select the book, open **Details → More actions**, and choose **Regenerate cover**. Yamantaka rebuilds the local image using the saved cover page. An automatic decision that needs refreshing uses the existing cover detector. Title, author, annotations and the original PDF are not edited.

For a protected PDF, enter its password in the unlock dialog and choose **Rebuild cover**. The password is temporary. An incorrect password lets you retry. **Cancel rebuild** stops pending work; a cache write that already completed can remain.

A successful rebuild refreshes the book's cover. If the saved page or file changes while the task runs, retry using its current state. If two readings are active, close one and retry. Only one explicit rebuild runs at a time.

This also repairs a deleted or damaged local cover cache. Choosing another source page remains a separate **Choose cover** action in the same menu. Storage cleanup removes cached images while retaining cover decisions.

See [protected imports](protected-imports.md) for password behavior and [the validation checkpoint](../validation/rust-cover-rebuild-2026-11-01.md) for executed cases and limits.
