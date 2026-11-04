# Configurable native shortcuts checkpoint

Date: 2026-10-31. Scope: configuration and routing of 21 connected native commands, including focused comparison readers. This checkpoint is not full Flutter command parity or release acceptance.

## Behavior and persistence

Settings now has a sixth category, Shortcuts. A shared definition table supplies stable command IDs, defaults, labels, scope, text-focus guards and repeat policy. The editor uses a bounded choice list, reports the action that owns a conflicting combination, and keeps the previous assignment. Normal windows align action labels and selectors in compact rows; narrow windows stack them.

The connected commands cover PDF opening, library and in-document search, result stepping, paging, comparison focus, navigation history, zoom, actual size, fit width/page, fullscreen, reader focus and tab cycling/closure/reopening. Actual size and fit width route to the focused reader through existing zoom controls. The existing shortcutOverrides preference key accepts Flutter's encoded JSON string and compatible IDs. Rust-specific commands use namespaced IDs. Unknown entries and values remain in the valid raw object through edits and native-default restoration. Original legacy preferences stay separate.

Known invalid tokens or duplicate/equivalent active bindings select the complete native default map and show a warning. They do not silently remove an action or rewrite preferences while loading. The editor requires restoring the connected defaults before editing such a map. An accepted assignment saves a cloned preferences object atomically before replacing the active map. A failed save retains the old active preferences and binding. Restore removes only keys belonging to the 21 connected commands. An explicit reset of malformed JSON replaces that malformed override value.

Routing consumes exact Ctrl/Shift/Alt key events, preserving Ctrl+F versus Ctrl+Shift+F. Plus and equals-key variants are equivalent and conflict accordingly. Page Up/Down remain reserved. Page/history navigation does not consume text-editing keys. Reader-only actions stay inactive outside the reader; modal dialogs block commands except fullscreen. Repeat is enabled for navigation, result stepping and zoom, while open/close and toggles reject repeated keydown events. Reader-focus tooltips and comparison accessibility labels use current assignments.

## Executed evidence

No standalone test was added and no broad suite was run. Native capture setup performs a bounded self-check using actual egui input frames and an isolated synthetic preferences file. It checks:

- Custom Ctrl+Right routing and removal of the former plain Right binding.
- Modal and non-reader rejection, reserved Page Down navigation, and exact Ctrl+Shift+F routing.
- An actual focused TextEdit retaining Ctrl+Right and plain Right, including after a global action is rebound to Right.
- Fullscreen while modal, conflicting Ctrl+F rejection, equivalent plus-key conflict, and rejection of reserved Page Down assignment.
- Malformed/conflicting inherited maps falling back to complete defaults.
- Unknown JSON value preservation through reset, atomic preferences save/load, and reloading three custom bindings.

The initial headless probe was corrected to clear deliberately unused egui texture deltas before dropping its output. Product renderer ownership was unchanged. The first visual pass also corrected arrow-key captions and compacted wide rows. Final offline build, formatting, and binary Clippy with warnings denied passed; the final label-alignment and text-navigation guard edits were compiled in the final capture binary.

Four final native cases completed their input/state assertions and control-bound checks:

| Case | Observed result |
| --- | --- |
| shortcut-settings | Spanish light settings, 21 active definitions, three persisted custom assignments and one preserved future entry |
| shortcut-conflict-small-200 | Conflict names Find in PDF, retains Ctrl+Right, and shows the message in a requested 640 × 480 logical viewport with 200% text and reduced motion |
| reader-custom-shortcuts | Actual native PDF comparison uses F7 to switch pane, Ctrl+Right to page and Ctrl+D to zoom; independent page/search state and focus are preserved |
| reader-reference-shortcuts | Existing default-key comparison scenario still completes with its F6 accessibility label and independent readers |

The settings layout, small conflict state and capture provenance were inspected. All captures use generated acceptance PDFs and isolated synthetic preferences. Final source, lockfile, executable and artifact hashes were checked before publication. PNGs embed their origin metadata. See the [capture manifest](../screenshots/rust-shortcut-settings/capture-manifest.json).

## Remaining acceptance

At this checkpoint, seven inherited configurable commands were unconnected to the native editor: command palette, go to page, document navigator, toggle panel, toggle bookmark, toggle chrome and toggle immersive. Their stored overrides remained preserved. Physical keyboard layouts, AltGr behavior, pointer selection in every combination menu, save-failure injection, Narrator, monitor DPI and complete original-command parity remained acceptance work. No Computer Use or personal data publication was performed.

## Later implementation status: 2026-11-04

The later [reader-command checkpoint](rust-reader-commands-2026-10-31.md) and
[command-palette checkpoint](rust-command-palette-2026-10-31.md) connect those
seven commands. Current source inspection finds all 16 IDs in Flutter's
configurable command set among the native editor's 30 definitions. The reader
dispatcher routes the six reader controls, while the command palette has its
own application action. The preserved-unavailable count applies to other stored
IDs, not to these seven commands. This follow-up is source inspection, not a new
runtime or physical keyboard test; the remaining acceptance above is unchanged.
