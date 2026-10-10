# Atomic creation of annotation export files

The storage layer now stages an annotation export in a temporary file inside the destination directory. It publishes the destination only after the application writer completes, the output is flushed, file data is synchronized and cancellation is checked again. Publication uses the pinned tempfile implementation, which calls Windows MoveFileExW without the replacement flag for a new destination.

Existing destinations are rejected before the writer runs. Publication also refuses a destination created by another process during writing, without changing that process's bytes. Writer errors and cancellation drop the temporary file and leave no published output. The publication operation is the commit point; cancellation that arrives after publication does not undo a successful export.

The boundary accepts Markdown and JSON extensions, requires an existing parent directory and rejects registered document paths even when their files are missing. The guard recognizes Rust's canonical Windows path form and Flutter's ordinary path form. It never opens an existing destination for writing. File-system and database errors use a fixed message without local paths.

## Verification

Four storage regressions cover:

- Unicode file names and content, with no destination visible until the callback completes.
- Partial writer failure, cancellation during writing and cancellation before entry, including temporary-file cleanup.
- An occupied destination and a destination created during the callback, with exact original-byte preservation.
- A missing registered source stored in Flutter's path form, an unsupported extension and an absent parent, all rejected before writing without exposing the temporary directory in errors.

Two application regressions connect the existing coherent SQLite visitor and incremental writer to the new file publication boundary. Both JSON and Markdown publish all 181 records. Parsed JSON includes the final record, and Markdown contains all notes and the final page anchor. An invalid timestamp in the final record fails after earlier rows have streamed, leaving no export or temporary file and no retained governed row memory.

Formatting, warning-denied workspace/all-target Clippy and the full locked workspace test suite pass. The local test log is `build/verification/annotation-export-publication-tests.log`. The only dependency change adds the already pinned tempfile package to application dev-dependencies for the file integration tests.

## Remaining scope

This is the create-new publication path. Actor routing, native save dialog, explicit replacement confirmation and safe replacement policy, progress/error presentation and native export interaction acceptance remain pending. Process termination may leave an unpublished temporary file; crash cleanup and power-loss durability have not been validated. Network shares, junction aliases, permission failures and real disk exhaustion also need Windows acceptance. Large-export latency, intra-row cancellation and full output/SQLite resource accounting remain pending. No complete export UI or final release parity is claimed.
