# Reader continuity storage checkpoint

Date: 2026-10-11. This checkpoint implements and verifies the domain/storage contract. Automatic startup restoration and background UI persistence are not connected by this change.

## Persisted state

The version-one Rust JSON snapshot stores up to twelve open readings, their active tab identity, and up to ten closed readings in closure order. Each reading retains its independent document/tab identity, page, pin, panel and complete reader preferences. Two tabs for the same PDF remain independent. Native resource classifications are recomputed from the restored open readings; native handles, passwords and render caches are not serialized.

The adapter preserves the accepted legacy `spread` layout spelling and unknown extension fields at the root, tab and preference levels, including when a closed tab is reopened. It reads an existing snapshot before replacement. Unsupported versions, invalid contexts, duplicate identities, unknown panels and an absent active identity fail without overwriting the existing file. Missing files return no snapshot.

Input is limited to four MiB. Output is checked by a bounded counting writer before the atomic JSON helper creates or replaces a file. The helper writes to a temporary file in the same directory, flushes it and atomically replaces the destination. This is a serialized single-writer storage API, not a concurrent merge protocol. Its input limit does not prove compliance with the application's global CPU/GPU memory governor.

Complete domain restoration validates a candidate before changing the current model. Reusing a previously closed tab identity removes its stale closure only after successful opening or workspace restoration, preventing a later reopen from targeting an already open identity.

## Legacy boundary

Flutter's `recently_closed_documents_v1` metadata deduplicates by document identity and contains only a subset of reader preferences. Its `atomic_session_snapshot_v1` is a separate recovery record. Neither key is modified by this Rust JSON adapter. Importing these records with recovery reporting, connecting persistence to the actor and shutdown barrier, and exposing startup recovery remain required work.

## Verification

Five new regressions cover full-context round trips and reopening, unknown extension preservation across closure/open transitions, non-destructive rejection of incompatible snapshots, input/output limits, and atomic domain restoration with identity reuse. Tests use temporary synthetic stores and Unicode document identifiers.

`cargo test --workspace --locked` passed all 404 tests, including the five new regressions. `cargo clippy --workspace --all-targets --locked -- -D warnings` passed. Formatting was applied with `cargo fmt --all` and checked separately. There are no new native captures because this change adds no presentation controls and does not connect startup restoration. The previously opened demonstration window remains available for visual inspection.

## Remaining acceptance

Connect the storage contract to scoped application commands and the existing persistence barrier. Verify successful and interrupted restarts, safe mode, restore-disabled settings, missing/modified PDFs, failed writes and password reopening. Preserve malformed imported legacy entries with an explicit recovery outcome. Verify workspace, queue, native warm-resource ownership, physical interaction and accessibility acceptance before claiming complete continuity.
