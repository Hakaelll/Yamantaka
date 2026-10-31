# Frozen automation action review checkpoint

Date: 2026-10-31. Scope: preparing and displaying an immutable action plan after simulation. Confirmed execution, execution journals, recovery, cancellation rollback, and undo remain pending. This is an intermediate step toward full automation parity.

## Connected behavior

Tools → Saved rules → Simulate → Review actions now resolves all six inherited action kinds. The review displays document and per-document action counts, interpreted reading-state/favorite effects, destination names, and destinations that would be created. Closing the review releases the plan and returns to the simulation. Preparation creates no tags, collections, memberships, workspace tabs, journals, or document changes.

Preparation runs on an independent application worker. It opens a read-only SQLite connection and captures one transaction snapshot. Simulation matches now retain a typed SHA-256 signature of the effective title/author, source identity, path, file name, missing state, and condition membership observed during the scan. Preparation verifies these signatures and the complete saved-rule revision. It then retains signatures of all v11 document columns and existing target rows for enforcement by the future execution boundary. The plan owns its immutable simulation and governed data through Arc ownership.

Tag and collection resolution prefers an exact ID, then the normalized name. Missing destinations receive planned UUIDs without being inserted. Repeated new names within the same target kind share one planned identity. Workspaces and reading-queue destinations must already exist; Unicode case matching is supported, and ambiguous workspace names require an ID. Existing tag/collection lookups use their ID/normalized-name predicates; workspace fallback scans names on the worker because the inherited schema has no normalized workspace-name column.

Reading state accepts finished/terminado and pending/pendiente/notstarted. Favorite preserves Flutter's interpretation: 0, false, and normal disable it; other nonempty values enable it. Unknown reading states and missing workspaces reject the complete plan.

Exact-duplicate review requires a complete simulation without unverified sources. The application worker rereads each matched source, checks its recorded source revision, and compares its full SHA-256 with the simulation proof before preparing the database snapshot. This is a review-time observation. Confirmed execution must repeat source verification and enforce the document/target revisions; no execution claim is established here.

Cancellation is checked during preparation and file reads. Leaving the manager discards late review results, including a result queued before cancellation. Simulation results are also discarded after leaving, even if the manager is reopened before their reply arrives. Native window closure requests cancellation and waits for acknowledgement. Blocking operating-system reads may delay cancellation. Idle preparation does not request continuous repainting. Native SQLite/driver allocations remain outside the controlled memory ledger.

## Executed evidence

The retained application test `file_comparison::tests::background_comparison_rejects_partial_hash_collision_and_confirms_full_copy` was extended through the existing simulation verification chain. Its focused offline run passed. The new checks use real simulation/preparation workers and an isolated file-backed v11 store. They cover all six action kinds, existing-ID resolution, new destinations, repeated-name identity reuse, Unicode workspace lookup, interpreted scalar values, full document revision count, observer data-version stability, absence of premature collection creation, pre-start cancellation, memory admission failure/release, changed matched metadata, unavailable workspaces, unsupported reading states, and a changed duplicate source after simulation. Original synthetic source bytes are restored and checked by the retained chain. No standalone tests were added.

The final binary built offline. Formatting and binary Clippy passed with warnings denied. Four native cases completed through the normal application runtime:

- `rule-review-actions`: ten synthetic documents, seven declared actions spanning all six kinds, Spanish dark theme. Repeated new tag actions show the same planned name.
- `rule-review-small-200`: the same plan in Spanish light theme, reduced motion, requested 640 × 480 window and 200% text scale. The sequence scrolls to Close review and verifies its bounds.
- `rule-review-missing-workspace`: a missing workspace rejects the entire plan, preserves the simulation, and leaves Review actions available for retry. English light theme.
- `rule-simulation-cancel`: cancellation after real scan progress still retains no partial match list after the simulation-result ownership change.

The [capture manifest](../screenshots/rust-rule-review/capture-manifest.json) records executable, Rust source, Cargo.lock, image, and control hashes. Native sequences assign fixture values and invoke controller actions programmatically; they do not prove physical keyboard/pointer behavior. The captures use isolated synthetic stores. No Computer Use or personal library data were used. No broad workspace suite was run.

## Remaining work

Confirmed writes must enforce this plan, persist a compatible operation journal, handle cancellation/failure atomically, recover after restart, and undo only changes whose postconditions remain valid. Import-triggered context, physical close/reopen and rapid-context interaction, memory-pressure UI recovery, Narrator/DPI, paired performance measurements, and final Release packaging remain pending. These requirements are preserved in the rewrite scope; preparing a plan does not satisfy them.
