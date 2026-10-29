# Reference bookmarks checkpoint

The reference navigator now includes Bookmarks. Both panes share the existing bookmark editor, list, pagination, retry feedback and canonical page navigation. Saves, label changes and removals target the owning document through the application worker and existing SQLite v11 repository. Lists remain bounded to 80 entries per page.

The worker reports successful bookmark writes separately from reads. A committed change invalidates other list state for that document, including a pane with its own write in flight. Refresh preserves the current list offset and existing rows until the replacement arrives. Document and request identity reject stale replies. Closing the reference cancels list queries while accepted writes continue; errors without an owning editor reach the application's existing error surface. Library results refresh after a successful change.

Narrow reference popups use a section selector so adding Bookmarks does not add another persistent toolbar or a second row of section buttons.

## Evidence

The native binary compiled with `cargo build -p yamantaka --bin yamantaka-rust --offline`. Five focused native scenarios passed through `tool/capture_rust_ui.ps1`:

- `reader-reference-bookmarks`: create a page-2 bookmark in a distinct reference PDF, rename it, navigate away and back, remove it, then recreate and label it. The main reading stays on page 1 with its original two bookmarks.
- `reader-reference-bookmarks-small-200`: the same operations at a requested 640×480 window, 200% scaling, Spanish, light theme and reduced motion. The section selector and save-page control are fully visible.
- `reader-reference-bookmarks-shared`: submit writes from both panes of the same PDF, refresh the main list to observe the reference label, then remove a bookmark and immediately close the reference. The main list subsequently contains the committed remaining bookmark, proving the accepted removal survived pane closure.
- `bookmarks-dark`: the primary bookmark panel still displays its existing fixture through the shared view.
- `reader-reference-thumbnails-small-200`: thumbnail navigation remains functional through the new narrow section selector.

The three reference bookmark cases use isolated, file-backed synthetic SQLite stores. They exercise database reads and writes; they do not establish whole-application restart or crash recovery acceptance. The [native captures and manifest](../screenshots/rust-reference-bookmarks/capture-manifest.json) record source and executable hashes. No personal store or original PDF was modified. No standalone Rust tests or broad workspace suite were added or run for this checkpoint.

Other reference reader tools, per-pane shortcut routing, overlays, full performance and accessibility acceptance, and Release packaging remain pending.
