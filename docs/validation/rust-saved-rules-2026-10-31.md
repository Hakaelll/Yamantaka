# Saved automation rule management checkpoint

Tools now opens Saved rules. The native manager creates a one-action definition, displays inherited definitions with up to eight actions, changes their enabled flag, and reviews deletion inline. It lists 80 rows per batch and exposes further pages. A collapsed New rule form keeps the list visible; narrow layouts stack controls. The screen states that simulation and execution remain in development. This checkpoint manages definitions and does not apply their actions to documents.

Typed domain trigger and action enums use the persisted camelCase names. The reader keeps unknown or structurally unsupported definitions as unavailable records rather than falling back to a different trigger or action. Existing missing trigger values remain representable. Known long inherited values are admitted through governed memory instead of an arbitrary JSON or value-length cutoff. New names retain the Flutter 120 UTF-16-unit limit; the native entry fields accept up to 32,767 characters each.

Reads use an application-actor request and a SQLite read transaction. A conservative shared memory reservation precedes controlled metadata and definition allocation. Each immutable Arc record owns its lease, including a record retained for a deletion review or command. Budget failure releases the previous list and review snapshot so Reload can recover capacity. No SQL runs in the UI.

Create, enabled-state changes, and deletion use immediate transactions with cancellation before commit. Changes to existing definitions verify a length-prefixed SHA-256 snapshot of all seven v11 fields, including the original condition and action JSON. This detects external payload changes even when updated_at stays unchanged. Enabled-state writes update only enabled and updated_at, preserving raw payload bytes and created_at. Unknown formats can be disabled or reviewed for deletion, but cannot be newly enabled. Rule deletion leaves document and operation-journal tables untouched.

## Evidence

The retained storage compatibility test was extended through support/automation.rs. Its focused run passed. It checks cancelled and invalid creation, trimmed names, enabled-state changes, stale snapshots, payload changes without a timestamp update, failed deletion rollback, unknown-payload preservation through a guarding SQL trigger, unavailable-format enable rejection, cancelled deletion, memory admission failure and lease release, inherited eight-action definitions, known values exceeding 64 KiB, representable missing trigger values, unknown-field rejection without reinterpretation, 82-rule pagination, and SQLite integrity. No standalone tests were added.

The final application binary compiled offline. cargo fmt --all --check and Clippy for the binary and dependencies passed with warnings denied. Four native scenarios completed through tool/capture_rust_ui.ps1:

- saved-rules-legacy displays an inherited eight-action rule and retains an unknown future-format rule. The Spanish dark-theme capture checks New rule remains available.
- saved-rules-changed creates Reading PDFs, disables it, disables the future-format definition without replacing its payload, deletes only Reading PDFs, and retains both inherited definitions. The final capture uses English and light theme.
- saved-rules-review-small-200 stops deletion at review with all three definitions retained. Spanish, light theme, reduced motion, a requested 640 by 480 window, and 200% scaling exercise the compact layout. The sequence scrolls to Confirm deletion and Cancel and checks their bounds.
- saved-rules-paged loads 82 definitions, requests the next list batch, and verifies offset 80 returns two records while retaining the total.

The [capture manifest](../screenshots/rust-saved-rules/capture-manifest.json) records executable, Rust source, image, and control hashes. They were checked against the final working tree and binary before publication. The native sequences assign draft values and invoke controller actions programmatically, using the real runtime and SQLite. They do not certify physical keyboard/pointer interaction or real Flutter installations. All records and documents are synthetic. No Computer Use or personal library data were used. No broad workspace suite was run.

## Remaining acceptance

Matching and simulation, frozen reviewed plans, confirmed application, execution cancellation, journal recovery, and undo remain required native work. Exact-duplicate matching must use verified full content identity rather than the legacy partial fingerprint. Metadata updates here do not establish any execution behavior. Physical editor/selector interaction, full restart, pending-write closure, rapid context changes, memory-pressure UI recovery, physical DPI, Narrator, full performance comparison, and Release packaging remain pending. This checkpoint does not establish automation parity or complete rewrite acceptance.
