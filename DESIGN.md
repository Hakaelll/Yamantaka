---
name: Yamantaka
description: "A native Windows gallery for private reading and study."
colors:
  dark-accent: "#BDA3F5"
  light-accent: "#6748A7"
  dark-canvas: "#161615"
  dark-surface: "#1E1D1B"
  dark-raised: "#2A2825"
  dark-ink: "#F3EDE1"
  dark-secondary: "#B3AB9F"
  dark-line: "#3E3A35"
  light-canvas: "#F4EFE5"
  light-surface: "#FCF8F0"
  light-raised: "#E8E0D3"
  light-ink: "#29221C"
  light-secondary: "#60544A"
  light-line: "#CEC2B0"
  dark-primary-ink: "#161615"
  light-primary-ink: "#F4EFE5"
  jacket-slate: "#2B3D51"
  jacket-plum: "#532E39"
  jacket-forest: "#2C473A"
  jacket-umber: "#996A45"
  jacket-indigo: "#3A314F"
  jacket-teal: "#214A4D"
  jacket-ink: "#E6DECF"
  jacket-foil: "#C9A875"
  paper-top: "#D9D3C3"
  paper-edge: "#C6BFAF"
  paper-bottom: "#B1AB9D"
  paper-rule: "#989283"
typography:
  display:
    fontFamily: "Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "34px"
  display-compact:
    fontFamily: "Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "26px"
  headline:
    fontFamily: "Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "30px"
  title:
    fontFamily: "Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "18px"
  body:
    fontFamily: "Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "15px"
  label:
    fontFamily: "Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "14px"
  small:
    fontFamily: "Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "13px"
  cover-meta:
    fontFamily: "Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "12px"
  editorial:
    fontFamily: "Source Serif 4 Display, Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontWeight: 400
  jacket:
    fontFamily: "Segoe UI Bold, Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontWeight: 700
rounded:
  native-field: "8px"
  control: "9px"
  menu: "12px"
  theme-choice: "10px"
  continuity: "12px"
  cover-state: "6px"
spacing:
  item: "10px"
  button-inline: "14px"
  button-block: "9px"
  field-block: "10px"
  search-inline: "36px"
  gallery-gutter: "26px"
  library-wide-inset: "32px"
  library-narrow-inset: "12px"
components:
  button-primary-dark:
    backgroundColor: "{colors.dark-ink}"
    textColor: "{colors.dark-primary-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    height: "40px"
  button-primary-light:
    backgroundColor: "{colors.light-ink}"
    textColor: "{colors.light-primary-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    height: "40px"
  button-secondary-dark:
    backgroundColor: "transparent"
    textColor: "{colors.dark-secondary}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    height: "40px"
  button-secondary-dark-hover:
    backgroundColor: "{colors.dark-raised}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.control}"
  navigation-dark:
    backgroundColor: "transparent"
    textColor: "{colors.dark-secondary}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    height: "40px"
  navigation-dark-selected:
    backgroundColor: "{colors.dark-raised}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.control}"
  chip-dark-selected:
    backgroundColor: "{colors.dark-raised}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.native-field}"
  search-dark:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-ink}"
    typography: "{typography.body}"
    rounded: "{rounded.native-field}"
    padding: "10px 36px"
    height: "40px"
  theme-choice-light:
    backgroundColor: "{colors.light-surface}"
    textColor: "{colors.light-ink}"
    rounded: "{rounded.theme-choice}"
  continuity-dark:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-ink}"
    rounded: "{rounded.continuity}"
    padding: "8px 14px"
  cover-slate:
    backgroundColor: "{colors.jacket-slate}"
    textColor: "{colors.jacket-ink}"
---

# Design System: Yamantaka

## Overview

**Creative North Star: "Gallery study room"**

A personal collection supplies the identity. Matte book volumes, visible paper edges and family-specific jacket lettering make books recognizable, while graphite surfaces, ivory text and functional violet keep the interface calm and precise. Light mode changes the room to warm paper without changing the collection's authored colors.

Beauty comes from physical coherence and clear hierarchy: connected book geometry, lettering attached to the cover plane, stable catalogue titles, and restrained native controls. Segoe UI Variable is the explicitly required Windows interface face. Locally bundled Source Serif 4 Display accompanies document titles and three jacket families. The abstract black open-folio mark in `assets/branding/yamantaka-logo.svg` is the canonical brand asset, following the owner's 2026-11-02 request to replace the previous emblem.

