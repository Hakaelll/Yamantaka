# Document connections checkpoint

Details now opens a native Connections manager for the selected document. It supports Related, Replaces, Source of, and Attachment of, with incoming labels that preserve direction. Open document uses canonical reader opening; View connections resolves the other document through a scoped lookup before changing the manager context. Removal has an inline review and preserves both documents.

The manager uses typed domain queries and changes, the application actor, and the existing v11 relation table. Queries return at most 80 rows with independent total and offset. Titles prefer manual overrides. Existing source and destination anchors are read without rewriting their raw values. Whole-document creation is idempotent for a source, destination, and type. It rejects self-links and missing catalogue identities. Removal requires a matching complete stored snapshot and an owning document, so a changed anchor or relation must be reloaded first.

Mutations use an immediate SQLite transaction, check cancellation before commit, and refresh source relation FTS entries in the same transaction. An unindexed document stays out of search when indexing is disabled. Index exclusions suppress both excluded sources and excluded destination titles. The new helper updates only the changed source's relation entries.

The manager shares the paginated catalogue chooser with PDF comparison. It retains debounced search, scoped cancellation, and stale-result rejection. Reference selection still disables unavailable PDFs; relation selection allows missing files but disables the source itself. The moved chooser has one implementation. The relation list is an immutable Arc snapshot, so frames do not clone up to 80 records. Narrow layouts stack controls and wrap text inside a constrained content width.

## Evidence

One retained storage compatibility test was extended through support/relations.rs. It checks self-link rejection, duplicate creation, incoming direction, historical anchor preservation, opt-in FTS updates, cancellation, failure rollback, changed-snapshot rejection, deletion from both views, search invalidation, excluded destination privacy, and SQLite integrity. No standalone tests were added. The focused compatibility test passed.

The final native executable compiled offline, formatting passed, and Clippy for the application binary and its dependencies passed with warnings denied. Six existing style warnings in reader helpers were simplified without changing their behavior.

Four focused scenarios run through tool/capture_rust_ui.ps1:

- relations-backlinks creates all four relation kinds from demo-0 to demo-1, retries the existing Related connection without duplication, removes that connection, and follows the manager context to demo-1. The final list contains three incoming connections from demo-0, including Replaced by. A separate inherited anchored relation remains preserved during the sequence.
- relations-review-es follows creation and duplicate admission, then leaves removal at confirmation. All five relation records remain present. The Spanish light-theme capture checks Confirm removal and Cancel.
- relations-review-small-200 repeats that review in a requested 640 by 480 window, Spanish, light theme, 200% text scaling, and reduced motion. The scenario scrolls to the confirmation actions and checks their bounds.
- reader-reference-chooser retains the existing comparison catalogue-chooser scenario after extracting the shared selector.

The [capture manifest](../screenshots/rust-document-connections/capture-manifest.json) records exact Rust source, executable, image, and control hashes. All scenarios use isolated synthetic records and the native runtime. Draft values and manager actions are driven programmatically, so these captures do not establish physical pointer or keyboard acceptance. No Computer Use or personal library data were used. No broad workspace suite was run.

## Remaining acceptance

Anchor creation and anchor-specific navigation, revision groups, automation rules, journals, and undo remain pending. Additional evidence is needed for large relation lists, the relation-specific chooser flow, opening a linked PDF from its row, active-write closure, rapid context changes, FTS refresh after related-title edits, memory pressure, physical DPI and Narrator. Full performance comparison and Release packaging remain pending. This checkpoint does not establish full advanced-organization parity or complete rewrite acceptance.
