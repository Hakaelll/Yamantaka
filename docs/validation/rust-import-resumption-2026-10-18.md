# Resuming metadata analysis after reader preemption

Date: 2026-10-18. Scope: native-owner loss during import and metadata reanalysis.
This checkpoint does not establish whole-rewrite acceptance or a final release.

## Behavior

Opening a visible reader can reclaim a background PDF handle to preserve the
two-document native limit. Previously, subsequent source or geometry requests
reported a generic PDF error. Import analysis could then be stored as unavailable,
losing useful evidence collected before the interruption. A cancelled background
open could also remain asleep behind two protected readers until another native
event occurred.

The native owner now returns `NativeDocumentUnavailable` for source snapshots and
page geometry when that handle is absent. This identifies a recoverable owner
loss without matching error-message strings. User cancellation and actual PDF
failures retain separate outcomes.

`ImportEnrichment` retains its governed front-matter state and interrupted page.
It reopens the same temporary document through the existing background queue,
which waits while both reader handles are protected. It compares the reopened
page geometry and verifies the captured source revision before requesting the
interrupted page again. Previously observed pages are not rescanned.

The source revision binds canonical path, size, modification time, and the SHA-256
of the first 64 KiB. It is a bounded identity check, not a full-file digest. A
changed source or page geometry aborts the document without publishing mixed
metadata. Existing storage source checks and atomic metadata writes still apply.
Reanalysis failures preserve the previous review and manual corrections.

If the native owner disappears before the initial source snapshot is accepted,
the actor repeats opening and source capture together. This supplies fresh page
metadata rather than associating old page information with a replacement file.
The guarded reanalysis write still checks its original catalogue decision.

A coalesced cancellation wake now reaches the native scheduler as well as the
actor. Cancelling import or reanalysis can finish a background open waiting behind
two readers without releasing either reader. Cancellation and failure still wait
for acknowledged native cleanup before publishing the terminal result. No new
polling timer, processing banner, or UI layout was added.

## Regression evidence

Before the changes, two native tests failed: reader preemption did not return the
recoverable owner-loss result, and cancellation did not wake a queued background
open. A separate actor regression failed because interrupted analysis never
reopened its native document. Local logs retain those failures:

- `build/verification/import-resumption-native-before.log`
- `build/verification/import-resumption-actor-before.log`

Eight new actor scenarios use real PDFium and public reader-opening commands:

- Resume import at the interrupted second page, preserving first-page evidence.
- Cancel a waiting import without unpinning either reader.
- Reject a source changed while import analysis waits, leaving it uncatalogued.
- Survive two preemptions while repeating only the interrupted page.
- Resume reanalysis while preserving manual title and author corrections.
- Preserve the complete previous review when waiting reanalysis is cancelled.
- Preserve that review when the source changes during its wait.
- Lose the initial source owner, replace a two-page synthetic PDF with a seven-page
  one, and verify fresh page metadata and all six bounded analysis pages.

The scenarios check SQL responsiveness while reopening is blocked, retained
reader text, exact page-request sequences, stored title/author evidence, scoped
cancellation, and governed memory returning to zero after owners close. Source
replacement modifies only temporary copies. All original acceptance fixtures
remain byte-for-byte unchanged.

All 528 workspace tests passed, including 105 UI tests, with no ignored tests.
Formatting, all-target Clippy with warnings denied, and the native application
build passed. This checkpoint adds eight real actor scenarios and two native
ownership/cancellation regressions.

Thirteen fresh native capture scenarios passed their independent verifiers:
five metadata reanalysis states, six reader layouts/panels, and two cover-capacity
recovery cases. They cover both themes, English/Spanish, reduced motion, and
640×480 with 200% text. The checks inspect synthetic SQLite records, source hashes,
control bounds, retained preview/refinement traces, stable viewports, and native
PDF cover textures. They do not establish a new performance percentile.

Local capture sets are `build/rust-visual/import-resumption-reanalysis-final`,
`build/rust-visual/import-resumption-reader-final`, and
`build/rust-visual/import-resumption-covers-final`. Their manifests bind captures
to the executable, lockfile, and all 73 UI source files. Raster and synthetic
store artifacts remain in ignored build directories. This checkpoint preserves
the existing UI composition and publishes no duplicate screenshot collection.

The frozen development executable is
`build/verification/yamantaka-import-resumption-final.exe`, SHA-256
`7EC3A164602EEA99547FF1C8CBF2D6D65D90D39444B99761238CD7CDD9BD823A`.
It is not the final Release bundle.

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo build -p yamantaka --locked --offline
cargo test -p yamantaka-application --test import_resumption_runtime --locked --offline
cargo test -p yamantaka-pdf --test background_lease_pdfium --locked --offline
python tool/verify_rust_metadata_reanalysis.py build/rust-visual/import-resumption-reanalysis-final --source-hash-file build/verification/import-resumption-source-hashes.json
python tool/verify_rust_reader_fluidity.py build/rust-visual/import-resumption-reader-final build/verification/yamantaka-import-resumption-final.exe
python tool/verify_rust_cover_capacity.py build/rust-visual/import-resumption-covers-final
```

## Remaining work

This proves recovery for the tested owner-loss transitions, not universal smooth
rendering or a measured latency improvement. One PDFium call still runs until it
returns. Durable import resumption after process restart, folder import and
watcher acceptance, live file revalidation, complete metadata presentation and
localization, full-file duplicate identity, physical DPI/Narrator/multi-monitor
validation, paired Flutter/Rust performance runs, remaining parity, and the final
Windows Release bundle remain pending.
