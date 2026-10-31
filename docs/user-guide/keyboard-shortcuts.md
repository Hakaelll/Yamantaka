# Customize keyboard shortcuts

Open **Settings → Shortcuts**. Choose a combination beside an action. A valid change takes effect after the preferences file is saved successfully. If the combination is already used, the screen names its owner and keeps the previous assignment. Change that other action first to free the combination.

The page currently configures 21 connected native actions. **Page Up** and **Page Down** remain reserved for structural reading navigation. Combinations are selected from a bounded list; normal text-editing shortcuts are not assignable choices. Page/history commands stay with text fields while editing. Reader commands apply to the focused comparison pane. Modal dialogs block commands except fullscreen.

## Default assignments

| Action | Default |
| --- | --- |
| Open PDF | Ctrl+O |
| Search library | Ctrl+Shift+F |
| Find in PDF | Ctrl+F |
| Previous / next match | Shift+F3 / F3 |
| Previous / next page | Left / Right |
| Switch reader pane | F6 |
| History back / forward | Alt+Left / Alt+Right |
| Zoom in / out | Ctrl++ / Ctrl+- |
| Actual size | Ctrl+1 |
| Fit width / page | Ctrl+2 / Ctrl+0 |
| Fullscreen | F11 |
| Hide reader controls | F8 |
| Next / previous reading | Ctrl+Tab / Ctrl+Shift+Tab |
| Close / reopen reading | Ctrl+W / Ctrl+Shift+T |

Ctrl++ also accepts Ctrl+= and the corresponding shifted plus-key events. Those aliases count as the same assignment during conflict checks. Held page, history, match, and zoom keys can repeat; toggle and open/close commands ignore repeats. Focus-mode tooltips and comparison-canvas accessibility labels show the active assignment.

## Restore and inherited preferences

**Restore default shortcuts** removes overrides for the connected native actions. It retains values belonging to other commands. Unknown preference entries survive ordinary edits and resets.

Known inherited assignments use the existing shortcutOverrides preference key and compatible command/token names. Invalid or conflicting active assignments use the complete native default map and show a warning. The raw values remain untouched until you choose a change. Restore the connected defaults before editing a conflicting map.

Seven Flutter-configurable commands are not connected to this native shortcut editor yet: command palette, go to page, document navigator, toggle panel, toggle bookmark, toggle chrome, and toggle immersive. Their saved overrides remain preserved and contribute to the displayed unavailable-command count. This page does not establish complete Flutter command parity.

See the [shortcut checkpoint](../validation/rust-shortcut-settings-2026-10-31.md) for the verified scope.
