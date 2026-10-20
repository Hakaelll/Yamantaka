# Guarded library removal coordinator

Date: 2026-10-20. Scope: close-before-delete orchestration and stale-opening rejection for the [atomic storage operation](rust-library-removal-storage-2026-10-20.md). The selection menu does not invoke this coordinator yet. Confirmation, UI consumer retirement and reader-continuity reconciliation remain required before user-facing removal is complete. No release acceptance is claimed.

## Runtime contract

`Action::RemoveFromLibrary` validates and deduplicates up to 80 catalogue identifiers, then captures their source records in one actor admission. Missing records become individual failures. One native close ticket runs at a time; SQL removal starts only after successful native closure and another check of path, fingerprint and page count. A changed source or native/SQL failure preserves the local record. Missing-file records can still be removed.

Cancellation stops new native admissions. After an admitted close, an observed cancellation skips SQL removal; the storage transaction also checks cancellation before commit. An admitted close reaches its real terminal outcome. `BulkReport.retired` records successful native closures separately from removed IDs, including a close followed by cancellation, source conflict or SQL failure. The caller must therefore reconcile its presentation for retired readers even when their records remain available for reopening. Successful removals remain saved, while unattempted IDs remain in the report. Progress contains metadata only and uses a coalesced scoped snapshot.

The coordinator shares the bulk-write reservation. Admission cancels selected pending reader requests and ordinary cover consumers, including their native render request scopes. The actor parks on ticket readiness while continuing catalogue queries. Successful closes release visible-reader ownership and advance native capacity. Index work yields before admission and retains its paused state while removal owns the reservation.

An already-running import/reanalysis causes immediate busy rejection, leaving its request intact. The coordinator does not wait behind an import that may itself be waiting for reader capacity. Imports submitted later retain their durable intents and defer admission until removal finishes. This is explicit serialization, not a claim that all removal/import interleavings are accepted.

Opening now validates a captured document against its current catalogue source before native admission and after native completion. A removed or changed record cannot be opened through a stale `Document` value. If the catalogue changes during a successful native opening, a separate close ticket retires that rejected opening before the terminal failure is delivered. Later openings of the same native identity wait for this cleanup. Visibility is restored without replacing a newer opening's pin set.

## Executed evidence

- The retained `library_bulk::tests::page_selection_and_bulk_operations_keep_fixed_targets_and_partial_results` case passed. Its new native helper uses real PDFium behind controlled asynchronous reply gates and an independent file-backed catalogue inspector. No standalone test was added; the retained Rust source count remains 55.
- While the first close reply was held, its catalogue record and native text remained readable and an actor query returned all four documents. An opening of that retiring ID was rejected before native admission.
- After the first real native close, its record was removed. Cancellation during the second admitted close retained the second record while reporting both native closures, the first successful removal, the missing-ID failure and the remaining ID. Duplicate IDs were processed once; a pre-cancelled request admitted no close.
- An injected native-close failure kept the record and its actual reader usable. An injected SQL failure after a real close reported a retired reader and preserved its saved title. A fingerprint change during the close gate also preserved the newer source record.
- A durable import intent submitted during removal remained pending until admission could resume. The helper cancelled that intent without disturbing the removal outcome.
- A stale removed/changed document value did not reach native opening. A record removed through a second catalogue connection while native opening was held produced a rejected-opening close, then a scoped failure; no `Opened` result revived it. Actual native text requests confirmed that the rejected handle was closed.
- After runtime/worker shutdown, tracked PDF-buffer usage was zero and the original fixture bytes matched their saved pre-run bytes.
- All eight retained `import_resumption_runtime` scenarios passed with the opening guards. The existing reader-preemption scenario additionally verifies immediate busy rejection of removal during a held import without cancelling that import.
- Workspace formatting, all-target offline Clippy with warnings denied and the offline native build passed. UI source/layout is unchanged, so no new screenshot batch was generated. The full workspace suite was not run for this checkpoint.

## Remaining acceptance

Connect explicit confirmation, stop the selected UI consumers before submission, reconcile all retired reader/tab/session contexts, persist coherent continuity, refresh selection/organization/search state and expose partial outcomes/cancellation. Verify the connected flow through native captures and physical interaction. Owned-cache/auxiliary-record reclamation, broader import/index/fault stress, large indexed-corpus deletion latency and full migration/performance/accessibility/release gates remain open.