**Key Characteristics:**

- Physical cover volumes with independent, readable catalogue labels.
- Sculpted spine light, recessed hinges and fixed cloth weave attached to the book plane; projected selection follows the same volume.
- Available PDF imagery is mounted on the physical cover and spine. Procedural jackets provide the fallback; native WGPU first-page extraction is verified in the [PDF cover checkpoint](docs/validation/rust-pdf-covers-2026-10-13.md).
- Neutral graphite or warm paper; violet for selection, position and focus.
- Local Segoe interface type paired with editorial document lettering.
- Drawn vector controls with separate focus, pressed and selected states.
- Pointer feedback on controls, physical volumes and reader chrome, with immediate reduced-motion states.

This document records the native Windows layout and the existing book rendering system. The preserved
[Flutter v0.15 document](docs/architecture/flutter-design-v015.md) is historical;
its feature inventory and acceptance claims are not Rust implementation facts.
The native sources are `rust/ui/src/theme.rs`, `book.rs`, `app.rs`, `main.rs` and
`app/{details,list,settings,compact,navigation,reader_controls}.rs`. Frontmatter `px` serializes egui
logical points at text scale 1.0. The product has no CSS runtime; sidecar HTML/CSS
is explicitly illustrative panel translation, never a native screenshot.

The [release library captures](docs/screenshots/rust-readme-2026-11-03/) and
[earlier visual review](docs/validation/rust-visual-stage2.md) record the baseline.
New comparisons use the native synthetic fixture, with the same language, theme,
viewport and text scale on both builds. The [testing guide](TESTING.md) lists the
workspace checks and the native capture workflow.

The [direction seed record](docs/architecture/rust-visual-seed.md) preserves seed
`26a7630f`, API source, pool `c3b204a1eed6` and grounded candidate 7. It is recorded
direction evidence, not the complete raw hand: one challenger was lost in the
truncated output, no challenger was adopted, and no challenger quality-board
review is claimed. The direction is code-led; no approved visual comp is implied.

## Colors

Graphite and warm paper share a functional violet accent. Jacket pigments and
subdued foil are book materials, not additional interface accents.

### Primary

- **Soft violet** (`dark-accent`): focus, selection and reader position on graphite.
- **Study violet** (`light-accent`): corresponding roles on warm paper.
- **Deep plum ink / white action ink** (`dark-primary-ink`, `light-primary-ink`):
  paired foregrounds on accent controls.

### Secondary

Slate, plum, forest, umber, indigo and teal are deterministic jacket pigments
selected by document ID, unchanged between themes. Jacket ivory supplies cover
lettering and attenuated artwork; jacket foil supplies rings, facets, bars and
wave highlights. Native mesh lighting alters these source pigments; it does not
create alternate global swatches.

### Neutral

Graphite room, graphite surface and raised graphite are the canvas, utility bands
and interaction states. Ivory ink, quiet grey and graphite rule provide primary
text, secondary context and boundaries. Warm paper, ivory surface and pressed
paper supply the corresponding light layers; charcoal ink, warm grey and paper
rule supply light-theme text and boundaries. The four paper tokens belong to the
connected page block, not interface containers.

**The Functional Violet Rule.** Use the accent for selection, focus and reader position. Primary buttons invert the ink and canvas colors.

## Typography

**Interface font:** locally loaded Segoe UI Variable, then local Segoe UI, then
egui proportional fallback. **Editorial font:** bundled Source Serif 4 Display
Regular (OFL provenance in `assets/fonts/source-serif`), followed by the interface
family. **Bold jacket font:** local Segoe UI Bold, followed by the interface family.
No remote font request is part of the interface.

The explicit Segoe UI brief governs interface display and body roles. Source Serif
is reserved for document lettering rather than navigation. Source `strong()` calls
provide contextual emphasis; no global line-height or tracking scale is established.

### Hierarchy

- **Display / compact display:** headings for task views at different widths.
- **Headline:** shared native Heading style; contextual settings section titles use 28 points.
- **Title:** reader document context; list and constrained catalogue titles use 16 points.
- **Body:** catalogue labels and supporting interface text.
- **Label / small / cover metadata:** controls, quiet context and page/progress summaries.
- **Editorial:** inspection title at 22 points; book titles remain the library focus.
- **Jacket:** families 0, 3 and 5 use the bold jacket face; families 1, 2 and 4 use editorial type.

