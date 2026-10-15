# Bounded front-matter metadata evidence

The domain now evaluates actual PDFium text snapshots and their character
geometry. It produces typed decisions for unchanged metadata, corroborated
metadata, an explicit author value eligible for autofill, or a proposal requiring
review. It does not call PDFium, SQLite or the UI itself.

This is an analysis component, not connected import behavior. The application
actor, persistence of evidence and review controls remain the next integration
work. The existing import metadata hygiene remains active independently.

## Decisions and evidence

The analyzer accepts only unique, increasing page indexes from zero to five.
Each snapshot is consumed independently, so callers need not retain the text
of all six pages. Retained candidates include their source page, normalized
source bounds and a qualitative evidence kind. No score is presented as a
measured probability of bibliographic correctness.

Sparse pages contribute bounded text lines. Prominent title candidates require
reliable geometry, an upper-page location and a height larger than typical text.
A nearby explicit byline helps select the title rather than a larger publisher
logo. Repetition on different pages is retained as evidence, but visual titles
still require review. Repeated logos, running headings and incomplete subtitles
are reasons not to commit an inferred title automatically.

Author candidates use explicit `By`, `Por`, `Written by`, `Author:`, `Autor:` or
`Autora:` lines. A bare `By` or `Por` line needs the supporting title hierarchy.
The analyzer does not guess a person from capitalized words or impose Western
name shapes. Different explicit author values require review rather than
silently selecting the first one.

An accepted embedded value that matches a front-matter line is corroborated.
A different plausible value is preserved beside its proposal for review.
Text comparison preserves word boundaries, so `Now here` and `Nowhere` do not
become the same value. Missing geometry cannot create visual candidates.
Dense body pages and image-only pages do not generate bibliographic guesses.

## Rotation and source coordinates

A real PDFium regression initially failed at 180 degrees of intrinsic rotation:
the text was present, but upper-page filtering used the rotated display space.
Analysis now transforms character regions through the PDFium-supplied coordinate
basis into an unrotated crop basis. Title height, top position and byline gaps
use that semantic layout. Candidate bounds retain the original displayed-page
coordinates for future source highlighting.

The native test now produces the same title and author decisions at 90, 180
and 270 degrees. This change does not alter the reader's page coordinates,
rendering or annotation geometry.

## Work and memory limits

The shared memory governor reserves 16 KiB for retained metadata, candidates
and the result lifetime, plus 192 KiB temporarily while analyzing one page.
The existing native text snapshot has its own governed allocation. Cancellation
is checked before admission, every 128 characters, before committing an
observation and before returning evidence. A rejected allocation leaves the
observation unconsumed.

Line buffers are limited to 1,024 UTF-8 bytes and retained lines to 128, with
candidate lines limited to 180 characters. Analysis stops after more than 24
nonempty lines instead of scanning the rest of a dense body page. Native text
extraction itself retains its existing limits and is not made progressive by
this component.

## Verification

Four domain tests check missing geometry, page order and limits, cancellation,
memory denial and release, and word-boundary comparisons. Seven native PDFium
tests check:

- Repeated title proposals and explicit author evidence when metadata is absent
  or rejected.
- Conflicting plausible metadata preserved for review.
- Correct metadata favored over a larger publisher logo.
- A repeated large logo with missing metadata not replacing the nearby title.
- Single visual titles and different explicit authors requiring review.
- Dense body pages and image-only PDFs producing no guesses.
- Equivalent decisions under all three nonzero quarter-turn intrinsic rotations.

Native tests compare source PDF bytes before and after. After the worker and
snapshots drop, only the result's 16 KiB reservation remains; dropping the result
returns governed usage to zero. The rotation failure was observed before the
coordinate correction in `build/verification/front-matter-rotation.log`.

All 475 workspace tests passed, including 101 UI tests. Formatting, workspace
Clippy with all targets and warnings denied, and the native application build
passed. Repeating fixture generation produced fourteen identical hashes.

```powershell
python tool/create_rust_metadata_fixtures.py
cargo test -p yamantaka-domain front_matter --locked
cargo test -p yamantaka-pdf --test front_matter_pdfium --locked
cargo test --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The deterministic generator now creates fourteen synthetic metadata and
front-matter PDFs, including rotated variants. It does not regenerate the
existing reader or cover fixtures. Generated PDFs remain outside Git.

## Remaining acceptance

The actor must request snapshots at background priority, yield to visible pages,
validate source ownership before saving and handle cancellation or unavailable
geometry without losing an imported document. Stored decisions need manual-choice
precedence, candidate evidence, explicit review and reanalysis controls. None of
those integrations is claimed by these tests.

Multi-line title composition, multi-column front matter, multilingual byline
patterns beyond the listed forms, cropped and unusual text orientations, large
corpus accuracy and latency acceptance remain pending. The analyzer has no OCR
or network lookup. Its synthetic cases do not establish accuracy on arbitrary
personal libraries. No UI changed, so this checkpoint adds no screenshots or
frame-time claim. The full rewrite and final release remain unfinished.
