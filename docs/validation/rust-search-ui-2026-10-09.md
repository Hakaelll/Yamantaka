# Connected local search interface — 2026-10-09

The native interface now exposes global search from **Tools → Global search**, **Ctrl+Shift+F**, and the Privacy settings category. It uses the established graphite/ivory palette, violet controls, Segoe typography, keyboard focus states and a results/context composition. It adds no permanent library sidebar and retains the current reader while browsing results.

## Connected behavior

- Explicit local search consent is saved atomically in the independent Rust preferences. Loading an enabled preference restores consent without automatically starting PDF extraction. Disabling search cancels its query and stops indexing; stored index data can be erased separately.
- Index management exposes incremental start, full rebuild, pause/resume, stop and erasing with an explicit in-interface confirmation. Progress and statistics come from the application actor.
- Queries have a 180 ms typing debounce, cancellation, a separate search error state and retry. Results remain bounded to 80 entries per page. Type, missing-file and changed-file filters use the real repository predicates; reset is visible when filters are active.
- Results show ranked snippets, source kind, page and file state. Selecting a result loads its actual exclusion state. Excluding or reincluding a document uses the existing transactional index repository.
- Opening a result fetches its document independently of the catalogue's current 80-row page. The requested page is retained independently of subsequent result selection. An already active document uses canonical navigation without reopening PDFium; a newly opened document starts at the result page and saves progress.
- Synthetic capture instances now use separate per-process stores so automated verification does not forward to or close the user's demonstration window.

## Executed validation

- `cargo test --workspace --locked`: **113 tests passed**.
- `cargo test -p yamantaka-application --locked`: **19 application tests passed**, including a new real SQLite actor test with 81 documents. It verifies lookup outside the visible catalogue page, absent documents, cancelled lookup, exclusion state and statistics.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo fmt --all -- --check`: passed.
- The executable was built to a separate verification path with `cargo rustc ... -- -o ...` so the running user preview remained available. Rustc reported the expected output-path adaptation warnings; workspace Clippy remained clean.

Six real native screenshots were captured with actual PDFium extraction and SQLite FTS results:

| Capture | Verified state |
| --- | --- |
| [Dark search](../screenshots/rust-search/search-dark.png) | Ten indexed synthetic documents, ranked results, selected context and actions |
| [Light search](../screenshots/rust-search/search-light.png) | Same connected composition with the warm paper palette |
| [Disabled search in Spanish](../screenshots/rust-search/search-disabled-es.png) | Explicit consent and local index explanation |
| [Compact search at 200%](../screenshots/rust-search/search-small-200.png) | Search input fits the narrow viewport; remaining controls are vertically scrollable |
| [Open result page](../screenshots/rust-search/search-open-page.png) | Search for “Mixed” opens page 2 and renders the actual landscape PDF page |
| [Excluded document](../screenshots/rust-search/search-excluded.png) | The selected document disappears from results, indexed count decreases to nine and the include action remains available |

The first batched visual inspection found a clipped results pane and an over-wide compact input. Both were fixed together and the confirmation batch verified the six final captures. The [manifest](../screenshots/rust-search/capture-manifest.json) records executable, source, lockfile, image and control-bound hashes. All six published rasters include native/synthetic provenance metadata.

## Scope and remaining acceptance

These captures use an in-memory synthetic collection and the generated two-page PDF fixture. They are native screenshots, not mockups, and are not evidence of a personal library. Their effective pixels-per-point values do not certify physical monitor DPI, Narrator or multiple-monitor behavior. Controls below the first compact viewport require scrolling.

Full exclusion browsing, trust/recovery flows, source revalidation acceptance, large-corpus search/UI performance and complete rewrite parity remain pending. Annotation/organization result types expose existing stored content; their full creation and editing interfaces have their own outstanding parity work. No replacement release or whole-rewrite completion is claimed.
