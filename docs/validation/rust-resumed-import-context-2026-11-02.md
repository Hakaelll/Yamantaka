# Resumed import rule context

Resuming a native import now retains the documents that job previously imported, adds newly committed members, and excludes files that were merely already registered. It preserves existing document IDs and skips rendering/importing completed paths as before.

The separate import-recovery database migrates transactionally from version 1 to version 2. Its new nullable provenance field distinguishes imported members, pre-existing registered paths and legacy unknown origin. The main library stays at schema v11. Unknown future recovery versions remain unchanged and unsupported.

The document insert transaction now records a versioned native import-member receipt tied to its job, identity, normalized path, fingerprint and page count. This bridges interruption after the main database commit but before its auxiliary checkpoint. Registered records from another job do not gain membership. The runtime verifies ownership before creating checkpoints or adding newly processed IDs to the batch. The completed rule context is persisted before removing the recovery intent, so losing the intent cannot precede context publication.

For older version-1 checkpoints whose provenance cannot be established, resume retains the documents but offers no partial rule batch. It does not guess whether a record originally existed. The historical receipt is retained in the library journal and is not an automation action or a mutable user-visible rule execution.

## Executed evidence

- All four retained application tests passed. The existing source-hash verification now checks ownership recovered without an auxiliary checkpoint and rejects a different job. No standalone Rust test was added.
- The existing folder recovery verification checks all three original batch IDs after reopen/resume, mixed imported versus registered membership, unchanged catalogue IDs, manual edits, discarded intents and clean memory release.
- Its subsequent focused run additionally passed version-1 sidecar migration, unknown-origin recovery without a partial batch, main schema v11 preservation and byte-preserved rejection of a future sidecar version.
- Workspace Clippy across all targets passed with warnings denied; native debug build and formatting passed.
- The native synthetic resumed case completes an intent with one prior member and one new PDF. Its stored rule context contains both members and excludes the other nine catalogue records. The recovery intent finishes, the sidecar is version 2, and the library remains v11.

[Native status](artifacts/resumed-import-context.json) · [Resumed library](../screenshots/rust-resumed-import-context/resumed.png)

The native fixture explicitly seeds a prior checkpoint; it does not kill a running process. The main-commit/checkpoint gap is exercised through the real repository and a missing auxiliary checkpoint. Physical Resume interaction, abrupt native-process termination at each write boundary and large resumed-import performance remain pending. This checkpoint does not establish final Release acceptance.
