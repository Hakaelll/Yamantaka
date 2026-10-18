# Cover titles, a quieter reader, and a smaller test suite

The reader no longer has a permanent browser-style tab row. **Reading options →
Open readings** holds the document selector and its close, reopen, duplicate,
pin, and ordering actions. The existing reader toolbar remains the only toolbar.
The document model still supports twelve readings and two hot native documents.

The library reserves 24 logical points inside the catalogue scroll area before
the first cover row. That space belongs to the cover's hover projection, so an
elevated cover no longer immediately crosses the scroll clip. The physical book
geometry, native PDF cover imagery, springs, and reduced-motion behavior remain
unchanged. The separately committed reader change removes centered loading page
numbers while retaining navigation and accessibility numbering.

## Detect titles from a cover instead of a filename

Analyzer v3 groups nearby, aligned typographic lines into bounded title blocks.
It recognizes explicit compiler bylines, including English and Spanish labels.
Sparse cover blocks can qualify even when their letters have a similar height
to the author line. Early title/byline evidence ranks ahead of a later running
header, and conflicting author evidence prevents automatic title acceptance.

Supported title/byline candidates fill missing or rejected metadata. Plausible
accepted titles remain protected and conflicting values become proposals.
Reanalysis distinguishes a filename generated for rejected source metadata from
an accepted book title, while manual title and author corrections retain priority.
Older evidence needs reanalysis before it receives current source-page links.

A user-provided PDF reproduced both defects: the detector treated wrapped cover
lines independently, and a technical Word title led to a filename fallback. A
read-only PDFium diagnosis now returns the full cover title from page zero.
The source PDF and private diagnostic output are excluded from this commit.
Its SHA-256 matches before and after the local diagnosis. The diagnostic opens
and reads the file without an editing operation.

Public regression fixtures use invented titles and authors. A failing native
case reproduced the three-line title on the first or second page. It now checks
the complete title, correct page, retained author, released tracked memory, and
unchanged source bytes. A storage check also upgrades an old generated filename
while preserving manual corrections.

## Retain essential tests

The owner explicitly requested removal of 90–95% of the Rust tests. The previous
committed suite contained 536 tests. The current suite contains 50, about 90.7%
fewer, including the new wrapped-title and filename-reanalysis coverage.

Retained checks cover historical schemas, populated data, backup compatibility
and rollback, real PDFium rendering and passwords, source immutability, native
cancellation and preemption, metadata source checks, GPU upload ownership, and
Windows instance activation. Most detailed UI, annotation, search, organization,
navigation, and unit-level regressions were removed. This deliberately reduces
automatic coverage. Historical Flutter baseline tests remain with the reference
implementation until its separate removal gate.

Removed tests are recoverable from commit `7ad7bf2`. Historical validation reports
describe their original checkpoints; they do not imply that those suites still
exist or were rerun. Future verification uses the retained checks relevant to a
change rather than recreating the removed suite.

## Validation scope

The 50 retained tests pass. Formatting, warning-free all-target Clippy, and native
compilation pass. Seven native synthetic captures cover the single-toolbar reader,
multiple open readings with the strip hidden, the options dropdown, narrow text
at 200%, and library covers in light and dark modes, including a tilted cover.
Capture manifests bind images and control records to the frozen executable and
UI sources. The private PDF is not part of these screenshots.

The frozen executable has SHA-256
`232DF66A53C6DCC250C89D97DADA0F46EDD3356F88A8171CB9AF346D6ECF6573`.
The bounded visual review returned `ship` for the visible reader chrome and
catalogue headroom. The expanded readings submenu was not captured, so its
runtime interaction remains unverified. The documenter confirmed this as an
ordinary extension and preserved the existing design files.

These checks do not establish physical DPI, Narrator, multi-monitor behavior,
paired Flutter/Rust release performance, complete migration UI, or whole-product
parity. The rewrite goal remains active and no final release is declared.

An account usage limit temporarily prevented automatic permission review. The
pending operation was retried successfully after review became available; the
original PDF hash remained unchanged. No alternative execution route was used.
