# Compact library composition checkpoint

Date: 2026-10-31. Scope: responsive library composition, readable cover size, hover headroom and enlarged-text controls. This is not complete visual, accessibility or rewrite acceptance.

## Changes

The cover grid no longer derives jacket width from the remaining height inside a scroll area. Columns and jacket widths follow horizontal space and the existing density choice. Compact grids target 180-point book allocations; the regular gallery retains its 228-point jacket cap. Books reserve 32 points above their unprojected jacket, including room for the spring elevation and rigid projection. Their title and metadata allocation moves with that reserved space. Full-size physics and PDF front/spine materials remain connected; miniatures use a smaller elevation target while keeping tilt.

Narrow library headers expose filters through a menu and retain a visible reset action when filters are active. The smallest enlarged-text profile moves search and view choices into temporary menus. The search field shares one renderer and the existing query/reset/refresh handler. Very short library pages use one outer scroll area instead of nesting a fixed-height catalogue scroll inside it. Header rendering and catalogue paging moved out of the large application drawing method without changing their storage actions.

## Evidence

Offline binary Clippy with warnings denied and an offline native build passed. Formatting was applied. No standalone tests or broad workspace suite were added or run for this UI unit.

Two bounded native inspection rounds were used. The first exposed how much room the enlarged-text header still occupied. The final round confirms the search/view menus and single-scroll composition. No Computer Use was used.

| Final native case | Evidence |
| --- | --- |
| library-compact-dark-es | 720 × 540, Spanish, actual synthetic PDF covers and a forced upper-left pointer target through the normal spring; compact book allocations remain 180 points and the first projected book volume fits the visible area |
| library-compact-light | 900 × 600, light theme; the width-driven grid retains readable cover and metadata proportions |
| library-short-wide | 1440 × 600, Spanish and the opposite pointer target; compact book allocations remain 180 points and the elevated first book fits |
| library-compact-view-menu | 640 × 480, 200% text and reduced motion; Covers, List and Details are fully visible in their temporary menu |
| library-compact-search-menu | Same enlarged-text profile; the shared search field remains fully visible in its labelled popup |
| library-compact-filters-active | Narrow Spanish/light profile with a missing-files filter; the reset control remains visible alongside the filtered empty state |
| library-wide-light-pdf | Regular wide gallery retains the larger jacket cap and full filter composition |
| library-small-200 | Enlarged-text page without an overlay; the first miniature, title, author and reading action are visible, with search, filtering and view controls available |

Every final capture passed custom-control horizontal bounds checks. Additional checks used the captured book-slot rectangles and the union of all eight projected front/back corners: the three compact grid profiles retain 180-point book allocations and fully visible first-book volumes. The first enlarged-text miniature fits its viewport. Dedicated visibility checks passed for the reading action, view-menu choices, popup search input and reset control. Bounds do not by themselves prove occlusion or interaction; the affected compositions were also visually inspected.

The [capture manifest](../screenshots/rust-compact-gallery/capture-manifest.json) records source, lockfile, executable and artifact hashes. Published PNGs contain origin metadata. Stores and PDFs are synthetic; no personal document was published.

## Remaining acceptance

The popups were opened through internal capture flags, not physical pointer or keyboard input. Full grouping/list/selection interaction, focus across live resize, Narrator, physical DPI, multi-monitor operation and real-library visual acceptance remain pending. No frame-time or speedup claim follows from these screenshots. The full migration, PDF/performance corpus, wider parity matrix and Windows Release remain open.
