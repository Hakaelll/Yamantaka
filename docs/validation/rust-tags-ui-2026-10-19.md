# Native document tags and filters

Tools → Tags opens a dedicated native tag manager. Organize tags in a book's
details opens the same manager with assignment checkboxes for that book. Rows
show the saved color, independent book/annotation counts, and separate controls
for assigning the tag and filtering the library. No permanent sidebar or new
full-width library bar is introduced.

## Creation and assignment

Create tag makes or reuses a normalized tag. In a document context, Create and
assign performs both operations in one immediate transaction. If the catalogue
document disappears, a newly created tag is rolled back. Existing tag IDs,
colors and annotation memberships retain the shared storage semantics described
in [tag storage verification](rust-tag-storage-2026-10-19.md).

After creation or reuse, the manager resolves that exact tag ID and shows it,
even if its name sorts beyond the first 80 rows. An explicit Show all tags action
returns to the full list. Typing a new search also clears this temporary scope.
Queries and mutations run through typed actor actions. The actor determines
index access from its current controller state; the UI passes no SQL or PDFium
handle. Superseded reads are cancelled and mismatched replies are ignored.
Matching write replies remain accepted after leaving the manager, keeping a
committed change and the catalogue consistent. Drafts/errors survive failures.

## Combined filters and restoration

Several selected tags match books with any selected tag. The existing SQL
predicate still combines them with collection, reading/file state, smart view,
favorite and text filters. Selection is bounded to 80 unique IDs. The existing
filter row shows one tag's name or Tags (N), with clear and reset actions; compact
windows retain the existing filter popover. All books also clears tag filters.

Versioned atomic preferences save the ID/name pairs when selection changes,
without serializing them on every frame. Startup resolves the saved IDs in one
bounded actor request, refreshes labels and removes absent IDs. Inspection failure
preserves the saved selection. The old single-ID query field is replaced by the
bounded ID set and all callers are migrated.

## Executed verification

The retained catalogue-query test passes with multi-ID lookup past page one,
unknown IDs, the 80-ID bound, atomic creation/assignment rollback, two distinct
books matching different selected tags, and no duplicate book when several
selected tags match it. Existing shared-tag/FTS checks still run in that case.
Cargo formatting, workspace all-target Clippy with warnings denied and the
offline native development build pass. No test case is added; the reduced suite
remains at 54 cases. The full suite and performance benchmarks are not rerun.

[Native synthetic captures](../screenshots/rust-tags-ui) reuse a saved violet Study
tag already attached to an annotation, assign it to demo-0, create Reference and
assign it to demo-1. The manager shows two tags, one assignment for the current
book and one annotation use. The filter scenario saves and reloads an isolated
preferences file with outdated labels, invokes normal ID resolution and returns
two books under the restored Study/Reference filters. This is a one-process
preferences round trip, not a full application restart test.

Small-window verification exposed a form-width bug in Tags and Collections:
nested enabled controls could extend a wrapped row beyond the viewport. Both
managers now clamp to available width and stack form/paging controls when narrow.
The capture helper records native tag inputs and checkboxes, with tag-specific
accessible names. The compact tag and collection captures verify recorded
horizontal bounds and selected enabled controls inside their clips. A deliberately
clipped copy is rejected by the same checker. Future manager captures automatically
invoke `tool/check_rust_control_bounds.ps1` to retain this check.

Dark, Spanish light/reduced motion and 200% text are represented. Source and
executable hashes match the manifest, and public artifacts contain only synthetic
fixtures. The scenarios invoke application control methods; physical keyboard,
mouse, Narrator and monitor/DPI acceptance remain pending.

## Remaining parity

Tag rename/merge/delete/color editing, multi-document operations and broader
organization/journal acceptance remain incomplete. Full restart/crash, large-index
write latency, complete memory/performance accounting and the final Windows
release gates remain pending. This checkpoint is not the completed rewrite.
