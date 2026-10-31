# Resume protected metadata analysis after native eviction

## Failure and correction

The protected-import checkpoint handled password challenges before the initial source-revision read. Its metadata scanner had a separate reopening path: if the PDF disappeared while reading front matter, reopening without credentials ended the import with a password-required error. The current page and earlier metadata evidence were not resumed.

The metadata scanner now has an explicit password-wait state. It retains its governed front-matter accumulator, document source and interrupted page while using the same bounded one-shot challenge channel as the initial import. An incorrect credential produces retry feedback. Successful reopening checks page geometry and source revision before reading the interrupted page again. Cancellation and shutdown retain the existing queue and close behavior; no credential is persisted or cloned.

## Executed evidence

The retained real-PDFium import-preemption test reproduced the failure before this correction: the protected metadata restart terminated instead of requesting another password. It passes after the correction. No standalone test case was added.

The extended helper closes the actual native document at either the source-revision handoff or the second front-matter page, then returns the native-unavailable response through the controlled ticket. At the second page, the first page has already been analysed. The test retries an incorrect password, unlocks correctly and verifies the page request sequence `[0, 1, 1, 0, 1]`, including the following ordinary PDF. It compares the complete persisted bibliographic evidence with an uninterrupted import, verifies exactly two catalogue entries and no pending job, and checks that the protected PDF bytes remain unchanged. This exercises a controlled native-owner loss, not a physical driver failure or a full two-reader capacity-pressure scenario.

Formatting, offline all-target Clippy for the application crate with warnings denied, the focused retained test and the native UI build passed. No new UI capture matrix was run. The password dialog remains the one verified by the [protected import checkpoint](rust-protected-import-2026-10-31.md).

## Remaining work

Protected cover generation and full-text indexing still require coordinated access to an explicitly unlocked document. Physical password input, restart matrices, concurrent protected batches and actual capacity-pressure acceptance remain pending. Complete resource accounting, full rewrite parity and Release acceptance remain open.
