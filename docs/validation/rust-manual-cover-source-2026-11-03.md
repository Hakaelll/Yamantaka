# Manual cover previews reject changed PDFs

The picker previously saved a page from an obsolete preview if its PDF changed while the picker was open. The catalogue record had not changed, so its existing checks accepted the save. A native baseline reproduced this after replacing an owned synthetic PDF and restoring its modification date.

## Source binding and recovery

Fresh and cached cover images now carry the physical Windows file version that produced their pixels. Generation changes preserve that runtime token. The picker requires a bound preview before enabling **Use as cover**.

Saving starts one bounded background verification job. It acquires a read handle that denies concurrent writes and deletion, verifies the preview's version, and returns ownership to the application actor. The actor rechecks the handle and current path before saving through the existing repository transaction. The handle remains open through the database result and closes on every exit. Cancellation before saving preserves the previous decision; cancellation after a successful commit does not hide its actual outcome. Shutdown joins the verification worker.

A changed, unavailable, or unverifiable source produces a typed error. The picker preserves the previous saved cover, disables saving the old preview, and offers **Refresh preview**. Refresh obtains a new image and version before another save. ES and EN use the existing error layout and controls.

The token is not persisted. The database schema, backup format, inherited fingerprint, and full-content hash contracts are unchanged. A physical stamp remains a filesystem hint rather than a complete content digest.

## Executed evidence

- The baseline saved the obsolete preview as manual page 1. With the fix, the same native sequence rejected it and retained manual page 0 with its original decision timestamp.
- A native refresh sequence first rejected the stale preview, then obtained a new preview and saved manual page 1. Read-only SQLite checks confirmed both results and retained title/author overrides.
- A simulated SQLite save failure retained the error/retry path without misreporting a source change or a successful save.
- Native captures covered the English desktop error and the Spanish error at 640×480 with text at 200% and reduced motion. The latter uses the existing scrollable form; the recovery control remains enabled below the fold. These are rendered checks, not physical mouse, keyboard, or Narrator acceptance.
- One retained Windows test was extended, with no new standalone test cases. It verified that a held lease denies writes and deletion, releases those restrictions on drop, and rejects a later source revision even after its modification date is restored. The test passed.
- The native debug build, workspace formatting, and workspace Clippy across all targets with warnings denied passed. Third-party dependency versions and checksums are unchanged.

The [status artifact](artifacts/manual-cover-source.json) contains the native outcomes without document paths. Captures and isolated synthetic stores remain local under `build/verification/cover-picker-source` and `build/demo-store/captures`.

This checkpoint covers manual picker saves. It does not establish automatic-restoration race acceptance, remote/removable filesystem behavior, a bound on blocked Windows I/O, full-content identity, reader refresh behavior, physical accessibility, or whole-rewrite completion.