Settings has a 42-point editorial heading on wide windows, with 20 points in cramped composition;
these do not enlarge the global type ramp. Jacket titles start at 10% of cover
width, or 11.5% for families 1 and 4, capped at 25 points (27 for family 3).
Separate title and artwork regions prevent overlap. Titles allow three rows and
shrink to fit their region; authors occupy one row only when cover width is at
least 170 points. Covers at or below 60 points communicate through pigment and
artwork without jacket title text. Normal catalogue titles remain upright in two
rows. Constrained rows supply a readable title beside the miniature.

Text scale sets native zoom from 1.0 to 2.0, affecting logical layout width as well
as text. The catalogue starts with its controls and uses the available height
for books.

**The Two Voices Rule.** Segoe carries the interface; editorial type identifies documents. Keep catalogue labels independent of the projected jacket lettering.

## Layout

The application uses the native window frame. The catalogue and reader each have
one working row. The reader's document title opens the reading switcher where
width allows. Compact readers reach Open readings from More controls. The menu
uses the existing session model for selecting, pinning, reordering and reopening
documents. Session groups the queue, workspaces and history. Secondary views
provide a Library return control. Concentration keeps a named exit control.

### Library and book details

A single working row contains catalogue search, Add PDFs, Filters, View and
Organize library. Compact layouts use named icon controls. Collection lives
inside Filters and active criteria have a visible reset and result count. View
owns the grid/list choice, ordering, grouping and cover density.
Library-wide PDF content search opens its own surface with the existing opt-in
local index. Organize library also reaches tags, collections, rules, duplicates,
folder import, source checks, recovery and library notes. Session, Settings
and Select also start here. The selection actions appear while selecting.

Continue shows the recoverable title and last page on demand. It uses the active
reading or an independent query for the most recently opened book in progress.
While filtering, Continue moves to the result row. Saved readings also remain
available from Session.

The gallery uses an open canvas with 26-point gutters at normal width and
20-point gutters in compact density. Jacket slots cap at 212 and 172 points
respectively. Captions align below the projected geometry. They keep two title
lines and an author line. A started book reveals its percentage and progress
rail below the caption on cover hover or keyboard focus, within reserved space.
In the compact row the same rail sits beneath the miniature. The pointer fade
takes 120 ms and reduced motion makes the change immediate.
Each book has an accessible actions menu. Click or Enter opens it, while
modified clicks and selection mode keep their selection behavior. Narrow
windows use a miniature and title row. At the smallest size the full accessible
label carries metadata that would otherwise compete with the book actions.

Book details opens in the main area. Its return restores catalogue state and
focus. The cover, title and reading action come first. Metadata and goals,
organization, original file and cover, and connections and revisions form four
groups. Source recovery and validation stay with the relevant book. The actions
and confirmation text remain reachable by scrolling when the text scale grows.

### Reading and study

The normal reader reserves a stable toolbar above the PDF. Page and zoom
controls form one group, and Find, Navigate and Notes form the other. A pointer
reveal fades the controls in over 160 ms. Keyboard focus and reduced motion
reveal them immediately. In the narrowest row, page entry and the study
actions stay direct. More controls holds comparison, view and reading settings.
The existing preference hides the controls within the toolbar area, preserving
the PDF viewport and leaving a keyboard-accessible reveal. Concentration uses
the existing explicit exit path.

Navigate and Notes share one bottom study tray. Navigate groups contents, pages,
bookmarks and positions. Notes groups annotations and extracted page text. Find
uses the same study area, with an editable query, match count and previous/next
actions. Selection actions stay near the passage. In compact windows the study
area becomes a temporary full-width view with Back to PDF. Geometry and opening
preparation use the same toolbar and tray allocation as the visible reader.
Comparison labels the active document and keeps the existing automatic,
side-by-side and stacked arrangements and synchronization controls.

