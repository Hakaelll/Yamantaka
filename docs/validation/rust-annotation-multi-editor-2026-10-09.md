# Connected multi-page annotation editor

The reader editor now freezes the complete selected page range and submits an atomic annotation batch. It no longer rejects a selection merely because its endpoints are on different pages. Every source page must be ready; missing pages cannot become a partial save.

A shared TextSelectionSnapshot owns the ordered page references, their array reservation and an independently governed joined passage. Copy and annotation preparation use the same snapshot admission path. Transferring copied text releases the temporary page references and array reservation. The editor retains its snapshot while the live selection changes or clears.

The passage preview shows all selected text with page breaks. A multi-page caption explains that the entered note applies to each selected passage. Each page keeps its own geometry, context and identity when the batch is prepared. Any nonempty segment with unreliable geometry requires explicit confirmation for the affected page notes. Empty boundary segments do not trigger fallback. Successful actor replies close the editor and display the separate stored records.

## Verification

Two domain snapshot regressions check reversed endpoint ordering, Unicode page joining, source/text lifetime separation, missing/repeated/reordered/extra pages, failed source results, denied admission and release of partial reservations.

An additional SQLite-backed editor regression freezes two selected pages with different geometry coverage, verifies the joined Unicode passage, requires fallback confirmation, saves both records, checks the page-comment geometry policy and releases the snapshot after success. The previous three editor regressions also pass through the batch command.

Two native wgpu captures are local in `build/rust-visual/annotation-multi-native`:

- `annotation-multi-editor` shows text extracted by PDFium from both pages of the synthetic mixed-size PDF in the actual editor.
- `annotation-multi-saved-es` saves that passage through the actor and SQLite, then shows the two stored records in the light Spanish interface. Captured state confirms two results, no pending command and a closed editor.

The native scenario performs the existing pointer preview on the first page, extends its synthetic selection endpoint to the second page and waits for its actual asynchronous text extraction. It proves editor/storage integration with both native source pages. It does not prove a human cross-page pointer gesture or autoscroll.

Manifest SHA-256 is `EAC835D8C204A407B11B09795FB41766B40DD901F601E4FB387BAED627221491`. Executable, Cargo.lock, source, image and control-state hashes match, and control coordinates are finite. All captured documents and notes are synthetic.

Formatting, warning-denied workspace/all-target Clippy and workspace tests pass. The local full log is `build/verification/annotation-multi-editor-tests.log`.

## Remaining scope

Cross-page pointer/autoscroll and broader keyboard interaction acceptance, page notes without a text selection, complete paint coverage, intrinsic/directional line acceptance, distinct note/marker affordances, mutation history, tags, global review and exports remain pending. Large-passage latency, compact editor scrolling, OS clipboard, physical DPI, Narrator, multi-monitor and final release gates remain open.
