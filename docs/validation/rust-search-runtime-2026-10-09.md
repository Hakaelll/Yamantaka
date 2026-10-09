# Search actor and native slot handoff — 2026-10-09

The application actor now owns the incremental search controller and the SQLite search repository. Typed actions expose opt-in enable/disable, start/rebuild, pause, cancel, exclusions, clearing, search queries, index statistics and the visible native document set. Search is rejected while indexing consent is disabled. Progress is an overwritten bounded snapshot, independently readable without querying SQL or PDFium.

Opening a reader, admitting an importer or changing visible documents yields the index slot first. Deferred commands retain their existing admission reservations. The actor waits for the index close reply before reusing that slot and prevents new indexing during foreground inspection or import. Two visible documents pause indexing. Regranting a slot respects a manual pause.

Native indexing cancellation is now separate from cancellation of the whole run. Yielding discards late open/text replies, closes the handle and puts the interrupted document back at the front of the bounded candidate queue. Resumption reads its durable SQLite cursor. A failing step aborts once, records only a redacted error category and retains any native close ticket until it is consumed. Shutdown cancels native work and joins closure independently of UI result draining.

## Executed validation

- `cargo test --workspace --locked`: 112 tests passed, including the existing real PDFium, SQLite, DX12 and Windows process coverage.
- `cargo test -p yamantaka-application --locked`: 18 application tests passed (seven actor reservation tests, six controller integrations and five search actor integrations).
- After adding the final terminal-state assertion, all five `search_runtime` tests passed again.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo fmt --all -- --check`: passed on the final code.

The two added controller integrations use real SQLite and deliberately deferred native replies to verify slot yielding during open and text, late payload rejection, cursor preservation, waiting for close and independent manual pause.

The five new actor integrations verify:

1. Disabled consent rejects indexing and search; catalogue SQL remains responsive during pending PDF work and cancellation.
2. Reader admission waits for the index close reply; indexing resumes from the committed page and obsolete text is absent from FTS results.
3. Two visible readers prevent index admission; a cancelled importer never opens a PDF and index admission resumes afterward.
4. A deliberately missing index table aborts once, preserves a redacted terminal snapshot and leaves catalogue queries responsive.
5. Shutdown cancels a pending index open and completes after its close reply, while 128 ordinary UI result reservations remain undrained.

The new integration tests use a controllable asynchronous PDF adapter rather than PDFium so they can hold each scheduling boundary deterministically. Real PDFium coverage is provided by the separate native PDF suite; these actor tests do not constitute end-to-end native UI acceptance or performance measurements.

## Remaining scope

The search settings and results interface is not connected yet. No current UI action submits the newly added search or statistics actions. Persisting consent through that interface, localized index management, result navigation, full native reader/import/search acceptance and the remaining rewrite parity requirements are still required. This checkpoint is not a release.
