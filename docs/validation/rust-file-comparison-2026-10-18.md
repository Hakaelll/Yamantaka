# Background full-content PDF comparison

The legacy fingerprint uses file size and the first 64 KiB. Two different files
can share it. The storage helper could compare full hashes, but no connected
application action exposed that check.

## Connected behavior

Open a book's Details and choose **Compare with another PDF**. A native picker
selects one candidate. The inspector shows a local comparison state with Cancel,
then identical contents, different contents or an actionable error. Cancelling,
changing selection, leaving the library, closing the inspector or exiting cancels
the current request. Late replies must match the pending request identity.

One background comparison may run at a time. The repository actor validates the
selected catalogue document before admission and before accepting the result.
File reads and hashing run on a dedicated worker, independently of PDFium.
The actor waits on channel readiness rather than polling when idle. Shutdown
cancels and joins the comparison worker.

A shared 64 KiB buffer is admitted through the global memory governor. Both files
are hashed completely. The original's bounded fingerprint must still match the
catalogue, while exact equality requires matching sizes and full SHA-256 digests.
The legacy fingerprint formula now has one implementation in the domain crate;
storage and comparison use that same function.

The worker opens regular files read-only. On Windows, its sharing mode denies
concurrent writes and deletion while it holds the proof. Canonical paths, sizes
and modification dates are checked during hashing and at result acceptance.
The file handles stay alive through that acceptance check and are then released.
Errors do not expose full local paths. No comparison writes to the database or
to either source PDF.

## Verification

- `cargo fmt --all --check` passed.
- Workspace Clippy with all targets and `-D warnings` passed.
- Both new comparison tests passed. They exercise a same-size, same-head collision,
  full-copy confirmation, cancelled admission, stale catalogue rejection, changed
  physical source rejection and release of governed memory and file locks.
- All six retained storage compatibility checks passed. Four passed on the first
  run. Two atomic-file checks initially received access-denied errors in the host
  temporary directory; those two passed with temporary files inside the workspace.
- Two native captures drive the real application command and reply route. Their
  control records report `pending=false`, `failed=false` and respectively
  `equal=true` and `equal=false`. The synthetic inspector scrolls to the result
  for capture; the ordinary inspector remains user-scrollable.
- Only the two focused comparison tests and six affected compatibility checks ran.
  The minimal Rust suite now contains 52 tests; this is not a new full-suite run.

The [native captures](../screenshots/rust-file-comparison) include executable,
source and image hashes. Their fixture PDFs are generated synthetic files and
are not published. They do not establish physical DPI, Narrator, native picker
interaction, large-file I/O latency or full rewrite acceptance.

## Remaining scope

This is an explicit read-only comparison of two current files. Its result is a
point-in-time observation, not permission to consolidate files later. Digest
persistence, library-wide candidate discovery and a duplicate-resolution flow
with conflict review, backup and undo remain pending. Imports still preserve
separate documents at separate paths. No release or migration completion is
claimed by this checkpoint.
