# Metadata review with PDF page evidence

The title and author editor now places library values beside a real PDF page.
Each current proposal can show its source page, quoted text, and extraction reason.
**Compare page** selects that reference. **Open page N in reader** opens the same
page through canonical reader navigation, with **Back to metadata** to return.
The library's 3D book geometry, cover imagery, and physics are unchanged.

The editor uses the established graphite and ivory themes, an editorial heading,
padded fields, grouped actions, and an asymmetric desktop composition. Narrow
windows stack the editing area and reference. Unsaved changes take precedence
over earlier analysis and save feedback. Source opening and reanalysis require
a clean draft. Original PDF bytes remain unchanged.

## Bind evidence to its source

Import and reanalysis now save a source stamp with the auxiliary front-matter
evidence. It contains the legacy fingerprint, byte size, modification timestamp,
and page count. The principal SQLite schema remains v11.

Storage exposes a typed title or author citation only when the analyzer version,
source stamp, displayed value, evidence kind, and page bounds agree. Old evidence
without the stamp remains readable but offers no citation until reanalysis.
Manual corrections that differ from the extracted value have no misleading link.

A source request reads the displayed revision and document in one SQLite snapshot.
It checks the physical file's canonical path, size, timestamp, and legacy partial
fingerprint before returning a page. Metadata preview admission and completion
use that same check. Changed metadata or files reject the request without writing
metadata, clearing drafts, or overwriting newer decisions.

These are bounded source checks, not full-file equality. Same-size changes outside
the first 64 KiB that preserve modification time remain outside this guarantee.
Full SHA-256 duplicate acceptance remains a separate rewrite requirement.

## Ownership and navigation

The application actor handles source checks independently of pending native PDF
opens. Replies carry their own cancellation identity. Reload and editor closure
cancel obsolete requests. Preview jobs reuse the existing governed PDF worker,
retained capacity admission, native textures, and late-result validation.

Opening a citation reopens the native source through the reader's existing open
transaction. For the same document, canonical navigation history and positions
A/B transfer once when page counts agree. A subsequent unrelated open revokes
that transfer. The original editor remains available for the explicit return.

## Reproduce the checks

Verified on Windows with Rust 1.97.1: 536 workspace tests pass with none ignored,
`cargo fmt --all --check` passes, Clippy passes with `-D warnings`, and the native
debug build succeeds. The frozen capture executable has SHA-256
`393CB101F76CC3BE36E00361854E2BF30E3C9A4D7729215879C883F7077847A2`.
These are correctness checks and synthetic captures, not release performance data.

Generate synthetic fixtures with `python tool/create_rust_metadata_fixtures.py`.
Run the standard workspace format check, warning-free Clippy, and workspace tests.
Relevant regressions cover later-page evidence, malformed or legacy evidence,
changed catalogue rows and physical files, manual corrections, cancelled source
requests, editor closure, history and mark transfer, and actor responsiveness.

The native matrix uses a frozen executable and isolated synthetic SQLite stores:

- Eight `metadata-source-*` cases cover dark English, light Spanish with reduced
  motion, narrow text at 200%, a scrolled narrow reference, opening, return,
  an unsaved draft, and stale metadata rejection.
- Seven `metadata-review-*` cases retain save, dismissal, reset, and failure checks.
- Five `metadata-reanalysis-*` cases retain manual-value and work-limit checks.

`tool/verify_rust_metadata_source.py` checks capture hashes, executable and UI
source identity, horizontal control bounds, actual rendered reader page, disabled
draft and stale links, independent SQLite decisions, opt-in search, source stamps,
and original fixture hashes. Existing review and reanalysis verifiers check their
corresponding matrices.

The first capture round reproduced a horizontal overflow at 200% text. The width
cap now respects available space before selecting the stacked layout. A second
round verifies the corrected composition and unsaved-state message. A reported
light-theme text-loss finding was withdrawn after pixel comparison established
identical PDF paper content in both theme captures.

Physical DPI, Narrator, multi-monitor behavior, runtime keyboard acceptance, and
paired Flutter/Rust performance measurements remain pending. This checkpoint is
not final rewrite acceptance or a release candidate.
