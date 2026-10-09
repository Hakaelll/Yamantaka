# Visible native reader protection — 2026-10-09

The PDF scheduler now accepts an atomic set of up to two distinct visible document IDs. Changing this set only updates bounded scheduler metadata and wakes the worker; it does not wait for PDFium. The current single-reader application sets its visible ID before requesting an open.

Background opens cannot evict a protected handle. When both native slots are protected, admission fails immediately without changing document epochs or retaining a request reservation. Previously admitted opens check protection again before eviction. Victim removal and pin changes share the scheduler lock; native handle destruction occurs outside that lock. Explicit close removes its pin atomically with the epoch check and handle removal. Reopening an existing protected document remains possible.

## Executed validation

- `cargo test --workspace --locked`: 105 tests passed, including native PDFium, SQLite and Windows process tests.
- After tightening the close critical section, `cargo test -p yamantaka-pdf --locked`: all 14 PDF tests passed again.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed on the final code.
- `cargo fmt --all -- --check`: passed on the final code.

The new real PDFium test opens an older visible reader and repeatedly replaces its background companion. It verifies that the reader remains usable and the evicted background handle becomes unavailable. It then protects both documents, rejects a third open, reopens a protected ID, renders both readers and verifies that explicit close frees the background slot.

The new scheduler test rejects malformed pin sets and 256 inadmissible background opens, checking that the original pins, queue, epochs and reservation count remain intact. Clearing the visible set permits a new admission.

All native documents in these tests are generated synthetic fixtures. No personal PDF, database, development output or local editor settings belongs to this checkpoint. The existing configured Git identity is retained as requested.

## Scope

This is a verified scheduler and single-reader integration checkpoint. It does not complete the rewrite. Search-controller integration still needs explicit coordination when two readers occupy the native slots, and the comparison workspace and remaining parity requirements are pending. No release or complete performance claim is made here.
