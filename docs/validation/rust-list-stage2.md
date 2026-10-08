# Native library list — stage 2

Implemented and connected in `rust/ui/src/app/list.rs`, called inside the existing
catalogue scroll area. Wide rows are 92 logical points high.

- At 760 logical points and above, a shared column model aligns document,
  reading progress, page count and two fixed 40-point actions. The 54-point
  miniature uses the existing `book::miniature` scene and physics.
- Narrow layouts stack a two-line title, author, actual progress/page count,
  progress track and fixed action footer. Logical widths automatically shrink
  with the application's 200% zoom; no extra horizontal scroll area is added.
- Selection has raised fill and a violet point; pointer hover changes the surface; keyboard
  focus has a separate outline. Click selects, double-click/Enter opens, and
  the details action selects the document and opens the existing inspector.
- ES/EN action names include the document title in tooltips and widget
  semantics. Full titles/authors remain in the row's semantic name and tooltip.
  Favorite and unavailable-file states have text labels.
- The context menu uses existing controls for opening, details, favorite
  changes and Explorer. Favorite updates still use `Action::Favorite` and
  refresh; Explorer errors use the existing application error state.
- Every row has a document ID scope. Offscreen rows reserve known geometry
  and skip text, miniature and paint work. No SQL/schema, preference or FFI
  changes were made.

Integration formatting, build and Clippy passed. `library-list.png` verifies
the connected wide layout; `library-list-states.png` uses real selected state on
one row and `response.request_focus()` on another, showing separate treatments.
Miniatures at 54 points omit title lettering and retain pigment/art/volume.
The bounded independent review resolved these corrections; see
`rust-visual-stage2.md`. Physical keyboard walkthrough, Narrator and full list
parity are not certified by a screenshot or a programmatic focus request.
