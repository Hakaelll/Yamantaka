# Reading context restoration adapter

The application can now decode a history context object into an immutable, validated restoration plan. The adapter retains Flutter's missing/null-field defaults: zoom 1, single-page document mode, ivory paper, zero vertical scroll and a closed panel. An absent context returns no plan. This checkpoint prepares restoration; it does not yet apply a plan to the native reader.

Known fields are decoded directly into a typed context without constructing a generic JSON object. Unknown extension fields are ignored by the decoder and remain in the original governed history object. Known text is limited before copying into owned strings: 128 bytes for layout/mode/theme and 512 for a panel name. JSON parser scratch allocations are outside the history governor; no complete parser-memory bound is claimed.

The domain validates finite positive zoom, nonnegative finite vertical scroll, representability by the native reader's floating-point coordinates, supported layouts/modes/themes and explicit panel names. Invalid types, unsupported future values, oversized known fields and ambiguous duplicate known keys fail recoverably instead of resetting the snapshot. Both `facing` and historical `spread` spelling are supported. The shared layout parser now serves document preferences and restoration.

Applying a valid plan returns a cloned per-document preference record with the saved zoom, layout, mode, theme and scroll. The original record is unchanged. Document identity, rotation, cover/initial-side choices, margins and controls behavior survive. The saved zoom uses custom zoom mode so fitting does not override it; the pending native integration must verify this on the real viewport. The updated copy receives no persisted timestamp until a later save. Panel selection is a separate typed value, including an explicit closed state.

## Verification

Five application regressions cover missing/null/default fields, unknown nested extensions, partial contexts, preserving unrelated current preferences, all supported panel names, layout aliases, recoverable invalid/future values, floating-point overflow/underflow, negative scroll, duplicate known keys, escaped strings and text limits. A 400 KB unknown extension does not become a field in the restoration model.

All 355 locked workspace tests pass. Formatting and warning-denied workspace/all-target Clippy pass. Test evidence is `build/verification/reading-context-adapter-tests.log`.

## Remaining integration

The plan still needs ownership across canonical lookup/native opening, application after per-document preferences load, a one-shot viewport scroll target, panel restoration and a visible restoration action. Native captures and interaction acceptance are pending; the existing history source-opening action still opens only its saved page. Individual history deletion, category cleanup, recently closed documents, startup/crash recovery, goals, workspaces and the full release gates remain unfinished.