Selecting PDF text opens contextual marks near the passage. Study reveals
a second native working row for repeated marking and drawing. Compact widths
group the tools under the active tool selector. The paper and saved drawings
stay in the PDF viewport. Note indicators use the available page margin and
open the exact entry in Notes. Colors on the paper describe the saved marks.
The interface uses violet for the active tool and keyboard focus.
The eraser previews its target on the paper. It removes one mark per click
through annotation history, with written notes protected.
Notes can pin the page being read as a filter and combine it with Pending.
List entries pair the saved mark color with its type and review state.
Shared controls use softly rounded corners and a short violet selection
indicator. Menus use the reader palette and a restrained shadow.

### Settings and recovery

Settings puts its categories across the top and uses a named selector when they
cannot fit. A single content column uses labelled rows with bounded controls.
The storage section retains backup export, reviewed restore and legacy import.
Accessibility retains application text scale, reduced motion and the Electronic
ink profile. That profile keeps a white canvas and single-page navigation while
retaining each document's Standard preferences for later use.



Organize library includes Import folder and the existing Cancel import action.
Its contents scroll within the remaining viewport height, preserving access to
secondary destinations at enlarged text. Folder import uses the existing quiet
background workflow and never adds a permanent progress strip. Pending imports
appear as an Organize library submenu only when work is paused. Its bounded, scrollable list
shows the source label, confirmed count and Resume/Discard actions. Discard keeps
catalogued books. Startup leaves the work paused; recovery adds no global banner.

## Elevation & Depth

Interface depth comes from tonal layering and restrained boundaries. Books use
connected projected polygons, visible page edges, a darker spine, directional
jacket shading and two soft shadows. This is physical object depth rather than
an interface card-shadow scale.

The broad native contact shadow has resting offset (6, 10), blur 22, spread 0;
black alpha is 110/255 on dark canvas and 65/255 on light. Hover lift increases
vertical offset by `9 × lift` and blur by `10 × lift`. The tight shadow has offset
(2, 4), blur 6, black alpha `80 × (1 - lift × 0.5) / 255`. Both surround the book
inset by 3. Offset and blur use the native integer casts in `book.rs`.

Book depth is `clamp(width × 0.095, 12, 26)`. Rest rotations are −0.08 and −0.26
radians; perspective distance is 850. Pointer response uses a signed power of 1.28
before the bounded tilt spring (stiffness 190, damping 24, mass 1.15).
The separate interruptible lift spring uses stiffness 210 and damping 29; lift
is clamped to 0–1.05. Projection scale is `1 + lift × 0.018`; vertical displacement
is `tilt.length() × 4 + lift × 7`. Simulation steps clamp to 0.001–0.032 seconds.
Reduced motion resets tilt, velocity and lift to zero while retaining resting depth.

Controls interpolate hover over 140 ms with cubic ease-out. Theme selection uses
180 ms exponential ease-out. Keyboard focus avoids animated state interpolation;
reduced motion uses immediate state changes. Native scroll animation is disabled
when reduced motion is enabled. These are distinct from the physical spring.

**The One Rigid Book Rule.** Cover artwork, jacket lettering, spine and page edges belong to one connected volume; pointer motion never tilts the catalogue labels or surrounding UI.

## Shapes

Custom vector controls use the control radius; native text fields and selectable
labels retain the native-field radius. Theme choices and continuity use their own
radii. The projected jacket is a square-corner mesh with connected page geometry,
not a rounded UI card. Book selection and focus use the cover-state radius.

Custom controls have no ordinary boundary stroke. Selection uses raised fill plus
a 1.5-point violet underline; focus uses a distinct 2-point inset violet outline.
Native fields retain visible borders. Book selection uses a 1.5-point outline and
5-point dot; keyboard focus outlines the full allocation at 2 points. The corporate
logo appears at 24×24 with a 5-point clipping radius, preserving its rounded-square silhouette.

## Components

### Buttons

Drawn vector controls are quiet at rest and precise in action. Every custom target
is 40 points high and at least 40 wide. Icon-only controls are 40×40; labeled width
is measured text plus 28 points, or 52 with an icon. Text starts at 14 points, or
36 with an icon. Vector strokes are 1.5 points, drawn by the native painter.

Primary controls use accent fill and paired action ink, mixing toward palette ink
by up to 10% on hover. Secondary controls start transparent with secondary text;
hover blends toward raised fill and uses primary ink. Pointer-down moves the
painted control down one point. Selection has raised fill and an underline;
keyboard focus has its own outline. Disabled custom fill and foreground use gamma
multipliers 0.35 and 0.4. Semantic labels persist for icon-only actions.

