# Select passages across pages

Enable text selection in the reader, then drag across a passage. In continuous
layout, keep the pointer near the upper or lower edge of the document viewport
to extend the selection onto adjacent pages. Horizontal edge scrolling is also
available when zoom makes the page wider than the viewport. Page gaps use the
nearest visible page without resetting the start of the selection.

After review 12, double-clicking a word's glyph selects its Unicode word segment.
Punctuation and whitespace use Unicode segmentation; this is not a dictionary or
language-specific word recognizer. Pointer carets also use the keyboard's
grapheme boundaries so selection endpoints do not split combining accents or
joined emoji sequences. The behavior is shared by both panes and by edge-scroll
selection. Double-clicking a blank margin does not select a distant word.

The same behavior is connected to the main reader and the reference pane in
comparison reading. Each pane keeps its own selection. Leaving its viewport by
more than 32 logical points stops automatic scrolling. Releasing the mouse
finishes the selection, including when the pointer is outside a page.

Scrolling follows the drag directly, with no easing or inertia. Its speed
increases near the viewport edge and stops at the content boundary. The existing
text worker and memory limits remain in use; selection waits for a visible
page's text instead of skipping that page. Scanned pages without extractable text
do not gain invented text or highlight geometry. Use a page note when a passage
has no reliable geometry.

Use **Copy passage** or **Annotate passage** after selecting. With the page
focused, **Shift+arrows** extends the selection, **Ctrl+A** selects that page and
**Ctrl+C** copies the passage. Keyboard commands remain available independently
of pointer scrolling.

Edge scrolling is included in review 12. Word selection and pointer grapheme
snapping are later source changes and are not included in that package. The
locked, offline development build completed successfully. No new interaction
tests or native captures were run at the owner's request; pointer, rotation,
comparison and physical input acceptance remain pending.
