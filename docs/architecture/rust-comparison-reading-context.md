# Preserve reference reading preferences

Saved comparisons now carry an optional `reading` object alongside their existing page, anchor, zoom, theme, rotation, margins and pane arrangement. It records Document/Book mode, the persisted page-layout spelling, cover pairing, initial side and automatic control hiding. Both native comparison snapshots and the Flutter split adapter capture these values from the reference preferences.

Previously the adapter constructed fresh default preferences and restored only zoom, theme, rotation and margins. A saved Book or continuous reference therefore reopened as a single-page Document reference. The shared preference reconstruction now parses the additional settings through the existing domain adapters. Unknown values return an error without rewriting the stored context. Comparisons without `reading` retain their earlier defaults.

Default-session files containing the new context use continuity version seven. Versions one through six remain readable. Once written, version seven is retained on later saves, including after closing the comparison. Named workspaces store the same optional comparison object in their existing query JSON; the main SQLite schema stays at version eleven. JSON backup format nine is unchanged.

Restored references continue through the existing deferred comparison path. Its shared viewport planner prepares intersecting reference pages as well as newly exposed main-reader pages before revealing the saved split. This matters when restoring a facing spread or a continuous reference. The regular native reference chooser still starts in its existing single-page mode; this change restores saved choices rather than changing that default.

No tests, captures or compilation were run for this change at the user's request. The retained compatibility fixture was updated only to supply the optional field. Functional, interruption and visual acceptance remain pending, including restored Book pairing, continuous scrolling, synchronized comparisons, electronic ink and older snapshots.
