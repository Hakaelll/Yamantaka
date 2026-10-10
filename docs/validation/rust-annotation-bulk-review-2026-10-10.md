# Bulk review in the reader Notes panel

Notes now supports selecting individual rows, selecting the current list page and clearing the selection. Selected rows have the existing violet control treatment and accessible selected-button semantics. The action group displays the selected count and offers Pending, Accept and Resolve in English and Spanish. Rows display their persisted editorial state beside the mark kind.

The fixed 80-entry selection belongs to the current repository page. The mutation freezes IDs from that page and submits the existing typed Review command through the application actor. The storage layer applies the whole batch transactionally. Filters remain part of the refresh query, so accepting pending notes removes them from a pending-only list without removing their PDF marks.

Editor drafts and accepted writes disable bulk actions. Success clears selection and refreshes the list; failure preserves it for recovery. New reads, filters and document changes clear selection so an index cannot identify an unrelated row later. Selection does not change PDF navigation. Global cross-document review remains a separate unfinished requirement.

The Notes composition now groups page-note creation and selection annotation, moves explanatory selection copy to its tooltip, and places export formats in one labeled row. This gives more space to notes and their review actions. The saved message describes updated notes rather than implying that a batch created one note.

## Verification

Two new SQLite-backed controller regressions prove that a frozen batch changes only selected IDs, preserves notes and passages, refreshes the active pending filter, increments geometry revision and releases governed memory. They also check editor/pending-write exclusion, rollback when a later selected record is missing, preserved failed selection, and selection reset for filters, pagination and document switches.

Formatting, warning-denied workspace/all-target Clippy and all locked workspace tests pass. Local log: `build/verification/annotation-bulk-tests.log`. The alternative preview build emits only its two expected output-path warnings.

Five final native wgpu captures use an isolated synthetic fixture:

- Dark Notes contains three selected rows among 181 annotations and exposes the review actions.
- Light Spanish Notes shows three accepted rows after successful publication of the mutation, with cleared selection.
- Combined text/tag/kind/pending filters contain 17 notes after accepting three of their 20 original matches.
- The 640 × 480 window at 200% text retains the three selected records, but the initial compact drawer does not demonstrate scrolling or keyboard access.
- Export controls still publish complete JSON with all 181 records after their composition change.

All four normal-size cases retain 181 PDF marks, page 0 and zoom 1.0. Executable, Cargo.lock, all UI Rust source, screenshot, control and exported-file hashes match the final capture manifest. Local directory: `build/rust-visual/annotation-bulk-native-final`. Preview executable: `build/verification/yamantaka-annotation-bulk-preview.exe`. Manifest SHA-256: `34605419427023076ADBA2C1B6353EFF1F9A17F5FD768DECAD3493501107AFB0`.

## Remaining scope

The capture hook sets synthetic selection and submits the real actor mutation. It does not certify physical pointer/keyboard interactions, Narrator, multi-monitor DPI or compact scrolling. Bulk tagging, cross-document review, mutation history and broader annotation acceptance remain pending. Selection currently spans one list page, not every filtered result. No final release or full rewrite completion is claimed.