### Navigation and filter chips

Application and settings navigation reuse the custom control state system.
Text accompanies vectors where space permits. The library has no permanent
navigation sidebar. Filter menus retain actual choices and reset actions;
native selectable labels retain theme raised fill and selection stroke.

### Search and fields

Title/author search uses native single-line editing, the surface fill, body type,
field radius and a drawn search vector in its left inset. The resting boundary
mixes the line color toward secondary (0.5 in dark mode, 0.65 in light). Active
native fields use accent treatment. Search and filters have a Reset filters path.

### Continuity and list rows

Continue displays the current book and its saved page alongside a small 3D
cover. The action and title wrap into a narrower composition when needed.
List rows use open alignment, separators, a physical miniature and independent
Read and Book details actions. Selected rows use raised fill. Keyboard focus
has its own outline.

### Book details

Read or Continue and Favorite sit beside the cover. File actions stay with the
original path and cover controls. Check file, Compare with another PDF and
Locate PDF keep their cancellation, review and result in this area. Result text
wraps at enlarged text. More actions contains the less frequent file and cover
operations.

### Settings theme choices and fields

The category row exposes appearance, accessibility, language, privacy, storage and shortcuts. Appearance
choices are working radio-style controls with drawn diagrams of the library and
reader palettes; diagrams are illustrations, not screenshots or pretend controls.
Hover uses raised fill, selected boundaries interpolate to accent and a filled
radio marker, and focus adds a separate 2-point outside outline. Preview diagrams
use the current slate, plum and forest jacket pigments. They communicate the
theme's composition while the native gallery retains the full six-family palette.

Text size has a labeled slider, numeric companion and 100/150/200% choices.
Reduced motion and ES/EN are live controls. Settings changes apply and persist
automatically outside the synthetic demo. This does not certify the full
accessibility or language acceptance matrix.

### Physical book volume

Each document ID selects one of six material/artwork families: foil rings, split
diamond, connected facets, tonal bars, flowing waves or radial spokes. Families
share geometry and physics but use different nonoverlapping title/art/author
regions and alignments. Click or Enter reads. Modifier keys and selection mode select. Contextual
actions and visible Read controls provide additional paths. Pointer response is
bounded to the actual book rectangle rather than its whole catalogue cell.

**The Independent States Rule.** Selection, keyboard focus and hover motion are separate signals. Reduced motion retains the static volume and both state treatments.

## Do's and Don'ts

### Do:

- **Do** use the paired native palette for the active theme.
- **Do** keep jacket lettering, artwork and edges attached to the same cover geometry.
- **Do** preserve readable catalogue titles and semantic labels for vector actions.
- **Do** distinguish selection, keyboard focus and pointer motion.
- **Do** use the canonical corporate geometric open-volume mark and preserve the local font fallback order.
- **Do** adapt to logical width after text zoom, keeping constrained layouts scrollable.
- **Do** record executed evidence and acceptance scope before claiming a section complete.

### Don't:

- **Don't** add a permanent library sidebar.
- **Don't** make essential actions depend on pointer hover.
- **Don't** turn violet into a large decorative surface.
- **Don't** replace the physical books with generic flat cards.
- **Don't** treat panel HTML/CSS examples or settings diagrams as native screenshots.
- **Don't** claim whole-rewrite, Narrator, physical-DPI or full QUALITY BAR approval from these captures.

Missing-source relocation stays inside the scrollable book details view. Locate PDF opens the native picker; selection produces a review rather than a write. A complete-reference match and a legacy partial match use distinct explanatory copy, followed by Update location and Cancel. The selected file name is visible and its full path is available on hover. No additional global toolbar or gallery overlay is introduced.

Library-wide source checks live under Organize library → Check library files. Its submenu contains the current phase, outcome counts, cancellation or restart, and missing/changed catalogue shortcuts. The popup may grow to its available viewport height; overflow scrolls inside it. The gallery keeps its normal chrome. Modified sources join the existing composable State filters and Reset filters behavior.

New-import content verification stays in the background import flow. It adds no progress row, modal or persistent gallery chrome. A later Check file result uses the file status in book details to confirm the imported reference. Registered paths reuse their document without reopening the metadata analysis flow.


