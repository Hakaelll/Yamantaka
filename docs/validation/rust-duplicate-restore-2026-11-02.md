# Native consolidation backup installation and restart

The synthetic native flow now continues past consolidation-history review: it validates the recorded backup hash, prepares restoration, closes the application, installs the snapshot through the production restore handover, then launches a separate native process against the restored store.

`tool/verify_rust_duplicate_restore.py` runs the real executable twice. It requires successful exit from both processes, checks the installed and safety databases, compares every backed-up table with the pre-merge JSON, verifies foreign keys and source PDF hashes, then checks the restarted native library's control capture. Its output directory is restricted to workspace build; both store locations must be owned synthetic capture directories. Raw location markers and logs remain local and are not published.

The special apply mode requires demo plus capture, and the synthetic duplicate preview entry point. It drives the existing restore worker and shutdown handover rather than substituting a database copy in the verifier. Restored demo stores now resume from their installed `yamantaka.sqlite3`, while earlier capture stores continue to use their existing preview database.

Synthetic source PDFs for this scenario live outside the replaceable library directory, as normal imported external PDFs do. They are not moved with the library's safety folder. The verifier checks their original paths and hashes after installation. This case does not establish behavior for user PDFs stored inside the application data directory.

Executed results:

- Workspace Clippy across all targets passed with warnings denied; native debug build and formatting passed.
- Native history-based restore completed: three books installed; the two-book consolidated library remained in its safety directory; the retired record returned only in the restored database.
- Every table in the pre-merge backup equalled the installed snapshot before reopening. Foreign-key check passed.
- The backup retained its receipt hash, and both duplicate source PDFs retained their complete SHA-256 hashes at their original paths.
- A separate native application process reopened the installed library and captured three records. Both processes exited successfully and produced empty stderr logs.

[Status report](artifacts/duplicate-restore-native.json) · [Reopened library](../screenshots/rust-duplicate-restore/reopened.png)

The earlier [history checkpoint](rust-duplicate-history-2026-11-02.md) documents review without installation. This extends it with automatic native preparation and a separate process restart. Physical activation of the confirmation button, Narrator/DPI interaction, abrupt termination during this native flow and large-library acceptance remain pending. No final Release acceptance is claimed.
