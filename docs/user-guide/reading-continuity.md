# Resume a reading comparison

Enable **Restore last session** in Settings to reopen your readings on the next ordinary startup. An active comparison returns after the main PDF opens, including its reference page, zoom, appearance, divider and following options.

If the reference PDF is missing, the main reading still opens. Yamantaka keeps the reference configuration so a later restart can recover it after the file becomes available. Closing the reference explicitly discards its saved comparison configuration. Removing a document from the library also removes its comparison context.

Disable **Restore last session** to start without reopening the saved readings. Recovery after a graphics failure still attempts to return to the interrupted reading session; it does not enable the preference permanently.

Passwords and rendered page images are never part of the saved session. A protected document may require its password again. Saved workspaces remain separate named reading arrangements.

See the [validation checkpoint](../validation/rust-comparison-continuity-2026-10-31.md) for the cases exercised and remaining acceptance work.

When you close the window with an unsaved reading note or manual title/author correction, choose **Return to editing** to keep working. **Discard and close** abandons the open draft. An accepted save finishes before closing. New page-note drafts in either reader are also saved locally with the session. They reopen as unsaved notes when the matching document is available, including after checked graphics recovery. Missing-reference drafts stay retained. Selection annotations and edits to existing notes still lack persistent recovery. A forced exit can lose edits made since the last background snapshot.

See [page-note recovery validation](../validation/rust-page-note-recovery-2026-10-31.md) for the supported draft fields and limits.

Unfinished title and author corrections also persist locally. With session restoration enabled, the metadata editor reopens. If the catalogue review changed, compare the recovered text with the current values and choose **Use recovered text** or **Keep current values**. Recovered text remains unsaved until you choose **Save changes**. See [metadata draft recovery](../validation/rust-metadata-draft-recovery-2026-10-31.md).
