---
name: Yamantaka
description: "A native Windows gallery for private reading and study."
colors:
  dark-accent: "#BDA3F5"
  light-accent: "#6748A7"
  dark-canvas: "#0F1012"
  dark-surface: "#16171A"
  dark-raised: "#202125"
  dark-ink: "#EFEDE8"
  dark-secondary: "#A7A8B0"
  dark-line: "#323339"
  light-canvas: "#F7F5EF"
  light-surface: "#FFFDF8"
  light-raised: "#EBE8E0"
  light-ink: "#212227"
  light-secondary: "#64646D"
  light-line: "#D3CFC6"
  dark-primary-ink: "#191324"
  light-primary-ink: "#FFFFFF"
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
  library-display:
    fontFamily: "Source Serif 4 Display, Segoe UI Variable, Segoe UI, egui proportional fallback"
    fontSize: "38px"
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
  native-field: "7px"
  control: "8px"
  theme-choice: "10px"
  continuity: "12px"
  cover-state: "6px"
spacing:
  item: "10px"
  button-inline: "14px"
  button-block: "9px"
  field-block: "10px"
  search-inline: "36px"
  gallery-gutter: "36px"
  library-wide-inset: "42px"
  library-narrow-inset: "22px"
components:
  button-primary-dark:
    backgroundColor: "{colors.dark-accent}"
    textColor: "{colors.dark-primary-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    height: "40px"
  button-primary-light:
    backgroundColor: "{colors.light-accent}"
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

Beauty comes from physical coherence and clear hierarchy: connected book geometry, lettering attached to the cover plane, stable catalogue titles, and restrained native controls. Segoe UI Variable is the explicitly required Windows interface face. Locally bundled Source Serif 4 Display accompanies document titles and three jacket families. The exact existing Yamantaka logo remains the brand asset.

**Key Characteristics:**

- Physical cover volumes with independent, readable catalogue labels.
- Sculpted spine light, recessed hinges and fixed cloth weave attached to the book plane; projected selection follows the same volume.
- Available PDF imagery is mounted on the physical cover and spine. Procedural jackets provide the fallback; native WGPU first-page extraction is verified in the [PDF cover checkpoint](docs/validation/rust-pdf-covers-2026-10-13.md).
- Neutral graphite or warm paper; violet for action, selection and focus.
- Local Segoe interface type paired with editorial document lettering.
- Drawn vector controls with separate focus, pressed and selected states.
- Cover-only pointer motion with an equivalent static reduced-motion state.

This refreshed system records the current stage 2 Rust build. The preserved
[Flutter v0.15 document](docs/architecture/flutter-design-v015.md) is historical;
its feature inventory and acceptance claims are not Rust implementation facts.
The native sources are `rust/ui/src/theme.rs`, `book.rs`, `app.rs`, `main.rs` and
`app/{details,list,settings,compact,navigation,reader_controls}.rs`. Frontmatter `px` serializes egui
logical points at text scale 1.0. The product has no CSS runtime; sidecar HTML/CSS
is explicitly illustrative panel translation, never a native screenshot.

**Evidence and acceptance scope:** authoritative current captures are in
`build/rust-visual/stage2/`: `library-dark.png`, `library-light.png`,
`library-details.png`, `library-list.png`, `library-small-200.png`,
`settings-light.png`, `settings-small-200.png`, `reader-dark.png` and the subsequent
`reader-small-200.png`. They show a
labeled synthetic study collection and a PDF fixture. The initial independent
stage 2 review returned **fix**. Independent confirmation of the eight named fixes
and compact-reader regression returned **remaining: clear; disposition: ship**.
That verdict covers only those corrections; see `docs/validation/rust-visual-stage2.md`
and the reviewed snapshot in `docs/screenshots/rust-stage2/`. Earlier bounded stage
1 acceptance does not approve this replacement. No web detector evaluates this
native build. Narrator, physical
DPI, multiple monitors, the full language/text/layout matrix, functional parity
and the full QUALITY BAR challenger/card ceiling remain separate evidence gates.

The [direction seed record](docs/architecture/rust-visual-seed.md) preserves seed
`26a7630f`, API source, pool `c3b204a1eed6` and grounded candidate 7. It is recorded
direction evidence, not the complete raw hand: one challenger was lost in the
truncated output, no challenger was adopted, and no challenger quality-board
review is claimed. The direction is code-led; no approved visual comp is implied.

## Colors

Graphite and warm paper share a functional violet accent. Jacket pigments and
subdued foil are book materials, not additional interface accents.

### Primary

- **Soft violet** (`dark-accent`): primary actions, focus and selection on graphite.
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

**The Functional Violet Rule.** Use the accent for primary actions, selection and focus; preserve neutral surfaces around the collection.

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

- **Display / compact display:** library heading in spacious and compact layouts.
- **Headline:** shared native Heading style; contextual settings section titles use 28 points.
- **Title:** reader document context; list and constrained catalogue titles use 16 points.
- **Body:** catalogue labels and supporting interface text.
- **Label / small / cover metadata:** controls, quiet context and page/progress summaries.
- **Editorial:** inspection title at 22 points; book titles remain the library focus.
- **Jacket:** families 0, 3 and 5 use the bold jacket face; families 1, 2 and 4 use editorial type.

Settings has a contextual 38-point heading, with 20 points in cramped composition;
these do not enlarge the global type ramp. Jacket titles start at 10% of cover
width, or 11.5% for families 1 and 4, capped at 25 points (27 for family 3).
Separate title and artwork regions prevent overlap. Titles allow three rows and
shrink to fit their region; authors occupy one row only when cover width is at
least 170 points. Covers at or below 60 points communicate through pigment and
artwork without jacket title text. Normal catalogue titles remain upright in two
rows. Constrained rows supply a readable title beside the miniature.

