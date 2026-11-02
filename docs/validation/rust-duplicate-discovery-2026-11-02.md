# Native identical-copy discovery

## Connected behavior

**Tools → Identical copies** opens a dedicated view. **Find copies** starts a cancellable background scan; **Review group** shows the catalogue entries and locations; **Open reading** uses the existing canonical document-opening flow. Leaving the view cancels pending work and releases retained results. No permanent library bar or sidebar is added.

An independent read-only SQLite transaction supplies ordered pages of at most 80 candidate records. The existing fingerprint index narrows candidates with equal inherited fingerprints and file sizes. Each available candidate is then read through the guarded full-content hash worker. Size, inherited fingerprint and catalogue modification date must match the observed source. Only equal freshly calculated full SHA-256 digests and sizes form a group.

Reports describe a point in time. They retain source revisions and full proofs for later review; they do not authorize merging files after they change. Original files are opened read-only and never moved or deleted. Missing, inaccessible or changed candidates increment the unverified count. Cancellation, memory pressure or a worker failure never publishes an incomplete report as complete.

Candidate records, hash buffers, verified source paths and result storage reserve memory through the application governor. The job has one worker, a bounded terminal channel and a 100,000-candidate limit. Cancellation is checked while reading file chunks. Shutdown cancels and joins the job. The interface polls progress only while a request is active, rejects unrelated replies and presents bounded pages of groups and members.

## Verification

The retained application test now uses the actual storage reader and discovery worker. It checks two byte-identical files and a different-tail file sharing their legacy fingerprint, exact membership and hashes, cancellation before admission, 85 candidate records across the 80-row boundary, read-snapshot consistency after a concurrent catalogue edit, and release of all tracked memory. No standalone test was added.

Executed checks include all four retained application library tests, workspace all-target Clippy with warnings denied, formatting and an offline native build. [Five native captures](../screenshots/rust-duplicate-discovery) exercise the normal actor path: a matching group, its two members, a compact light ES view at 640×480 with text at 200% and reduced motion, an empty result for different-tail candidates, and Tools navigation. Capture assertions check exact group/match/candidate counts and visible primary controls. The compact view uses vertical scrolling for results; this does not prove physical scrolling input.

An initial compact capture exposed an off-screen search action. The final compact header uses a named back icon and shorter introductory copy; Back to library and Find copies are fully visible. The Tools menu was also compared with a native capture from the frozen pre-change executable at commit `6c93c77`. No gallery geometry or cover-physics code changed.

The capture manifest records final Rust source, Cargo.lock and executable hashes. PNG text metadata records synthetic provenance while preserving native pixels; the manifest retains original and published image hashes. No personal paths or library records are published.

## Limits

Discovery depends on current inherited catalogue identities to select candidates. Uncatalogued files and records with incompatible stored identities require import or revalidation first. The view reviews and opens groups. Consolidation, metadata conflict resolution, pre-merge backup, journal and undo still need implementation. Large-library timing, physical input, Narrator, monitor DPI and final Release acceptance remain pending.
