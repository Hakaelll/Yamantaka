# Restore canonical positions within reading pages

Reader tabs now retain an optional normalized page point in addition to their page number and legacy vertical scroll offset. The point uses the existing `ReaderPosition` convention: coordinates before user rotation, with the displayed horizontal position aligned to the viewport center and the vertical position aligned to its top. Nonfinite and out-of-range coordinates are rejected by the domain constructor before replacing a tab.

The live reader captures this point through the same navigation feedback used by paging and scrolling. Tab switching, duplication, recently closed readings, late closure-context refresh, default-session persistence and named-workspace remapping carry it together. The SQLite schema and inherited workspace rows remain unchanged; the optional point lives in the existing richer Rust context.

Both the initial page-image planner and the adopted reader use the saved point. New snapshots therefore restore position through current page geometry rather than replaying an absolute vertical pixel offset from a differently sized viewport. Snapshots without a point keep their existing scroll-offset behavior. Electronic ink keeps canonical page-start presentation rather than restoring a panned viewport.

The legacy split adapter also carries the secondary tab's optional point into its reference configuration, instead of replacing it with the rotated page start. A reference with that point uses current geometry rather than the old vertical offset. Original Flutter tabs without a point retain their offset fallback.

Default-session continuity supports version eight and selects that version when any open or recently closed reading has a point. Versions one through seven remain readable; once written, version eight is retained. Older executables that do not support version eight cannot restore that new session file. Main catalogue records and original PDFs are unaffected.

No tests, compilation or native captures were run for this change at the user's request. Existing fixture callers were adapted to the additional optional argument without adding tests. Acceptance remains pending for horizontal panning, rotated pages, continuous layouts, Book spreads, changed window sizes, recently closed readings and restart.
