# Recover a book title from its PDF

Yamantaka reads embedded title and author metadata and examines text from the first six PDF pages. Adjacent title lines are combined when their size, spacing and alignment support a single heading. The analyzer now orders extracted lines by their page coordinates before building these blocks, so PDF drawing-object order does not determine the order of the title lines.

An embedded title that merely repeats the filename, with or without its extension, remains a fallback. A different front-matter title can replace it automatically when supported by an explicit byline under the existing evidence rules. For example, metadata containing `scan-004` in `scan-004.pdf` no longer blocks a supported book title from the cover. Matching ignores case, whitespace and punctuation. A different embedded bibliographic title still requires review rather than automatic replacement.

For an existing book, open its title-and-author editor and request metadata reanalysis. Reanalysis uses the same filename rule and retains source checks and transactional saving. It does not replace your manual title or author correction. Newly imported books use the updated analyzer directly; existing books are not silently rescanned.

Automatic suggestions still need usable PDF text. Image-only covers require a manual title; this feature does not add OCR or online lookups. A large logo without sufficient bibliographic support remains a proposal for review. Conflicting bylines do not authorize automatic title replacement. Original PDFs are never rewritten.

This update uses analyzer version four. Earlier stored evidence retains its recorded version and can be refreshed through reanalysis. Version four is included in [review 10](../release/rust-0.16-review-10.md), whose build, packaging and user-reported functional acceptance are recorded separately. Wider metadata and reanalysis acceptance remains pending.

## Analysis under load after review 10

The development source now pauses briefly and retries when native PDF admission
reports a full queue. It keeps the current page and earlier bibliographic evidence
instead of abandoning extraction because of temporary queue pressure. Reopening
and source verification use the same retry policy; native-slot contention also
yields before another opening attempt.

Cancellation remains available during the pause and is returned as cancellation,
including when it arrives during a native text reply. It is not recorded as
unavailable extraction. A failed source check, text work
limit or memory limit still follows its existing error behavior. Passwords are
not retained for a later reopening attempt. No new runtime or pressure tests were
performed for this scheduling change, and it is not included in the published
review-10 executable.
