# Document quarantine checkpoint

Date: 2026-10-31. Scope: persisted reader-opening failures, blocked automatic lookups/covers, paged review and manual recovery. This is not complete startup recovery or release acceptance.

## Connected behavior

The native implementation reuses v11 `restore_blocked` and `last_failure_code`. Typed health requests reach the storage actor only after primary/reference opening ownership checks. Reported PDF opening failures persist a fixed `pdfOpenFailed` code. Password, cancellation, queue and governed-memory failures do not enter that classification. A matching successful reader opening clears its old block. Already healthy records use a read-only no-op transaction. Raw native failure text never enters the quarantine code.

Recovery reads one snapshot of at most 80 records plus the total. Existing unknown codes remain untouched and map to a general presentation reason. Source/failure review revisions hash the registered path, fingerprint, size, modification date, page count, missing flag, block and exact failure value. Release verifies that revision and commits its update and returned page together. A partial index covers blocked records in their display order without changing schema version or historical migration SQL.

Saved-tab and workspace opening now use a restorable-document lookup. Stored references are filtered before comparison restoration. New library cover requests check the same persisted policy. Committed changes pause or resume cover consumers, and a released unavailable cover can be requested again without invalidating an existing healthy image on every reader opening.

Tools → Document recovery exposes retry, frozen release review, pagination, refresh and missing-file guidance. Explicit retry uses the normal native opening path. Closing the dialog cancels reads but retains accepted writes. Application close waits for tracked health/release writes; write failures keep the window available to show the error. The source PDF is never written by this feature.

## Executed evidence

The retained storage compatibility test `overrides_win_and_queries_are_bounded_and_escape_wildcards` passed with a quarantine helper: one passed, five filtered, 6.14 seconds. It exercises classification exclusions, source-guard rejection, inherited/unknown reasons, stale review, cancellation, an injected after-update failure, release, successful retry state, 81-record pagination, and rollback when the post-write page cannot decode a malformed unrelated row. It also checks unchanged title/progress and SQLite integrity. A file-backed store is closed and reopened to verify that the block survives; after release, a separate read-only reopen confirms it remains cleared. No standalone test was added.

Binary Clippy with warnings denied and an offline native build passed. Application targets, including retained tests, compiled with the new repository capability. No broad workspace test suite was executed for this unit.

Five final controller-driven synthetic native scenarios passed their state/control assertions using the same request handlers as the UI:

| Case | Observed result |
| --- | --- |
| quarantine-review-small-200 | Preserved blocks open a named release review with no mutation at requested 640 × 480 logical size, 200% text, light theme and reduced motion; Close remains accessible |
| quarantine-released | One reviewed block is removed, the other remains, and the released book's automatic PDF cover returns while the still-blocked cover stays unavailable |
| quarantine-retry | An explicit native opening installs the PDF page and clears only that document's inherited block |
| quarantine-auto-blocked | A restorable lookup is rejected before the downstream canonical open; no reader tab opens and both blocks remain |
| quarantine-pdf-failed | A real PDFium opening of an invalid synthetic file records the third block with a typed native reason; original fixture bytes remain unchanged |

The recovery list and compact release review were visually inspected. Fixtures and stores are synthetic. The [capture manifest](../screenshots/rust-document-quarantine/capture-manifest.json) records source, lockfile, executable and artifact hashes; publication checks them and embeds PNG provenance. No Computer Use was used.

## Remaining acceptance

Native process death before an opening-failure event is not covered by this path. Page-render failures, all inherited restoration/comparison scenarios, close-during-write timing, real pointer/keyboard/Narrator interaction and physical DPI remain unproven. Review checks registered database state, not a fresh full-file hash. Quarantine does not move PDFs or repair startup records, malformed preferences or damaged libraries. Those startup-recovery flows, the broader PDF/performance corpus and the final Windows release remain required work.
