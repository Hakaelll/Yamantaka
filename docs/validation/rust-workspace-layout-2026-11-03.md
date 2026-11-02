# Workspaces restore panel width and list columns

Native workspaces now read, validate, save, and apply the existing v11 `panel_width` and `visible_columns_json` values. Previously these fields survived some storage operations but did not reach the native view. The schema and backup format are unchanged.

The reader restores the saved panel width when the panel becomes visible, replacing any earlier egui panel-size state. It retains mouse resizing and the narrow-window drawer. Settled changes save to native preferences after the pointer is released. Opening a workspace also applies its list-column choices. New native workspaces capture both values; explicitly updating a workspace records later changes. Queue/focus/comparison autosaving retains its existing scope.

The sorting menu now contains **List columns**, with optional author, reading state, progress, and page count. Titles and actions remain available. Hiding progress or page count returns horizontal space to document titles; compact rows also shorten when fields are hidden. Missing-file warnings remain visible. Column visibility does not change row selection, reader opening, document metadata, or the 3D cover renderer.

Unknown column keys remain in the saved list and survive an explicit workspace update. They have no native renderer until supported. Recognized fields retain their existing presentation order; this checkpoint does not add column reordering or custom column kinds.

## Executed evidence

- One retained storage compatibility test passed. Its workspace checks now include an inherited 360-pixel panel and document/author columns, an explicit update to 420 pixels, preservation of an unknown column key, and rejection of invalid widths or excessive column counts. No standalone test cases were added.
- The existing native workspace scenario created, replaced, duplicated, opened, updated, deleted, and restored saved workspaces. It deliberately changed the live width and columns before opening the saved copy, then verified their restoration and read the saved result through a separate read-only database connection.
- The final workspace capture recorded both the requested and actual egui panel widths as 360 pixels. A separate reader capture recorded 420 pixels for both values.
- Native list captures covered the previous desktop layout, a desktop list with only document/state selected, and the light Spanish list at 640×480 with text at 200% and reduced motion. Rendered inspection confirmed the hidden fields and retained document actions. Small windows keep the existing scroll behavior.
- Native debug compilation, workspace formatting, and workspace Clippy across all targets with warnings denied passed. No dependencies were added.

The [status artifact](artifacts/workspace-layout.json) records these outcomes without document paths. Local captures remain under `build/verification/workspace-layout`.

Physical resizing, mouse/keyboard interaction with the new submenu, Narrator, multiple monitors, and restart of a production personal store were not exercised in this checkpoint. Complete workspace, accessibility, performance, and replacement-release acceptance remain open.
