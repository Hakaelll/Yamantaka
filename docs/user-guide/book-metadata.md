# Recover a book title from its PDF

Yamantaka reads embedded title and author metadata and examines text from the first six PDF pages. Adjacent title lines are combined when their size, spacing and alignment support a single heading. The analyzer now orders extracted lines by their page coordinates before building these blocks, so PDF drawing-object order does not determine the order of the title lines.

An embedded title that merely repeats the filename, with or without its extension, remains a fallback. A different front-matter title can replace it automatically when supported by an explicit byline under the existing evidence rules. For example, metadata containing `scan-004` in `scan-004.pdf` no longer blocks a supported book title from the cover. Matching ignores case, whitespace and punctuation. A different embedded bibliographic title still requires review rather than automatic replacement.

For an existing book, open its title-and-author editor and request metadata reanalysis. Reanalysis uses the same filename rule and retains source checks and transactional saving. It does not replace your manual title or author correction. Newly imported books use the updated analyzer directly; existing books are not silently rescanned.

Automatic suggestions still need usable PDF text. Image-only covers require a manual title; this feature does not add OCR or online lookups. A large logo without sufficient bibliographic support remains a proposal for review. Conflicting bylines do not authorize automatic title replacement. Original PDFs are never rewritten.

This update uses analyzer version four. Earlier stored evidence retains its recorded version and can be refreshed through reanalysis. No tests, compilation or new PDF runs were performed for this update at the user's request; detection and reanalysis acceptance remain pending.
