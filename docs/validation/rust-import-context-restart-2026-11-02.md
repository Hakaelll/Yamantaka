# Durable completed-import context

The last completed native import now stores its ordered, deduplicated rule context in one versioned `rustImportContext.v1` operation-journal record. This preserves the main SQLite v11 schema and includes the context in normal library backups and restores. No rule action runs automatically.

Starting a new import clears the previous supported context before new document processing. Completion persists the full context before publishing its immutable UI snapshot. Reanalysis does not replace it. Failed or cancelled imports do not publish a completed partial batch. Startup loads the context through a typed actor command; stale load replies cannot replace a batch started later. Forget batch removes supported persisted context through the actor before updating the available UI batch. Reviewed previews retain their own immutable scope.

Persistence bounds the JSON snapshot to eight MiB, limits identities to 100,000 and validates individual IDs. Temporary serialization/deserialization uses a shared memory-governor reservation, and loaded identities retain their existing governed leases. Incompatible journal records are neither overwritten nor removed by the native context writer. Malformed supported payloads fail loading without being replaced.

## Executed evidence

The retained application verification checks a persisted batch through a separate read-only connection, exact sorted identity recovery, durable forgetting, rejection of malformed context, preservation of a future incompatible journal and memory release. No standalone Rust test was added. Workspace Clippy across all targets, native debug build and formatting passed.

`tool/verify_import_rule_restart.py` runs two native processes. The first imports three synthetic PDFs beside ten pre-existing records and simulates only those three imported IDs. The verifier compares the committed journal with that exact scope. A separate process resumes the same library, loads its persisted context and simulates the same three IDs without another import. Both native processes must exit successfully; no rule action is executed.

[Status report](artifacts/import-context-restart.json) · [Completed import](../screenshots/rust-import-context-restart/completed-import.png) · [Restarted simulation](../screenshots/rust-import-context-restart/restarted.png)

Published images contain only generated PDFs and synthetic records. PNG provenance is added without changing pixels; origin files record original and published hashes. The earlier [session-local checkpoint](rust-import-rule-context-2026-10-31.md) retains its original acceptance scope.

## Remaining acceptance

A resumed import still collects records committed during that resumed run, not its earlier checkpointed records. Persisting the completed context does not reconstruct that full original import scope. Mid-write process termination, physical Forget-batch interaction, Narrator/DPI and large-context performance remain pending. The existing atomic SQLite statement protects context publication, but this checkpoint does not claim whole-rewrite or Release acceptance.

## Resumed-scope follow-up

[Resumed import context](rust-resumed-import-context-2026-11-02.md) later connects known prior checkpoint members and atomic main-commit ownership proofs. Older scope statements above describe their original checkpoints; unknown legacy provenance is preserved without inventing a partial rule scope.
