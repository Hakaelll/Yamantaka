# Reference bulk note review checkpoint

Reference Notes now supports selecting individual rows or the current list page, changing their review state together, and adding one tag without replacing existing tags. The main and reference readers share AnnotationState.show_bulk, selected-ID validation, typed review and additive-tag mutations, and the existing transactional application actor.

Selection belongs to one immutable list snapshot and is limited to 80 rows. Filter changes, list-page requests, and successful writes clear it. An open editor, pending query or write, or another blocked note operation disables bulk actions. Reference writes use the same scoped reply routing and document-level list and mark refresh introduced with its individual editor. Main bulk actions now also disable during an active reference write.

The controls stay in the Notes popup. Review and tag controls appear when a selection exists. Each reference row uses the same selection control as the main list; navigation, edit, and removal retain their existing behavior.

## Evidence

The final native binary compiled offline, and cargo fmt --all --check passed. Four scenarios completed through tool/capture_rust_ui.ps1:

- reader-reference-note-bulk loads 82 synthetic reference notes and selects the first 80. The Spanish dark-theme capture checks visible Clear selection, Accept, and Resolve controls. Two source-document notes remain unselected.
- reader-reference-note-bulk-saved accepts those 80 notes and adds Reference batch through separate typed mutations. All 80 retain their Existing study tag. The sequence reads the next batch, confirms its two notes remain pending with only their original tag, then returns to the first batch and verifies the saved review states and both tags. The main document retains two unchanged pending notes without tags. The final capture uses English and light theme.
- annotation-bulk-filtered retains the existing primary filtered-review scenario after extracting the shared component.
- annotation-bulk-tag-saved-es retains the existing primary additive-tag scenario in Spanish and light theme.

The [capture manifest](../screenshots/rust-reference-note-bulk/capture-manifest.json) records the executable, Rust sources, images, and control hashes. They match the final working tree and binary before publication. Scenarios set selection and tag values programmatically, then use the same mutations and real SQLite actor as the product. They do not certify physical pointer selection, text input, keyboard traversal, or Narrator. Only synthetic documents and records were used. No Computer Use, new standalone tests, or broad workspace suite were used.

## Remaining acceptance

Reference export controls remain pending; the main and library Notes tools retain export. Compact bulk-tag interaction, mutation failure and rollback presentation, rapid cancellation and concurrent edits, resource-pressure behavior, physical DPI and accessibility, complete performance comparison, and Release packaging remain pending. This checkpoint does not establish full rewrite acceptance.
