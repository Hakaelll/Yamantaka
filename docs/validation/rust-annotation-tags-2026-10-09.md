# Annotation tags in the reader editor

The Notes editor now accepts optional comma-separated tags when creating a passage annotation, a multi-page batch or a standalone page note. Stored notes show their tags and populate the field when edited. Saving can replace or clear the links in the same transaction as the note, kind, color and review state.

An omitted tag change preserves existing links. The UI uses this route when the field matches the stored tags, including inherited names that contain commas or exceed the new-input length limit. Changed input trims empty segments, deduplicates normalized names and checks twelve distinct tags and 160 UTF-16 units. Invalid input keeps the draft and disables saving instead of silently dropping tags. This follows Flutter's comma-separated editor and shared normalized tag identity, with explicit rejection instead of Flutter's silent twelve-tag truncation.

Owned actor commands carry the replacement list to the existing repository. The repository preserves annotation identity, source geometry and extension columns, reuses canonical tag rows and refreshes consented FTS entries inside the mutation transaction. An empty replacement removes annotation links without deleting shared tag definitions.

## Verification

An additional SQLite regression replaces tags while preserving geometry, checks tag search, injects a late annotation-tag insertion failure and confirms rollback of note, links and search, then clears links and verifies search removal. The actor's held-native-work regression now replaces tags through the owned Edit command and searches for the new Unicode tag.

A SQLite-backed editor regression checks trimming, normalized duplicates, supplementary Unicode length, thirteen distinct tags, creation, untouched inherited comma-containing names, replacement, clearing and release of governed source/snapshot payloads.

Formatting, warning-denied workspace/all-target Clippy and workspace tests pass. The full local log is `build/verification/annotation-tags-tests.log`.

Five native captures are local in `build/rust-visual/annotation-tags-native`. Dark English and light Spanish editors show the populated tag field, length/count guidance and Save action. The saved Spanish multi-page scenario shows the same two tags on both records; the scanned page-note scenario also shows its stored tags. The compact 640-by-480 capture at 200% shows the scrollable Notes drawer at its initial position. It does not prove interaction with controls below the fold.

Manifest SHA-256 is `5BB8ECD84077F7A7A31C636C4ECBA13ECFFB4A1321B44DDD2EF592F2294A368B`. Executable, Cargo.lock, source, image and control-state hashes match, and control bounds are finite. Captured data is synthetic.

## Remaining scope

Annotation tag filters and bulk actions, global review and Markdown/JSON export interfaces remain pending. Mutation history, intrinsic/directional mark acceptance, distinct note/marker affordances, broad keyboard/compact scrolling acceptance, physical accessibility, performance and final release gates remain open.