Storage settings use the existing Operate layout and category navigation. Cover
cache size, image count and explicit Calculate size / Free cover cache actions
stay inside the settings content. A local Cancel action replaces those controls
while work is active. No gallery or reader progress bar is introduced. Cleanup
results report bytes removed and preserve an unknown remaining size after
cancellation. Narrow layouts reduce the heading and scroll the actions; synthetic
200% text captures focus the measured size and actions through that same scroll
area. The saved PDF page choice and resident cover textures remain in place.


Collections are an Operate surface reached from Organize library, with no persistent
navigation rail. The same manager gains membership checkboxes when opened from
a document's details. Flat rows keep names, member counts, pin state and View
books together; rename and reviewed deletion stay local to the row. Search and
creation use labelled fields. Pending writes retain the layout and disable only
their controls. The active collection uses the library's existing filter row and
clear/reset actions. The manager scrolls at narrow widths and 200% text; the book
gallery and reader keep their established composition.


Smart views share the existing State control rather than adding another library
row. The popover separates Reading state and Smart views, keeps selected states
visible and gives recent views a last-14-days tooltip. Its scroll area and the
compact parent menu remain bounded by the viewport. The active view changes the
control label and participates in the existing reset feedback. Gallery covers,
physics and reader chrome keep their established composition.


Tags are a dedicated Operate surface in Organize library and in document details. Row color
marks identify stored colors; name, independent book/annotation counts and
separate assignment/filter checkboxes keep their meanings distinct. Multi-tag
filters match any selected tag and use one compact control in the existing
library filter row. Creation from a book assigns atomically and then shows that
exact tag, with an explicit return to all tags. Tag and collection forms clamp
to available width and stack controls on narrow windows; recorded native control
bounds enforce that controls stay inside their horizontal clip.

Library multiselection uses the existing filter row, never a separate persistent action bar. Its count opens a bounded, scrollable menu; narrow layouts use a checkbox icon with the full accessible name. When books are selected, favorite actions come first. A running write puts cancellation first. Selection keeps the projected violet cover outline and marker, list rows use raised fill and a marker, and keyboard focus retains its independent outline. The inspector cursor does not stand in for the multiselection set.

Bulk organization uses a temporary, named destination chooser with a fixed selection count. Its header, search field and navigation/cancel footer stay stable; only destination rows scroll. Small windows use icon pagination with accessible names. Long destination labels use the same custom control renderer with bounded text, while AccessKit and hover retain the complete name. Closing the chooser cancels its read scope; submitted writes retain their independent scope and report through the compact Result menu.

Automatic cover regeneration stays inside the selection menu. A temporary confirmation names the selected count and states that manual covers will be replaced without changing PDFs. Cancel receives initial focus; background shortcuts stay inactive during review. The compact version keeps the entire consequence and both actions visible, with no reserved empty panel space. Progress and remaining-work cancellation use the existing selection menu, not a persistent banner. Existing cover textures remain registered through regeneration and any governed upload wait; replacement switches the texture and source page together.

Library removal shares the temporary cover-review component. Its consequence names the fixed count, deletion of local reading data, preservation of PDFs and absence of undo. Cancel receives initial focus. The compact layout keeps the consequence and both actions visible without reserved empty space. Progress stays in the selection menu. Reader recovery lives inside Session with Open readings and Reopen last reading; no additional recovery row interrupts the library header or raised 3D covers.

Collection ordering and icon changes stay inside each row's existing actions menu. Move earlier and Move later work with keyboard-accessible controls and the complete pinned or unpinned order. The inline icon editor uses five named choices with a saved-state marker and cancellation; headings draw the corresponding local stroke icon. Narrow layouts wrap the choices in the normal manager scroll area. The library gains no additional action bar.

Shared-tag actions stay inside each row's named menu. Rename and color use inline editors with explicit save and cancel. Merge uses a searchable, paged chooser with a fixed source, bounded destination labels and a stable cancellation footer. The source cannot be chosen as a destination. Merge and deletion use a separate compact review that preserves books and notes, states the irreversible association change and initially focuses Cancel. Review actions remain outside any scrolling body. These controls add no permanent library bar.

Library grouping stays within the existing sorting menu. Temporary section headings carry the group name, current-page count and an authored disclosure chevron; they wrap instead of truncating the name. Hover is quiet, keyboard focus remains distinct, and collapsed groups leave only their heading. Expanded groups reuse the physical book renderer and its lifting headroom. Miniature layouts reduce heading spacing. Group choice persists; folding is temporary. No additional persistent toolbar interrupts the collection.