Text scale sets native zoom from 1.0 to 2.0, affecting logical layout width as well
as text. The library title/count band disappears below 420 available logical points;
this is constrained composition, not a global removal of branding or context.

**The Two Voices Rule.** Segoe carries the interface; editorial type identifies documents. Keep catalogue labels independent of the projected jacket lettering.

## Layout

The library uses horizontal utility bands and an open collection with no permanent
sidebar. Details opens a resizable bottom inspection tray, preserving gallery width
and column alignment. Settings category navigation and the reader's optional text
panel belong to their own surfaces.

The window starts at 1440×960 and has a 640×480 minimum. These are logical points
before text zoom. Compact library composition is triggered below 650 content
height; below 420 height the complete library body can scroll vertically. Side
insets switch at 850 available width. Utility insets are 24×8, or 18×4 when compact;
library vertical inset is 18, or 10 when compact, with compact vertical gaps of 4.

Columns derive from available width divided by 240, or 260 when compact. Equal
slots are separated by the gallery gutter; normal covers cap at 228 points.
Compact cover width is bounded by
`min(slot, clamp((available_height - CAPTION_HEIGHT - 5) / 1.34, 110, 180))`.
Normal cover allocation is `width × (width × 1.34 + CAPTION_HEIGHT)`, with the
shared caption height of 102 points. Captions reserve two title lines at 16
points, a separate 13-point author line, page/progress metadata and a two-point
progress track for started books. Long titles and authors truncate with ellipses;
the semantic book button retains the full title, author, page count and progress.
When the zoom-adjusted content width falls below 420, the cover collection becomes
76-point jacket-and-title rows with explicit Read controls instead of the normal
grid. Continue reading is a secondary action beside the library filters. Compact
composition keeps the existing Resume action in the utility band. Neither layout
reserves a separate continuity strip.

Search requests 40-point height; it spans its row on narrow layouts. Filters move
into a labeled semantic vector menu below 420 content width. View and inspector
controls switch from text plus vectors to semantic vector buttons when space is
limited. The utility row uses the same adaptation below 540 available width.

The inspection tray defaults to 148 points and ranges from 80 to half the available
content height (with a 160-point floor in the range calculation); its content
scrolls and wraps. List view switches at 760 available points from 92-point aligned
rows with headings to 184-point stacked rows. Wide list rows include a 54-point
actual book miniature, separated progress, page count and two 40-point actions.

Settings scrolls as a complete page. Its outer horizontal inset is 44, or 16 below
720 available width; its content caps at 1040. At 850 available content width,
180-point category navigation sits beside a content column capped at 700. Smaller
layouts wrap category controls horizontally; cramped layouts below 420 use one
category menu. Theme choices sit beside each other from 460 available points,
otherwise stack. All four categories remain reachable.

The reader uses one quiet, unfilled row with 8×2 insets at every width: Library,
previous page, a 64×40 page/total field, next page, Reading options and the reading
navigator. Options and navigator align to the right when space permits. The row
uses 2-point gaps and 4 points of separation above the document. Reading options
contains zoom, fit, open readings, history and saved A/B positions. Its optional page-text panel defaults to 280 and ranges
from 180 to 500. PDF page appearance comes from the actual document texture.

**The Stable Collection Rule.** Inspection opens below the library; preserve gallery width rather than shifting its columns sideways.

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
5-point dot; keyboard focus outlines the full allocation at 2 points. The exact
logo appears at 24×24 with a 12-point clipping radius; its geometry is unchanged.

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

Continue reading sits beside the filters as a transparent secondary action,
with an arrow-only version below 1100 points of content width. Its tooltip shows
the document title, current page and percentage. Short windows keep the existing
Resume action in the utility band, avoiding duplicate controls. There is no
separate continuity card, miniature or progress track above the collection.
List rows are open
aligned content with separators, actual miniature in wide mode and independent
Read/Details actions. Selected rows use raised fill; keyboard focus adds its own
outline. Narrow rows wrap document context into a bounded two-line title region.

### Inspection tray

The bottom tray has wrapped document title, author, reading position, Read,
Favorite, Explorer and Close actions, followed by original-file context. Internal
insets are 24×12. It adds vertical inspection space without changing gallery width. The Compare with another PDF action and its cancel/result state stay inside this
scrollable tray. A comparison never adds a global progress banner or changes
collection width. Result text uses the secondary foreground and wraps with the
available inspector width.

### Settings theme choices and fields

Four categories expose appearance, accessibility, language and privacy. Appearance
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
regions and alignments. Click selects; double-click or Enter reads. Contextual
actions and visible Read controls provide additional paths. Pointer response is
bounded to the actual book rectangle rather than its whole catalogue cell.

**The Independent States Rule.** Selection, keyboard focus and hover motion are separate signals. Reduced motion retains the static volume and both state treatments.

## Do's and Don'ts

### Do:

- **Do** use the paired native palette for the active theme.
- **Do** keep jacket lettering, artwork and edges attached to the same cover geometry.
- **Do** preserve readable catalogue titles and semantic labels for vector actions.
- **Do** distinguish selection, keyboard focus and pointer motion.
- **Do** preserve the existing logo geometry and the local font fallback order.
- **Do** adapt to logical width after text zoom, keeping constrained layouts scrollable.
- **Do** record executed evidence and acceptance scope before claiming a section complete.

### Don't:

- **Don't** add a permanent library sidebar.
- **Don't** make essential actions depend on pointer hover.
- **Don't** turn violet into a large decorative surface.
- **Don't** replace the physical books with generic flat cards.
- **Don't** treat panel HTML/CSS examples or settings diagrams as native screenshots.
- **Don't** claim whole-rewrite, Narrator, physical-DPI or full QUALITY BAR approval from these captures.
