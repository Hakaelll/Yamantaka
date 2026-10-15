# Automatic cover decision freshness

Library cover requests now check a saved automatic decision before reusing it.
The selection must use the current detector version, refer to an existing page
and match the document's catalogued source modification date. Stale decisions
request fresh native detection. Manual selections bypass detection regardless of
their old algorithm version or source date.

This follows the connected Flutter `ThumbnailService` behavior, whose validity
check uses `LibraryDocument.fileModifiedAt`. It does not claim a new filesystem
watcher or live invalidation of an already-resident GPU cover.

## Snapshot and replacement

Storage reads the catalogue source date, page count and all eleven cover fields
in one joined SQLite statement. The returned snapshot contains a typed render
or detection plan, the exact cover decision revision and the source date.
Equivalent RFC 3339 timestamps with different offsets compare as the same
instant. Identical legacy date strings retain their existing meaning.

No cover metadata is changed during this read. A manual index beyond a shortened
document's last page is clamped for rendering, matching Flutter's manual-page
rendering behavior; the stored decision remains intact. Unknown selection modes
fail without replacing data.

Background detection keeps the saved row until the replacement transaction
succeeds. The transaction checks document path, fingerprint, page count and the
captured source date. It then compares the exact saved decision revision.
A later manual or automatic choice wins, and the actor renders that choice's
page. Cancellation, a changed source date and failed writes preserve the prior
decision and its cache references. Explicit Restore automatic now carries the
same source-date guard.

Saved-page requests also reject a path that no longer matches the catalogue or
a document marked missing. This prevents an old UI request from reading another
path under the saved document ID.

## Executed verification

- All 446 workspace tests passed, including 95 UI tests.
- Formatting and workspace Clippy with all targets and warnings denied passed.
- A storage matrix covers absent, fresh, outdated-version, outdated-source,
  equivalent-offset, out-of-range and manual choices without read-time mutation.
- Three new actor tests use real PDFium: refresh three kinds of obsolete choice;
  bypass detection for current automatic and old manual choices; preserve the
  old row during delayed detection and handle later manual/automatic choices,
  source changes, cancellation and failed writes.
- Existing storage write-failure coverage now includes a source-date change
  during calculation. The saved manual-page UI test uses a matching source path
  and separately proves that a mismatched request is rejected.

Three isolated native capture cases seed obsolete version 2, an obsolete source
date, or manual version 4. The first two refresh page zero to the detector's
actual choice of page one. The manual case retains page zero and all seeded
decision metadata. Both graphite English and ivory Spanish retain the compact
reading strip introduced in the preceding visual checkpoint.

The [published captures](../screenshots/rust-cover-freshness/) include source
hashes, executable and lockfile hashes, synthetic provenance and control bounds.
The verifier reads actual SQLite output and validates cover pages, metadata,
unchanged reading progress, pixel-cache records and published PNG pixels.
It also checks the selected fixture's purple artwork in the rendered book,
independently of the stored page and readiness counters.
That check accepts the actual selected cover and rejects the real manual
page-zero contents capture, proving that a correct page number alone cannot
satisfy it.

Final development executable SHA-256:
`C0AFB61596656203A1A7FBE07B18799779C7A04200D67ED3DA76EB82CE4C79FB`.

## Reproduce

```powershell
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p yamantaka --bin yamantaka-rust --locked
./tool/capture_rust_ui.ps1 `
  -Executable target/debug/yamantaka-rust.exe `
  -OutputDirectory build/rust-visual/cover-freshness `
  -CaseNames @('cover-refresh-version','cover-refresh-source','cover-refresh-manual')
python tool/verify_rust_automatic_cover.py `
  build/rust-visual/cover-freshness target/debug/yamantaka-rust.exe
```

## Remaining acceptance

Filesystem revalidation, watcher behavior and invalidation of already-resident
covers still need connected acceptance. The native worker can reuse a warm PDF
handle whose file has changed; safe source-revision ownership remains separate
work. Small/large thumbnail parity, cleanup/recovery controls, physical DPI,
Narrator and complete cover interaction/performance acceptance also remain open.
This checkpoint is not a replacement release or completion of the rewrite.
