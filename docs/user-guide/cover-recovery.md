# Regenerate a missing cover

Select the book, open **Details → More actions**, and choose **Regenerate cover**. Yamantaka rebuilds the local image using the saved cover page. An automatic decision that needs refreshing uses the existing cover detector. Title, author, annotations and the original PDF are not edited.

For a protected PDF, enter its password in the unlock dialog and choose **Rebuild cover**. The password is temporary. An incorrect password lets you retry. **Cancel rebuild** stops pending work; a cache write that already completed can remain.

A successful rebuild refreshes the book's cover. If the saved page or file changes while the task runs, retry using its current state. If two readings are active, close one and retry. Only one explicit rebuild runs at a time.

This also repairs a deleted or damaged local cover cache. Choosing another source page remains a separate **Choose cover** action in the same menu. Storage cleanup removes cached images while retaining cover decisions.

See [protected imports](protected-imports.md) for password behavior and [the validation checkpoint](../validation/rust-cover-rebuild-2026-11-01.md) for executed cases and limits.

## Batch regeneration after review 10

The development source now requests a password when automatic cover regeneration
encounters a protected PDF. This uses the existing individual unlock and cache
rebuild flow. Choose **Regenerate cover** to continue or **Cancel batch** to stop
the remaining covers. Completed replacements stay saved. Passwords are temporary
and are not retained for the next document.

If native document slots become busy during a protected batch item, the batch
waits for availability. Close an unused reading to let it continue, or choose
**Cancel batch**. The completed source hash stays available while waiting. If
the temporary PDF owner must be reopened, its password may be requested again.

Batch regeneration replaces selected manual covers with newly detected automatic
choices only after its existing confirmation. The saved cover decision is checked
again before replacement; a later manual change is preserved and that item fails.
The protected path also verifies the complete source hash and native source before
saving. Ordinary unprotected books keep the existing path.

This change is not included in the published review-10 executable. Runtime,
password-retry, cancellation, capacity-pressure and stale-selection acceptance
remain pending; see [protected batch implementation](../architecture/rust-protected-cover-batch.md).
