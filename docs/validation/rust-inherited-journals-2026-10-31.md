# Preserved Flutter automation history checkpoint

Date: 2026-10-31. Scope: displaying inherited automation history and validating a linked rule before a new simulation. Applying inherited undo receipts remains required work. This checkpoint does not establish complete legacy recovery or rewrite acceptance.

## Baseline and implementation

Flutter's connected app_shell recovery actions distinguish retry from rollback. Its retry path requires a retry/resume capability, a surviving rule, and an exact compatibility hash, then simulates the current library before confirmation. The hash is SHA-256 over UTF-8 JSON with ordered id, trigger, and actions fields. Trigger/action objects use their declared kind/value order. Rule name, enabled state, and timestamps are outside that hash. Its rollback path calls the old undo handler, whose receipts do not contain all the postconditions recorded by the native executor.

Native history now reads both rustAutomation.v1 roots and inherited automation roots in one transaction snapshot, with batches of up to 80. Internal native receipt items remain excluded. Each entry identifies its origin. Inherited records never receive the native reversible flag. Unknown undo payloads are not parsed or rewritten by this read path.

Simulate linked rule checks the complete inherited journal row signature and its timestamp, retry/resume capability, saved rule ID, known rule definition, and Flutter-compatible hash. It reads one linked rule using the same governed decoder as the saved-rule list. Hashing streams the typed payload instead of allocating a second large JSON document. Missing or changed rules, unsupported definitions, non-retry capabilities, malformed summaries, stale journal snapshots, cancellation, and memory admission failure return without modifying the journal.

A successful check starts the normal cancellable simulation using the returned rule revision. Any later application still requires reviewing the matches, preparing an immutable action plan, and confirming a new execution. The inherited journal is not silently marked recovered or resumed at an old offset. Leaving the manager cancels the pending check and ignores its late result. The journal's current linked rule name is used when no historical name was stored.

Old rollback receipts remain visible with a recovery-review explanation. Their preparation and application are not connected in this checkpoint. This preserves the distinction between inherited and native formats rather than passing old receipts into the native undo executor.

The native execution receipt-memory admission calculation also now includes retained rule ID/name data used by the root summary. This does not constitute full process-memory acceptance.

## Executed evidence

The retained application test `file_comparison::tests::background_comparison_rejects_partial_hash_collision_and_confirms_full_copy` was extended through its existing execution verification chain. The focused offline run passed; no standalone tests were added. The inherited-history fixture uses an independent temporary v11 store.

The checks cover a literal canonical payload with Unicode against a fixed SHA-256, noncanonical stored JSON field order/spacing, rule name and enabled-state changes that remain compatible, action changes that invalidate compatibility, stale journal changes without a timestamp update, missing rules, future trigger formats, wrong journal origin, non-retry capabilities, cancellation, memory admission and release, unchanged observer data version, byte-for-byte unknown undo preservation, 82-record pagination, exclusion of native child receipts, and SQLite integrity. The fixed payload follows the local Dart implementation; no fresh Dart/Flutter execution or installed-store migration was performed here.

The final binary built offline. Formatting and binary Clippy passed with warnings denied. Four native scenarios completed:

- `inherited-journals-retry`: a compatible inherited retry starts a fresh simulation with ten matching synthetic documents. History remains unchanged.
- `inherited-journals-incompatible`: a mismatched compatibility proof reports an error and leaves the simulation idle, with the preserved history still visible. English light theme.
- `inherited-journals-small-200`: three preserved records, including an unknown undo payload, remain readable in Spanish light theme, reduced motion, a requested 640 × 480 window, and 200% text scale. The sequence scrolls to Simulate linked rule and verifies its bounds.
- `inherited-journals-paged`: an 82-record inherited history retrieves offset 80 as a two-record batch while retaining the total.

The [capture manifest](../screenshots/rust-inherited-journals/capture-manifest.json) records executable, Rust source, Cargo.lock, image, and control hashes. Native sequences invoke controllers programmatically through the real runtime and isolated SQLite stores. Only synthetic data was used. No Computer Use, personal library data, broad workspace suite, or release packaging was used.

## Remaining work

Inherited receipt review/application, unknown-format recovery, interrupted recovery writes, explicit linkage of a completed retry to its old record, import-triggered context, physical interaction and late-result checks, real Flutter migration/export fixtures, paired performance measurements, Narrator/DPI, and Release packaging remain pending. A linked simulation does not satisfy those recovery or whole-rewrite requirements.
