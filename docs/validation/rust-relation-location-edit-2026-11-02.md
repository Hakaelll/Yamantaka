# Edit existing relation locations

Existing connections now expose Edit location inside their document manager. The editor identifies the current book and saved anchor, then requires explicit replacement before enabling Save. It can select a page or the whole document. Cancel keeps the saved relation unchanged. Editing from the other document's manager changes that endpoint instead.

The typed UpdateLocation command carries the complete frozen relation record. Storage compares it inside an immediate transaction, verifies that the current book owns an endpoint, validates its current page count, and updates only that endpoint's two anchor columns. Relation identity, direction, type, creation time and the other anchor remain intact. The source document's relation FTS refresh remains in the same transaction. Cancellation is checked before commit. Reload and reopening the editor recover from stale-record rejection.

## Executed evidence

The retained compatibility test passed after extending its existing relation helper. It checks relation identity and creation time, source and destination edits, opposite-anchor preservation, stale snapshot rejection, an out-of-range page and a return to whole-document navigation. No new standalone test was added.

Workspace Clippy across all targets passed with warnings denied. The native debug executable built and formatting passed. Subsequent UI-only preview changes also passed UI-package Clippy across all targets.

Five synthetic native scenarios cover the normal editor, 640 × 480 with 200% text, saved source location, saved backlink destination and cancelled editor. Stored snapshot assertions verify the selected endpoint and unchanged opposite location. The two editor cases require fully visible Cancel edit and Save location controls. Production starts with replacement disabled; the capture fixture explicitly enables it before checking the enabled Save control. An earlier capture check correctly rejected the intentionally disabled Save button, then the fixture was corrected without changing that production safeguard.

[Editor](../screenshots/rust-relation-location-edit/relation-edit-form.png) · [Enlarged text](../screenshots/rust-relation-location-edit/relation-edit-small.png) · [Saved](../screenshots/rust-relation-location-edit/relation-edit-applied.png) · [Backlink](../screenshots/rust-relation-location-edit/relation-edit-backlink.png) · [Cancelled](../screenshots/rust-relation-location-edit/relation-edit-cancelled.png)

The prior frozen executable supplies the [before view](../screenshots/rust-relation-location-edit/before.png), using the existing page-location fixture. The baseline form has its new-connection fields populated; the new scenario opens the existing relation editor instead. They share synthetic records, viewport, light theme and Spanish locale; they are not identical interaction states. Native image hashes describe original captures. Published PNGs only add synthetic provenance text, with both hashes in their origin files.

Saving scenarios drive the typed native action after fixture setup; they do not establish physical pointer or keyboard activation of the Save button. Broader interaction, Narrator, physical DPI and large-library latency remain pending. Existing bookmark/annotation anchors can be explicitly replaced, but creating new anchors of those kinds is still outside this form. No final Release acceptance is claimed.
