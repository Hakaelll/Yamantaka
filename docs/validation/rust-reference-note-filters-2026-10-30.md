# Reference note filters checkpoint

The reference Notes section now supports the same combined filters as the main reader: passage or note text, exact tag name, mark type, and review state. The filter form stays collapsed until requested. Active filters show a count and Reset. Applying or resetting starts at the first 80-record batch.

Both panes share AnnotationState.show_filters and its existing query builder. The presentation emits a scoped Read action after applying the draft. Each pane retains its own filter draft, applied values, selected rows, record snapshot, and cancellation token. An open annotation editor or accepted write disables filter changes. Replacing a pending read cancels it before submitting the new query. Filtering the list does not invalidate the geometry-backed PDF mark cache.

## Evidence

The native binary compiled offline, and cargo fmt --all --check passed. Four scenarios completed through tool/capture_rust_ui.ps1:

- reader-reference-note-filters creates, edits, and removes reference notes through the real SQLite actor, then applies all four filters to its final tagged page note. It checks one matching record, changes the text to obtain no matches, resets to recover the record, and reapplies the matching filters. The main document retains its two notes, page zero, and no active filters. The final Spanish light-theme capture checks visible Active filters and Reset controls.
- reader-reference-note-filters-empty follows the same sequence and finishes with four active filters and no matching records in English and dark theme. The main document stays unchanged. Reset remains visible.
- annotation-filter-light-es exercises the existing primary combined-filter scenario after extracting the shared presentation.
- annotation-filter-empty retains the existing primary empty-result presentation.

The [capture manifest](../screenshots/rust-reference-note-filters/capture-manifest.json) records the executable, Rust sources, images, and control files. Their hashes were checked against the final working tree and binary before publication. Draft assignment and filter application are programmatic controller actions. These scenarios do not certify physical typing, pointer use, or keyboard focus traversal. They use isolated synthetic records and PDFs, with no Computer Use or personal library access. No standalone tests or broad workspace suite were added or run.

## Remaining acceptance

Reference-specific bulk actions and export remain pending; the main and library Notes tools retain those capabilities. Large annotation-list latency, memory-pressure recovery, rapid filter cancellation, compact filter-form interaction, physical DPI, Narrator, and full rewrite performance acceptance remain pending. This checkpoint is not a Release approval.
