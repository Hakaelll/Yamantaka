# Recover a library that cannot open

If Yamantaka cannot read the selected Rust library or its saved preferences, it opens a recovery window before starting the PDF engine. Unsupported preferences versions also use this path. Saved preferences are validated before opening the database; Yamantaka does not silently replace unreadable settings with defaults.

Choose **Choose JSON backup…** to select an existing Yamantaka backup. Review the file name and book, annotation and bookmark counts. Reviewing a copy does not install it. You can choose another file or select **Exit without restoring**.

Choose **Restore and close** to confirm. Yamantaka revalidates the reviewed backup, closes the recovery window and installs a staged library. The entire old Rust store remains in a separate `pre-restore-<identity>` folder, including unreadable preferences or a damaged database. The completion notice identifies that safety folder. Reopen Yamantaka to use the restored library. Original PDF files are not changed.

Closing the window during review or preparation cancels that work. Once confirmed preparation hands over to installation, the existing transactional restore journal and rollback rules apply. If installation fails, Yamantaka reports the failure rather than claiming success. Keep the old store, backup and restore journal for recovery.

The recovery window uses Spanish by default because saved language settings may be unreadable. Launch with `--en` for English. `--software` selects the existing Glow renderer if the GPU cannot run the default DX12 path. These options do not repair a damaged store.

This window handles opening failures in the selected database and preferences. A failure while selecting an imported store or recovering an interrupted restore journal still produces a startup error. Repair of those records and unknown startup markers remains pending. Without a usable JSON backup, exit and retain the files for separate recovery.
