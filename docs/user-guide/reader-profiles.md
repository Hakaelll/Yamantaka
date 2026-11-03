# Reader profiles

Open **Settings → Accessibility → Reader profile** and choose **Electronic ink** for a reader with white paper, single-page navigation, light controls and no page shadows or animated scrolling. The same profile applies to both panes when comparing PDFs. PDF content keeps its original colors.

The profile changes presentation. Your document's saved environment, layout, book pairing, margins, rotation and zoom remain available. Choose **Standard** to use those document settings again. The library keeps its existing appearance and 3D covers.

The choice is saved locally and recognizes the `visualProfile` preference imported from Flutter. Profile controls wait for an in-progress document opening or comparison preparation to finish. No PDF is modified.

**Reduced motion** also disables animated scrolling and drag inertia in both reading panes when using the Standard profile. Wheel input is applied without egui's additional smoothing, and native UI state transitions use immediate feedback. Direct scrolling, dragging and navigation remain available. This change has not undergone a new runtime check.

This is a software presentation profile. Refresh behavior on physical electronic-ink displays has not been certified. See [native profile validation](../validation/rust-electronic-ink-2026-11-03.md) for the executed checks and remaining limits.
