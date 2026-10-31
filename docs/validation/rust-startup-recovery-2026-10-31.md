# Startup recovery checkpoint

Date: 2026-10-31. Scope: a selected Rust store whose preferences or database cannot open. This checkpoint does not establish complete recovery or release acceptance.

## Connected behavior

Production startup now validates preferences before opening the database. A preferences decode/version failure or database opening failure opens an independent native recovery window, before PDFium and the library runtime start. The primary instance guard stays owned until recovery and installation finish. No default preferences are silently saved over unreadable state.

The window reuses the cancellable restore worker and existing backup review, hash revalidation, preparation, staged installation, safety-copy and journal machinery. Confirmation is required before preparation. Installation occurs after the recovery window and its worker have closed. The previous store remains in a separate safety directory. A failed review clears the old review so it cannot be confirmed accidentally. Explicit exit cancels work and clears a pending handover. A window close request before handover also cancels the worker.

The footer uses shorter labels when enlarged text makes the logical viewport narrow. Document information scrolls independently of the actions. The window does not read the unavailable saved language setting; Spanish is the default and `--en` selects English. `--software` selects Glow explicitly.

## Executed evidence

Offline binary Clippy with warnings denied and an offline native build passed. Formatting was applied. No new standalone test or broad workspace test run was added for this unit.

Five isolated native scenarios exercised the production opening adapter, restore worker and handover. The scenarios drive controller operations, not physical pointer clicks:

| Case | Observed result |
| --- | --- |
| startup-recovery-review | Malformed preferences open the Spanish review; two books, one annotation and one bookmark are read from the synthetic backup; exiting preserves the original settings bytes |
| startup-recovery-invalid | Invalid JSON produces an error without a confirmable review; original settings bytes remain unchanged |
| startup-recovery-library-installed | A non-SQLite database fails opening; a reviewed backup installs and a separate read-only reopen returns two books; the damaged database survives byte for byte in the safety store |
| startup-recovery-preferences-installed | Unsupported version 99 settings fail before a database is created; installation produces readable settings and two books while preserving the unsupported original settings in the safety store |
| startup-recovery-small-200 | Unsupported settings open a review at requested 640 × 480 with 200% text; compact actions remain fully visible and the original settings remain unchanged |

Every case records a synthetic result only after checking the original file bytes or safety-copy bytes. Installation also independently reopens the installed database and preferences. Native control-bound checks passed. Captures of normal review, invalid backup and enlarged-text review were visually inspected. An initial enlarged-text layout devoted too much height to actions; shortened labels fixed it before the final checkpoint.

The [capture manifest](../screenshots/rust-startup-recovery/capture-manifest.json) identifies source, lockfile, executable and artifact hashes. Published screenshots contain origin metadata. No personal store, PDF or Computer Use was used in these scenarios.

## Remaining acceptance

Imported-store selection failures, restore-journal selection failures, unknown startup-record repair and recovery without a usable JSON backup remain outside this window. Native physical close-during-preparation timing, file-selector interaction, Narrator and monitor/DPI acceptance remain unverified. Synthetic text zoom does not certify physical DPI. Existing restore interruption/rollback checks were not rerun in this UI unit, and this evidence does not replace the full migration, PDF, performance or Windows release gates.
