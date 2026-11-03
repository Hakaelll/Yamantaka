# Return to a previous library

Open **Settings → Storage → Previous library**. The folder picker opens beside the active Rust library. Select one of Yamantaka's `pre-restore-…` safety folders, review its filename and record counts, then choose **Return and close**. **Cancel** leaves both libraries unchanged.

Close other instances using the selected copy before confirming. A copy with pending SQLite WAL data is rejected, so that uncheckpointed data cannot be silently ignored. Do not delete WAL files to bypass this check.

The entire selected folder becomes the active library. Its former safety-folder path disappears. Your current library moves into a new `pre-restore-…` folder, preserving its database, preferences, sessions, journals, auxiliary files and unknown resources. PDF contents are unchanged. Reopen Yamantaka when the operation finishes.

To return again, select the new safety folder through the same action. The library you are leaving will receive its own safety folder. This is a directory exchange, rather than an immutable copy of the selected safety folder.

The same **Previous copy** action is available on the startup recovery screen when saved preferences or the library cannot be opened. Use **Restore JSON backup** for a JSON export instead. Flutter stores continue to use the independent import assistant.

If a selected copy changes after review, Yamantaka refuses to install it. An interrupted exchange is checked before the active library opens. Uncertain combinations of folders or journals are preserved and stop automatic recovery; this flow does not guess which changed folder to delete.

See [validation and limits](../validation/rust-store-rollback-2026-11-02.md).

## Several interrupted operation records

Startup recovery can review the current library or a previous copy even when
several operation records remain. The selected source and its book, annotation
and bookmark counts appear before the explanatory text. Confirm the intended
library with **Confirm and close**, then reopen Yamantaka.

Confirmation preserves all pending records without using their stored paths to
choose a library. Keeping the current library leaves its files in place. Choosing
a previous copy retains the whole current library as a safety folder. If record
preservation is interrupted, startup asks for review again before replaying any
remaining record. See [multiple-record recovery](../validation/rust-multiple-pending-records-2026-11-03.md)
for implementation scope and remaining acceptance.
