# Find passages in a PDF

Open the reader's **Find** panel, enter a phrase, choose case or whole-word
matching if needed, and run the search. Select a result to navigate to its page.
**F3** and **Shift+F3** move between results through the canonical reader
navigation, including continuation beyond the current result batch. The reference
pane in comparison reading has its own query and selected result.

After the review 11 candidate, the source also requests highlights for visible
pages independently of the result-list batch. This keeps visible occurrences
highlighted when you scroll away from the pages in that batch. The selected
occurrence retains its stronger fill and outline without duplicate painting.

Page highlighting follows the existing 80-match continuations with at most two
outstanding requests per pane. Leaving a visible page cancels its requests and
releases its retained match payloads. Closing Find, changing its options or
starting a fresh query also discards these payloads. Moving between result batches
for the same query can retain the current visible-page highlights.

PDF rendering has higher worker priority than page search. Highlight requests use
the existing text memory governor, source guards and per-page work limits. A full
queue retries after 100 ms. Memory, source or native failures retain completed
highlights and show a short notice in Find; run the search again to retry. Very
dense pages can reach the existing 10,000-match or 128,000-character work limits,
so highlighting is not promised beyond those limits. Scanned pages without
extractable text have no invented matches.

This change is absent from the existing review 10 and 11 packages. Development
compilation completed successfully. No new application tests, captures or
performance measurements were run at the owner's request. Broad search and
interaction acceptance remain pending.
