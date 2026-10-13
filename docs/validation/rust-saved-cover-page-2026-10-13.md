# Preserve the selected PDF cover page

The native library now reads `document_covers.source_page_index` before asking
PDFium for a cover. A saved source page takes precedence over the default first
page, including manual choices copied from Flutter. Documents without a saved
selection continue to use page one. Reads do not rewrite cover metadata.

The native ticket, render key and installed texture retain the actual page
index. The cover actor continues to use cancellation identity and generation
ownership; the UI no longer rejects a valid image solely because it comes from
a later page. Page geometry and aspect are read from the chosen PDF page before
the bounded bitmap is allocated. An invalid saved index fails extraction and
keeps fallback presentation; it does not silently replace the saved decision.

## Verified behavior

- 410 workspace tests passed. A new SQLite test reads a manual selection while
  preserving cache paths, selection mode and algorithm version. A new actor
  test renders the saved second page without opening a reader and checks the
  returned page index, generation, dimensions, cancellation scope and unchanged
  PDF bytes.
- The existing native PDFium cover regression now also checks the second page's
  landscape dimensions and rejects an out-of-range page.
- The WAL migration regression now seeds a manual source-page choice. The
  independent Rust copy preserves that choice, mode, algorithm version and
  aspect ratio while clearing recreatable thumbnail paths. The Flutter source
  retains its decision. All six compatibility tests passed again after adding
  these assertions.
- Workspace Clippy with warnings denied and `cargo fmt --all --check` passed.
- Four native synthetic captures cover a saved second-page selection in dark
  and Spanish light/reduced-motion modes, the unchanged default, and compact
  2× scale. Each records installed WGPU covers, no pending cover replies and
  the actual source page per resident document.
- Independent verification matches all 64 UI source hashes, executable,
  Cargo.lock and image/control hashes. PNG publication adds origin metadata
  without changing pixel chunks. No personal library was opened.
- Executable SHA-256:
  `C10B456BF170638F3C84528E3B844C746A0D82C0C6D8D3E3785FD0B90A2F960D`.

See [the capture manifest](../screenshots/rust-saved-cover-page/capture-manifest.json)
and [the second-page cover](../screenshots/rust-saved-cover-page/pdf-saved-cover-page.png).
The white first book in that capture is deliberate: the saved second PDF page
contains a study heading rather than the colored first-page artwork. The other
books retain their first-page covers.

## Remaining work

This connects existing cover decisions to rendering. The interactive native
page picker, saving new choices, automatic candidate scoring, persistent
thumbnail cache and retry controls remain pending. It does not establish whole
Flutter cover parity, physical accessibility, large-library performance or
final-release acceptance.
