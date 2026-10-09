# PDF search text-availability evidence

The reader now distinguishes an absent phrase from a PDF with no extractable
text. Both remain valid reading states. An absent phrase suggests changing the
query or options; a document without text suggests continuing to read and using
page bookmarks. This checkpoint adds no OCR or PDF modification.

## Evidence and scope

Each native page-search result carries whether PDFium extracted a non-whitespace,
non-NUL character from that page. A page may contain text even when its query has
no matches. The domain rejects a result that claims matches while declaring text
absent. All constructor callers were migrated to the explicit evidence contract.

The native extraction occurs inside the existing page-search working reservation
and character limit, with cancellation checked before proceeding. No text is
persisted merely to determine availability. Result ownership and original source
bytes retain the existing contracts.

The document coordinator accumulates text presence before releasing empty page
payloads. Finding text anywhere proves availability. Declaring it unavailable
requires a successful scan from the beginning through the complete document.
A partial cursor range with no observed text remains Unknown. Cancelled, failed
or work-limited scans do not publish a false no-text result.

## Verification

A new domain test checks contradictory native evidence, valid text with zero
matches and lease release. A controlled-ticket actor test checks full empty
inspection, text appearing only on the final empty-match page, and a partial
range whose text availability must remain unknown. A real PDFium integration
test searches textual, scanned and 5,000-page blank synthetic PDFs for an absent
phrase, verifies distinct availability states and zero final Text leases, and
compares original PDF bytes after reading.

```text
cargo test --workspace --locked                               185 passed
cargo clippy --workspace --all-targets --locked -- -D warnings passed
cargo fmt --all --check                                       passed
```

Native captures verify the no-match message in dark English and the no-extractable
text message in light Spanish. Anonymous capture state records the availability
enum and zero matches. The PNGs, source and executable hashes, and controls remain
local under ignored `build/rust-visual/find-text-presence`; check logs remain under
ignored `build/verification/find-text-presence-*`.

Broader interaction, all-occurrence highlighting, native work-limit stress,
physical accessibility/DPI, paired performance and all remaining rewrite and
Windows Release gates are still pending. These tests do not establish those gates.
