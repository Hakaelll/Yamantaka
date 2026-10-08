# Incremental search-controller validation — 2026-10-09

The controller now has four integration tests against a real schema-v11 SQLite database and a deliberately deferred native engine. Its synchronous PDF methods panic, so the tests cannot accidentally establish behavior through blocking PDF calls.

Verified boundaries:

- Disabled indexing rejects start without creating index rows. Pending native work registers readiness and does not busy-spin. Pause prevents new page admission while allowing an already-admitted boundary to complete.
- Cancellation invalidates a late text result. Previously committed text and its durable cursor survive; a fresh controller resumes the next page without duplicate earlier requests.
- Exclusion invalidates pending text, removes searchable document content and remains effective for a new controller.
- Native failures persist a redacted code, close the temporary handle and complete the run. Shutdown cancels pending text and returns a ready close ticket without requiring any UI drain.

`cargo test -p yamantaka-application --offline`: 11 passed (seven runtime tests plus these four controller tests). `cargo fmt --all --check` and Clippy for application/all targets with `-D warnings`: passed.

This verifies the controller boundary, not its as-yet-unconnected settings or search UI. Active reader slot protection, actor integration, live-text accounting and a native end-to-end indexing smoke remain pending in the acceptance matrix.

## Public checkpoint review

The previously pushed `fec9e21` checkpoint was reviewed as 206 changed files. Inspection covered textual files, decoded JSON strings, complete PNG text metadata and unexpected document/database/archive/native-binary extensions. No recognized credential tokens, private-key blocks, local user paths or file-embedded email addresses were found. The screenshots have synthetic capture provenance. The project data, editor settings and generated third-party lockfile were not added.

`tool/check_public_checkpoint.py` reviews exact staged Git blobs or changes from a specified base. It reports finding categories and filenames without echoing matched private values. Five parser tests cover escaped paths, contacts/credentials, metadata after image data, malformed/unexpected metadata and restricted/generated artifacts. The guard complements human review; it cannot certify that arbitrary prose or image pixels contain no personal information.

The user explicitly requested retaining their configured Git identity and email. The guard's optional author check compares HEAD with that configuration without printing the address. Commit metadata is not rewritten.
