# Visible covers follow physical source changes

The native library previously checked resident covers against catalogue values only. Replacing a PDF without changing its catalogue record left the earlier texture on screen. A synthetic native run reproduced that behavior after replacing the file and restoring its modification date.

## Source observation and ownership

One Windows worker now observes at most sixteen explicitly supplied visible PDF paths. It reads file attributes rather than PDF contents, never enumerates a folder, and never imports or removes a document. The initial observation precedes ordinary cover admission. A one-second wait separates subsequent rounds for an unchanged visible set. The worker publishes a coalesced latest batch only when its observations change. Empty subscriptions sleep until new targets arrive. Target generation and path checks discard observations for retired slots.

Filesystem queries run outside the UI and PDFium workers. Each attribute handle permits concurrent reads, writes and deletion and is closed after inspection. Target changes and shutdown are checked between files. Shutdown joins the worker after any current Windows attribute query returns; no bound is claimed for an unresponsive filesystem or network provider.

Changed or unavailable sources cancel the earlier cover token, release deferred pixels, remove the stale GPU surface and retire its displayed source page. An available source schedules a fresh cover through the existing actor and native worker. Saved page choices remain authoritative. Unavailable observations do not write a missing flag or change catalogue identity. Entries already marked missing retain their explicit file-verification workflow. The existing 3D geometry and pointer physics are unchanged.

## Cache and native source binding

The shared Windows stamp contains volume/file identity, size, creation time, last-write time and change time. The [Windows FILE_BASIC_INFO contract](https://learn.microsoft.com/windows/win32/api/winbase/ns-winbase-file_basic_info) distinguishes the last-write and change timestamps. The implementation captures the stamp from an open handle and checks that it remained stable during inspection.

Both the disk cover-cache key and the native PDF owner's source guard now include that physical version. The cache also checks the open source handle and the current path after hashing its bounded prefix. This closes a reproduced cache-key collision after a same-size write beyond the first 64 KiB with the modification date restored. A changed visible native owner is rejected rather than silently replaced; an unprotected owner can follow the existing reload path.

The domain's inherited source fingerprint and full SHA-256 contracts are unchanged. A physical stamp is a filesystem version hint, not a complete content digest. It cannot replace full verification for duplicates or prove that arbitrary providers expose every possible change. Existing pixel-cache entries regenerate under the new key; saved cover choices and source PDFs are preserved.

## Executed evidence

- A native baseline reproduced the resident stale image. The fixed native run observed one invalidation, a new GPU image and cancellation of the earlier request after replacing the owned PDF and restoring its modification time.
- A read-only check of the isolated store confirmed the saved manual page, selection mode, decision timestamp, title/author overrides, favorite and reading position remained unchanged.
- Native bulk regeneration under GPU pressure and the manual cover picker completed with all ten synthetic covers ready. The original generated PDFs retained their hashes.
- Three retained tests were extended without adding standalone test cases. They cover Unicode paths, same-size/date writes, disappearance and return, subscription generation, empty-subscription sleep, lack of repeated unchanged-source notifications, the cache collision and rejection of a changed visible native source.
- The cache collision assertion failed before the key change and passed afterward. Atomic cache persistence was denied in the restricted test environment; the same implementation passed with normal host permissions. No persistence checks were weakened.
- Native debug build, workspace formatting and Clippy across all targets with warnings denied passed. Third-party package versions and checksums are unchanged; Cargo.lock records the added workspace edges and the existing test-only temporary-file dependency.

The [status artifact](artifacts/live-cover-sources.json) records the native outcomes. The three related retained tests passed; this checkpoint did not rerun the entire workspace test suite. It does not establish frame-time, CPU or full process-memory targets, remote/removable filesystem acceptance, instantaneous change detection, active-reader refresh behavior, source-change races while a cover picker is open, automatic metadata reanalysis, physical accessibility or whole-rewrite completion. Filesystem observation does not silently adopt a replacement PDF as a new catalogue identity.
