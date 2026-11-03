# Browse and restore search exclusions

Global search previously allowed excluding a selected result but provided no complete exclusion list. Once the result disappeared, users could not find that decision through search itself.

The index menu now opens a dedicated exclusion view, also available while search is disabled. The view preserves the current search, filters titles and authors, pages through 80 entries at a time, and restores individual documents to pending indexing. Inclusion uses the existing index coordinator and does not enable or start indexing.

Typed domain records separate the exclusion list from ranked FTS results. Count and page reads share a SQLite transaction, preserve metadata overrides and missing-file entries, and clamp an emptied last page. Work runs through the actor with independent cancellation tokens. The UI retains its previous list during a read, disables conflicting actions during writes, rejects obsolete replies and offers Refresh after an error.

Executed evidence:

- Three file-backed native scenarios each start with 165 exclusions, visit all three pages, include a book from the final page, verify the new count of 164, filter by title and author, and check an empty result before returning to the first page.
- Scenarios cover disabled search in dark English, enabled search in light Spanish at 640 by 480 logical window dimensions, and text at 200 percent. Native captures were inspected. The compact view was revised after the initial capture placed the list below the general search controls; equivalent compact captures show the list now starts immediately below its own heading and filter.
- Read-only SQLite reopening after each process confirms all 165 documents and authors remain, 164 exclusions persist, the included document is pending with zero indexed pages, and none of these documents has FTS rows. This proves inclusion did not silently index them.
- The offline native debug build, workspace formatting and workspace Clippy across all targets with warnings denied pass. All 39 generated PDF hashes remain unchanged. No dependencies or standalone tests were added.

The [status artifact](artifacts/search-exclusions.json) records selected results and the native executable hash. Captures remain local under `build/verification/search-exclusions`.

Physical keyboard/Narrator acceptance, concurrent external edits, fault injection during inclusion and large-catalogue filter latency remain unverified. This checkpoint does not establish final rewrite readiness.
