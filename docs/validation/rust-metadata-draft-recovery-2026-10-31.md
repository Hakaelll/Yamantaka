# Recover metadata drafts

Date: 2026-10-31. Scope: unsaved title and author input in the native metadata editor.

## Behavior and storage

The default-session snapshot now stores metadata drafts alongside readings, comparisons and new page-note drafts. Each metadata draft contains its document identity, the existing metadata-review revision and exact title/author input. The revision is the SHA-256 of the complete catalogued review record, including file details and enrichment evidence. It is not a fresh whole-PDF hash.

Version 4 adds `metadata_drafts`; versions 1 through 3 remain readable. A snapshot upgrades when it first stores a metadata draft and does not downgrade after that draft is resolved. Embedded workspace reading contexts remain version 1. The existing four MiB total bound remains in place. At most twelve distinct document drafts are accepted, each with title and author input bounded to 64 KiB. Invalid revisions, duplicate documents and unsupported records fail without replacing the previous session file.

An ordinary restart with session restoration enabled opens the last retained metadata editor after reading restoration settles. Checked graphics recovery also restores it without permanently changing that preference. Disabled restoration retains the drafts without opening their editors. Returning from the editor to the library retains unfinished input. Reopening a document's editor retrieves its draft. Explicit document removal clears its recovery context.

Matching review revisions restore the unsaved fields. A changed revision keeps the current catalogue values and shows the recovered text separately. Save and reanalysis are disabled until the user chooses Use recovered text or Keep current values. The first choice puts the old text into a new unsaved edit against the current review; the second explicitly abandons the recovered input. Neither choice writes the catalogue by itself. Existing guarded metadata mutations still validate the current review at commit time.

During graphics shutdown, metadata replies are consumed after the accepted-write barrier before capturing drafts. A completed save becomes a clean editor instead of a second recovery draft. Other retained drafts remain independent.

## Executed evidence

Offline workspace Clippy for all targets with warnings denied passed. The existing historical-schema compatibility test now also covers version-four round trips, exact Unicode/whitespace preservation, unknown root extensions, duplicate rejection, document removal and non-destructive rejection of malformed revision tokens. No standalone Rust test was added. Focused UI Clippy and the native build passed after the final capture-controller correction.

The native graphics-failure profile seeds a synthetic metadata draft while inspecting a PDF, destroys the wgpu device and restarts through Glow. All four draft fields must match. The [capture manifest](../screenshots/rust-metadata-draft-recovery/capture-manifest.json) records executable, source and artifact hashes. The controller keeps the reader visible until its raster is ready; an earlier attempt timed out after hiding the reader while still awaiting its refinement, before injecting the failure. Resume no longer replays the initial metadata-preview setup.

The restart verifier makes independent SQLite backup copies. It checks normal restoration, disabled restoration and a changed catalogue title. The changed-revision case must retain the original draft, show the current title and an explicit recovery choice, and disable Save. The post-exit JSON must still contain the exact draft. The [restart report](../screenshots/rust-metadata-draft-recovery/continuity-restart-report.json) and sibling controls record the three results. The source fixture's database, preferences and continuity hashes remain unchanged. Restart PNGs remain local artifacts.

## Remaining acceptance

Physical keyboard/Narrator interaction with both recovery choices, slow or failed metadata writes during actual device loss, missing-document recovery and interruption during atomic replacement still need acceptance evidence. Selection-based annotation drafts and edits to existing annotations still lack persistent recovery. This checkpoint does not establish full recovery, migration or Release acceptance.
