# Reference catalogue chooser checkpoint

The book button in the reference header now opens an independent catalogue chooser. Search uses the existing title, author and filename matching, with a short typing delay and cancellation of obsolete requests. Previous and Next retrieve bounded pages of up to 80 books. This search covers the catalogue independently of the main library's current filters or visible page.

Opening the chooser refreshes its results and focuses the search field. The popup remains open while searching or changing pages. Earlier rows and their range remain visible while a request is pending, with selection disabled until the matching result arrives. Missing documents are visible but cannot be opened. Empty results and query errors have local feedback and retry. Closing the reference cancels its pending catalogue request. Query replies are matched by cancellation identity before the main library receives them, preserving its separate page and filters.

The native binary compiled. One focused native scenario passed: `reader-reference-chooser`. Its synthetic catalogue contains 110 books; the chooser requested offset 80 and received 30 rows, then searched for a generated title absent from the main library's 80 visible books. The final reply contained exactly one result. The [native chooser capture](../screenshots/rust-reference-chooser/reader-reference-chooser.png) and control record preserve that evidence. Only local source inspection and this scenario are claimed; no broad suite or new standalone tests were run.

This verifies bounded catalogue pagination and independent search through the native runtime. Physical keyboard/accessibility acceptance, secondary reader tools and saved split restoration remain pending. The rewrite remains pre-release.
