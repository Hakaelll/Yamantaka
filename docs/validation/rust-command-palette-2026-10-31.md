# Native command palette checkpoint

Date: 2026-10-31. Scope: the inherited command-palette entry point, scoped reader actions and recent-document opening. This checkpoint does not establish full rewrite parity or release acceptance.

## Connected behavior

Ctrl+K and Commands in Tools or reader options open the native palette. Its assignment joins the configurable registry, which now has 28 actions. Spanish and English action names, stable command identifiers and common accent-insensitive text are searchable. Arrow keys and Enter act on the selected result while the search field owns focus. Escape or Close dismisses the palette. A no-match Enter cannot navigate the underlying reader.

The palette captures the active pane and document identity. Reader actions use that context and the existing shortcut dispatcher or navigation controllers. Closing restores canvas focus. Replacing or closing the captured document dismisses the palette. Library-selection keys and reader shortcuts cannot act behind the modal.

Additional entries expose first/last page, page layouts, navigator sections, six reader themes, temporary A/B save/jump/swap and read-only document information. Unset mark destinations are disabled. Main and reference marks remain independent. Existing comparison synchronization still governs navigation and zoom.

An asynchronous catalogue query supplies at most eight documents ordered by last opened, independently of the library's active filters. Closing cancels that request. A late reply must still own the current request before updating the palette. Choosing a document uses normal document lookup and opening. Actions remain available while the query is pending or if it fails. No SQL, PDFium call or filesystem access was added to drawing.

## Executed evidence

The offline native build passed. A fresh binary Clippy run with warnings denied passed after the final visual changes. Formatting and whitespace checks are recorded with the commit. No standalone test or broad workspace test run was added for this unit.

Four native synthetic scenarios passed their state and control-bound assertions using real egui keyboard/text events:

| Case | Observed result |
| --- | --- |
| palette-reader | Ctrl+K opens the palette; the asynchronous recent snapshot settles; `pagina` finds accented page actions; reader context and search remain visible |
| palette-reference-actions | F6 targets reference; last/first-page commands preserve main-page independence; save A, jump A and swap A/B operate on reference marks; document information shows that reference's native page count |
| palette-recent | A library filter produces zero rows; the independent recent snapshot still opens its selected document through normal reader opening |
| palette-empty-small-200 | Down/Enter on an unmatched query leaves the reader unchanged; Escape closes; the reopened empty state keeps Close accessible at requested 640 × 480 logical size, 200% text, light theme and reduced motion |

The final dark search and enlarged light empty-state PNGs were visually inspected. The list uses flat rows with an accent outline for selection. Its scroll area fits the available window height. No Computer Use was used.

Captures contain only isolated synthetic stores and generated acceptance PDFs. Source files, Cargo.lock and the frozen executable are checked against the capture manifest before publication. Published PNGs embed origin metadata. See the [capture manifest](../screenshots/rust-command-palette/capture-manifest.json) and [user guide](../user-guide/command-palette.md).

## Remaining acceptance

Native annotation undo/redo is still required and is not advertised as an available palette action. These scenarios do not exercise every command, every cancellation timing or every physical keyboard layout. Narrator, physical monitor DPI, full interaction/performance acceptance, safe-mode and migration gates, and the final Windows release remain pending. The bounded recent query is not a latency benchmark. No personal PDF, library data or executable is published in this checkpoint.
