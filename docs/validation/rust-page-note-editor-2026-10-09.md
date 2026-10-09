# Page notes without text selection

The reader Notes panel now creates a note on the current page without requiring selectable text. The editor freezes the document identity, page and rotation. Saving uses the existing owned annotation command, transactional SQLite repository and consented search updates.

The domain constructor creates a PageComment with no selected text, context or rectangles. It reserves memory before copying source identity and note strings, then accounts for retained capacities. Shared validation checks identity and the 4000 UTF-16 unit limit. The constructor accepts an empty optional note for legacy compatibility; the creation interface requires meaningful text before saving.

This route requests no PDF text snapshot and needs no geometry fallback confirmation. It works with a scanned PDF and does not provide OCR or modify the original PDF.

## Verification

Two domain regressions check Unicode trimming, retained page identity and rotation, absent geometry/context, input limits, denied memory admission and lease release. An application integration test saves and searches the note through the actor with real SQLite and confirms that no native text extraction request occurs. A SQLite-backed editor test rejects a blank note, saves the trimmed note and closes the editor after success.

Formatting, warning-denied workspace/all-target Clippy and workspace tests pass. The full local test log is `build/verification/page-note-editor-tests.log`.

Two native wgpu captures are local in `build/rust-visual/page-notes-native`:

- `page-note-scanned-editor` shows the dark English editor on the synthetic scanned page.
- `page-note-scanned-saved-es` saves the note through the actor and shows the stored record in the light Spanish interface.

The saved state contains one annotation, a closed editor and no pending command. Both captures have no loaded page text or enabled text selection. Manifest SHA-256 is `D1356C91599C3CCF696064C8FE061436BE22018F0AF3C87719A6E87D3C1DC723`. Executable, Cargo.lock, source, image and control-state hashes match, and control bounds are finite. Documents and notes are synthetic. These scripted captures do not certify physical DPI, Narrator or complete manual interaction.

## Remaining scope

Complete paint coverage beyond the paged list, intrinsic/directional line acceptance, distinct note/marker affordances, mutation history, annotation tags, global review and exports remain pending. Cross-page pointer/autoscroll, broader keyboard and compact scrolling acceptance, OS clipboard, physical DPI, multi-monitor, performance and final release gates remain open.
