# Protect open drafts on normal close

Date: 2026-10-31. Scope: normal window close with a reader-note or metadata editor open.

## Behavior

Closing the main window now checks unsaved annotation editors in both reader panes and manual title/author corrections. It cancels the native close request and offers Return to editing or Discard and close. Existing stored notes that have not changed do not require review. New annotation editors retain their selection/page context until the user decides.

Accepted annotation, global-note and metadata writes must finish before this guard allows closing. The waiting state can be cancelled to stay in the app. If a failed save leaves a draft open, the guard offers review rather than discarding it. Returning to editing clears deferred close intentions from other existing close coordinators. A final cancellation prevents an automatic close from another coordinator from bypassing an open draft review.

The dialog uses the current native palette, control, focus and modal components. Keyboard shortcuts for the underlying reader are blocked while it is open. Return to editing receives initial focus; Escape or dismissing the modal preserves the draft. Small windows use tighter spacing and a bounded scroll region for the explanation, leaving actions outside that scroll area.

## Verification

Offline UI Clippy for all targets with warnings denied and the native build passed. Four native synthetic capture profiles exercise a primary page-note draft, a reference draft, a manual metadata correction and returning to the note editor after a close attempt. Their assertions require the draft to remain open and the expected review/return state. Review actions must stay inside the logical viewport.

The reference profile uses a 640 by 480 window, 200 percent text, light theme, Spanish and reduced motion. Its first visual inspection showed insufficient edge spacing. The compact dialog spacing was corrected before the final capture batch. The dark primary-note profile preserves the reading context beneath the modal.

The [capture manifest](../screenshots/rust-editor-close/capture-manifest.json) records source, executable and artifact hashes. Capture completion closes only its owned synthetic fixture, bypassing this normal-user close guard. No personal library or PDF is published and no standalone Rust test was added.

## Limits

This is normal-close protection, not persistent draft recovery. The graphics-failure path still bypasses normal UI interaction; unsaved drafts after a device failure, forced termination or process crash remain pending. Other editors, physical keyboard/Narrator interaction, an actual slow or failed accepted write, and combined backup/import close flows still require acceptance evidence. The capture return controller changes the modal state directly; it does not certify a physical button click.
