# Background import analysis

The native import actor now runs the existing front-matter analyzer against real
PDFium text snapshots. Analysis and persistence are connected to the application's
normal import command. This checkpoint does not complete metadata review or the
full rewrite.

## Import behavior

Each file opens under a separate temporary native identity. The actor obtains
the source stamp captured by that handle, then requests one text page at a time.
Analysis reads at most six opening pages. It uses the shared memory governor and
the page's native coordinate basis.

An unambiguous explicit byline can supply a missing or rejected author. Visual
titles remain proposals. Conflicting embedded values also remain proposals,
preserving the current value. Raw embedded title and author values remain in
the catalogue's source fields.

The actor closes the temporary handle before saving. One SQLite transaction
saves the document, the versioned enrichment row, and the evidence record.
Evidence includes each decision's kind, candidate, source page, and normalized
bounds. Confidence columns remain unset because these decisions are rules,
not calibrated probabilities.

Writes have readback checks before commit. Ignored inserts, changed document
fields, changed enrichment rows, and altered evidence roll back the transaction.
Existing normalized paths return their existing identity without replacing
manual metadata, reading progress, or cover decisions.

## Source and reader protection

Storage compares the canonical path, size, modification time, and SHA-256 of the
first 64 KiB with the frozen native source stamp. It repeats verification before
commit. The native test changes the header while retaining file size and time,
then confirms that import refuses the stale metadata.

This is a bounded source check. It is not full-file equality or exact-duplicate
confirmation. Same-size edits outside the sampled header that preserve time
remain outside this check's guarantees.

Reader opening, visible rendering, and reader text have priority over background
inspection and analysis. Background open continuations retain their priority.
Analysis limits a text page to 16,000 characters before allocating and walking
its character geometry. The reader retains its existing 128,000-character limit.
PDFium's native text-page construction remains an opaque call, so this limit
does not establish a worst-case native-call duration.

Background requests cannot replace a protected reader's native identity.
Import and indexing allocate UUID identities absent from the catalogue, with
different producer prefixes. A native regression uses the legal legacy document
ID `index-1-a`, indexes both PDFs, and reads the original reader afterward.
Restoring the old deterministic index identity makes that regression fail.

## Progressive catalogue and cancellation

Every successful document commit increments a coalesced revision and wakes the
UI. The catalogue reuses its current query and filters, with the existing
80-row limit. Revision capture happens before query admission so a concurrent
commit cannot be acknowledged without a corresponding query. Queue pressure
schedules a retry instead of adding a loading screen.

A held-second-open test imports the first file, queries it, changes its favorite
state, then cancels the remaining batch. The first document and its favorite
remain saved. Cancellation produces a terminal reply scoped to that import.

When analysis exceeds its work or memory budget, basic valid metadata can still
import with a static, redacted analysis failure code. Missing text does not
produce guesses. A failed source check or failed transaction does not advance
the catalogue revision.

## Verification

The native actor tests cover proposals, explicit author autofill, corroboration,
conflicts, atomic rollback, source changes, bounded fallback, and legacy identity
collisions. Additional tests cover progressive import cancellation, text limits,
frozen native identity, protected-reader admission, and scheduler priority.
Source fixtures remain byte-for-byte unchanged, and governed memory returns to
zero after owners drop.

All 486 workspace tests passed, including 101 UI tests, with no ignored tests.
Formatting, Clippy for all workspace targets with warnings denied, and the
native application build passed. Repeated fixture generation reproduced all
15 manifest hashes. The legacy index-identity regression failed with the old
allocation and passed with the corrected allocation.

Run the deterministic synthetic fixtures and the Rust checks with:

```powershell
python tool/create_rust_metadata_fixtures.py
cargo test --workspace --locked --offline
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo build -p yamantaka --locked --offline
```

## Remaining acceptance

Review and manual acceptance controls, reanalysis, multi-line bibliographic
analysis, and OCR remain pending. Covers continue through the existing native
gallery pipeline. This checkpoint does not add an import-time cover completion
barrier or establish accuracy across a personal PDF corpus.

No layout changes or new screenshots are claimed. Paired Flutter and Rust
Release traces, physical interaction latency, complete memory accounting, and
the wider fluidity acceptance gates remain pending. Passing these functional
checks does not establish universal frame-time or loading-time guarantees.
