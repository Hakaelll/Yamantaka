# Compact library and reader controls

The full-width Continue reading card separated the filters from the books. The
reader repeated its title and zoom controls in a prominent toolbar. The owner
requested less visual interruption in both views.

Continue reading now sits beside the library filters as a secondary action. Its
tooltip retains the document title, page and percentage. Short windows keep the
existing utility-row Resume control. The collection no longer reserves a card,
miniature or progress track above the books. Cover geometry, physical motion and
scroll headroom remain unchanged.

The reader uses the existing compact controls at every width: Library, previous
page, page number, next page, Reading options and navigator. Options align right
where space permits. The row has no separate fill, uses 8 by 2 point insets and
leaves 4 points before the document. Zoom, fit and open readings remain available
in Reading options. The unused wide toolbar, fit helper and fit icon were removed.

## Verification

- The locked native debug build completed without warnings; Cargo formatting passed.
- Six isolated native captures were inspected: dark tilted covers, light Spanish
  covers, small covers with 200% text, reader, reading options and small reader
  with 200% text. The cover tops remain clear and the reader controls fit.
- Published images and control records are in
  [the capture directory](../screenshots/rust-quiet-chrome). Their manifest records
  executable, source and image hashes. Only synthetic PDFs were captured.
- No tests were added or run for this layout-only change. The retained suite stays
  at 50 tests, following the owner's request to remove most tests.

These captures do not certify physical monitor DPI, Narrator, pointer and keyboard
interaction, full functional parity or completion of the rewrite. The new build
remains a development preview, not a release.