Flutter import lives in Settings → Storage and opens a temporary review dialog. Source selection, counts, copy confirmation, and next-launch selection occupy that same dialog. Progress and recoverable errors appear first in its scroll body; Close or Cancel stays in the footer and receives initial focus. Narrow windows and enlarged text keep the recovery action visible. The imported-copy result distinguishes the library chosen for the next launch from the library currently open. Use original can undo that choice before restarting. Published captures replace local source paths with a synthetic label; the live picker retains the actual location. The workflow adds no permanent library bar or new motion.

Backup export uses an inline Storage section rather than another modal or persistent bar. The native save dialog chooses the destination; progress, cancellation, result, and retry occupy the same section. Export remains available after completion, followed by Show in folder. Compact layouts shorten the explanation and keep the result and export action together; the full filename remains in the normal layout. The section shares the established typography, restrained violet feedback, and bounded controls. It adds no decorative animation while a file is being written.

Restore sits beside export in Storage and uses a protected review dialog. The filename, backup version, record counts, and consequence appear before the explicit Restore and close action. Cancel receives initial keyboard focus. The scroll body leaves the cancellation footer stable. Background preparation stays in that dialog; a completed handover closes the application before installation. A native completion message identifies the retained safety directory and explains reopening. No permanent library bar or loading screen is added.

Reading queue lives in Session as a temporary dialog. Each open reading has a status, optional blocking reason and next action, with bounded reorder controls and an inline editor. Close receives initial focus; navigation and completion actions stay below the scroll body. The library gains no permanent bar.

Saved workspaces share that temporary Session pattern. A named save field precedes a bounded list; each row exposes Open and a quiet Options menu. Rename and duplicate edit inline. Updating and deletion require confirmation with Cancel initially focused. Close and paging remain outside the list. The reader stays visible behind the dialog and survives failed opening.

Concentration mode removes the reader toolbar and navigator without changing the window's fullscreen state. F8 and a floating, keyboard-focusable exit button restore the controls. It adds no animation or waiting period. Cover density lives inside the existing sorting menu and reuses the physical book renderer. Workspace queue writes stay silent on success; a failed write preserves the current readings and reports the error.

Workspace deletion exposes one temporary Undo deletion action in the manager, available until a later deletion or application exit. Restoring it leaves the current reader in place. Startup recovery stays silent when a saved workspace matches; conflicts preserve the readings and explain that the workspace was not linked. No startup wizard, progress bar or permanent recovery strip is added.

Compact library composition uses width and height to keep a complete first row visible. Compact jackets cap at 172 points and regular jackets at 212. Reserved projection space protects the lifting cover; miniature lift stays restrained. Below the full filter-row width, filters move to the existing temporary-menu pattern with an explicit active reset. At the smallest enlarged-text size, search and view choices use labelled popups and the page shares a single scroll area, so the header can leave room for a complete book row.

## Corporate identity

The owner requested a replacement mark on 2026-11-02 and then asked for a more abstract black symbol. Two angular leaves frame an open reading space in the current geometric mark. Its symmetrical silhouette suggests an open folio in perspective without using a letterform. The `#101112` geometry sits on a warm paper `#F2EFE8` tile with a 104-unit radius on a 512-unit canvas. The tile keeps the black mark visible against both light and dark Windows surfaces. The mark uses no gradients, lettering assets or external fonts. Functional violet remains an interface accent rather than the logo color.

`assets/branding/yamantaka-logo.svg` owns the geometry. Run `tool/generate_brand_assets.py` to rebuild PNG and multi-size ICO variants, then `tool/verify_brand_assets.ps1` to check them. The Rust executable embeds the Windows icon resource as well as setting its live window icon. Historical Flutter launcher assets use the same mark; this does not reactivate mobile or web targets. The previous emblem remains recoverable in Git.

Existing relation locations use an inline editor inside the selected connection. It names the current book, describes its stored anchor and requires explicit replacement before enabling Save. The other endpoint remains untouched. Controls wrap with the existing page-level scroll area and reuse page-choice semantics and native focus. Cancel discards the local proposal; a stale snapshot retains the proposal and offers Reload.
