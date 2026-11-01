# Cache covers while importing a protected PDF

## Connected behavior

A newly imported PDF that required a password now captures its automatic cover while the importer still owns the unlocked native document and its full-content source read guard. The dedicated PDF worker checks the expected native source revision before detection and checks the source again around cache persistence. This operation uses the existing bounded cover detector and its background priority. Ordinary rendering from an already-open document retains its existing no-cache restriction.

The importer waits for confirmed native closure and validates the full-content reference before saving the book. It maps the temporary native cover identity to the committed catalogue ID while sharing the governed pixel buffer. Automatic cover persistence uses the existing revision checks; a current selected/manual cover remains authoritative. Source comparison uses the guarded canonical path, including when import arguments use ordinary Windows paths rather than their canonical extended form.

The library can then load the stored cover choice and local pixel cache without reopening the encrypted PDF. No password is cloned or persisted. Cover images are local cache files, not encrypted PDF replacements. Cache maintenance can remove them. Source PDFs remain unchanged.

Capture is skipped without an enabled cache and during metadata reanalysis. If the native document disappears before capture, the importer reopens through the password challenge and repeats the bounded metadata scan before trying again. Cancellation closes the native work without importing a new book. A cache or cover-selection write failure preserves the imported book and permits the procedural cover fallback; optional cover work is not a reason to lose metadata or restart a completed import.

## Executed checks

Formatting, offline workspace Clippy for all targets with warnings denied, the focused retained native import test and the native application build passed. No standalone Rust test case was added.

The retained helper imports through a noncanonical path with a real disk cover cache. After the original runtime has dropped, a fresh PDF worker cannot open the protected PDF without a password but can load its cached cover. Its dimensions and every pixel match the corresponding unencrypted source page, and its render key carries the committed catalogue ID and requested generation. The test also checks rejection of a mismatched expected source and a missing native owner. Existing retry, cancellation, shutdown, interrupted-analysis and unchanged-original checks remain in the same helper.

Two synthetic native captures passed: successful import with the imported UUID present in the library's rendered source-page map, and cancellation without an imported document. The successful gallery screenshot was inspected. The executable, lockfile and every manifest source hash were checked against the current tree. See the [capture manifest](../screenshots/rust-protected-covers/capture-manifest.json). The cold-cache check restarts the PDF worker; it is not a full application-restart matrix.

## Remaining acceptance

Unlock-driven durable rebuilding for existing protected entries after cache removal or corruption, protected manual-page selection, full-text indexing, repeated capacity-pressure eviction, disk-failure UX and full resource/performance acceptance remain pending. The imported-cover path does not retain a reusable credential or establish complete protected-document parity. This checkpoint does not satisfy the complete rewrite or Release gates.
