# Selection-note draft recovery

## Behavior

Continuity version six retains unfinished new selection annotations in the primary and reference readers. The draft stores native character boundaries, the exact selected passage, rotation, fingerprint, annotation style, note, tags and fallback confirmation. It never stores page geometry or rendered images. Versions one through five remain readable, root extensions survive writes, and version six remains sticky after drafts are cleared. Named workspaces reject draft payloads.

Recovery waits until the matching reader has an image. The application actor requests fresh PDF text and geometry through the dedicated PDF worker, one page at a time, below foreground rendering priority. A governed selection snapshot requires every source page in order and an exact passage match. Matching drafts reopen unsaved in the existing editor. Recovery neither changes the live pointer selection nor creates an annotation automatically.

A changed or unavailable source opens the existing separate recovery panel. Copy includes the full original draft, Retry checks again, and Discard only abandons the draft. An absent reference or disabled restoration retains its work. Document removal forgets matching recovery records and cancels the pending scan. During an asynchronous scan the raw draft remains in continuity, including when the reader changes or a graphics failure interrupts recovery.

The scan admits its page-reference buffer through the text governor before allocation, retains each page's existing lease, and checks page identity and cancellation at every step. It does not enqueue the entire page range or perform PDF calls on the UI thread.

## Executed checks

Offline workspace Clippy for all targets with warnings denied, formatting, the native build and the retained historical-schema/workspace compatibility test passed. The existing compatibility helper now covers version-six round trips, Unicode, unknown root extensions, sticky versions, duplicate editor rejection, invalid ranges, document removal and non-destructive rejection of malformed snapshots. No standalone Rust test was added.

Two native synthetic graphics-failure profiles passed. Both readers reopen their exact unsaved selection drafts with fresh geometry. The second submits the primary save just before destroying the wgpu device: exactly one highlight is committed, its draft stays closed, and the reference draft returns. The executable, lockfile and every manifest source hash were checked against the current tree. See the [capture manifest](../screenshots/rust-selection-note-recovery/capture-manifest.json).

Four independent SQLite-backup restart profiles passed: normal startup, an absent reference, a mismatched frozen passage and disabled restoration. Normal restoration checks exact editor fields and rebuilt source-page counts; the mismatch keeps the original text separately with Copy available, while the reference still recovers. The mismatch deliberately changes the primary draft's frozen passage in the isolated continuity JSON. It does not replace the PDF. Post-exit drafts retain all fields exactly, and the source fixture's database, preferences and continuity hashes remain unchanged. See the [restart report](../screenshots/rust-selection-note-recovery/continuity-restart-report.json). Restart PNGs remain local artifacts.

The captured restored editor and mismatched-passage panel were visually inspected. This verifies the existing native panels in these capture conditions, not physical interaction or general visual acceptance.

## Remaining acceptance

Physical Copy/Retry/Discard interaction, multi-page and geometry-poor recovery on a broader PDF corpus, original-file replacement during recovery, repeated close/reopen cycles and full resource/performance acceptance remain pending. The mismatch scenario must distinguish a changed frozen passage from a replaced PDF. Recovery input strings and shared-handle overhead still need complete governor accounting. The UI checks the catalogue fingerprint, not a freshly computed whole-file digest. This checkpoint does not certify whole-rewrite or Release acceptance.
