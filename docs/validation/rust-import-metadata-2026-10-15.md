# Import metadata hygiene and consistent search titles

New imports now distinguish missing, accepted and rejected metadata before
storing effective catalogue fields. The original PDF title and author remain
unchanged in `source_title` and `source_author`.

## Import behavior

The domain policy collapses Unicode whitespace, preserves names and punctuation,
and rejects empty strings, non-whitespace control characters, URL-only values
and values exceeding 512 characters. Title checks reject specific placeholder
labels and tool-generated values such as `Microsoft Word - Thesis.docx`.
A long numeric or hexadecimal title identical to the source filename stem is
treated as an identifier rather than a detected title.

Author checks reject explicit placeholders and known tool names. They do not
require a Western personal-name pattern: organizations, Arabic and Chinese
names, one-character names and `Anonymous` remain valid. Titles such as `1984`,
`道`, `Editorial Theory` and `Microsoft Word for Beginners` are retained.
The policy avoids rejecting a title merely because it contains a word such as
`contents`, `edition`, `editorial` or `ISBN`.

A rejected title uses the original filename as its effective title. A rejected
author has an explicitly empty effective database value, preventing fallback to
its rejected raw source. Catalogue decoding exposes that value as an absent
author. This is a persistence adapter detail; it does not invent another author.

Reimporting an existing normalized path still returns its original ID before
changing metadata. Manual corrections and reading progress remain intact.
Existing imported rows are not rewritten on database open.

## Search consistency

Global-search metadata, page titles, bookmarks, annotations and related-document
titles now use the same precedence as the catalogue: manual override, current
value, original source, then the filename for a missing title. Previously several
FTS paths preferred the original source over the current value.

The metadata-index comparison uses the same ordering, so changing the effective
title or author makes a completed index eligible for refresh. Consent and
exclusion checks are unchanged. This does not silently enable indexing.

## Verification

Two native import regressions failed before the correction. PDFium returned
the synthetic tool title unchanged, and the imported Unicode metadata retained
unwanted whitespace. After the correction:

- Real PDFium imports filter the tool title and unknown author, preserve their
  raw source values and return the filename fallback through the catalogue.
- FTS returns that effective title and does not index the rejected tool title.
- Reimport preserves the ID, manual title/author and reading progress. The PDF
  bytes remain identical.
- Native metadata preserves Unicode organization names, `1984` and `Anonymous`.
- Domain tests cover placeholders, tool labels, identifiers, controls, missing
  values, the work bound, whitespace and legitimate short or multilingual values.
- A SQLite regression checks current versus raw metadata, completed-index
  invalidation and manual overrides. Existing annotation and bookmark mutation
  tests now also assert effective titles when raw sources differ.

All 464 workspace tests passed, including 101 UI tests. Formatting, workspace
Clippy with all targets and warnings denied, and the native application build
passed. The annotation and bookmark suites also passed after strengthening
their effective-title assertions. Regenerating the three metadata fixtures
produced the same hashes.

The fixture generator is deterministic and writes only three synthetic PDFs
and their hash manifest below `build/rust-fixtures`. It does not regenerate the
existing reading or cover fixtures.

```powershell
python tool/create_rust_metadata_fixtures.py
cargo test -p yamantaka-domain import_metadata --locked
cargo test -p yamantaka-storage --test import_metadata_native --test search_index --test annotations --test bookmark_search --locked
cargo test --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## Remaining acceptance

Hygiene is not bibliographic verification. A plausible but incorrect embedded
title or author can still pass. Comparison with front-matter text and geometry,
candidate evidence, review/reanalysis controls and robust import-time cover
coordination remain pending. No title or author is inferred from the PDF text
by this checkpoint. Image-only files cannot provide text without OCR, which is
not an implemented rewrite capability.

Existing automatic cover detection and manual-choice precedence are unchanged.
This checkpoint changes storage and domain behavior, adds no UI controls and
does not supply new visual captures or performance claims. The full rewrite
and final Windows replacement release remain unfinished.
