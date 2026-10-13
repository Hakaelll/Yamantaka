# Persistent native cover pixels

Native PDF cover loading now uses an independent, recreatable disk cache.
Installed mode places it under the Rust cache directory in `covers-v1`.
Portable mode uses the portable Rust cache path. Synthetic demonstrations and
captures use their isolated store's `cover-cache-v1` directory.

The native worker receives the cache through a domain contract implemented by
storage. PDFium remains owned by one worker; the UI performs neither PDF calls
nor cache reads. An available cache record restores the selected page's pixels
and binds them to the current document, generation and cancellation request.
The existing native GPU upload and physical book presentation remain in use.

## Record validity and resource limits

Cache identity includes a format version, canonical path, source size,
modification timestamp, selected page and a SHA-256 digest of the first 64 KiB.
The source identity is checked before and after reading a cache entry or
rendering a replacement. Already-open native handles bypass the disk cache,
because their loaded PDF revision may differ from the current path.

The binary record contains a magic/version, identity, dimensions, checksum and
RGBA pixels. Dimensions must be between 1 and 512, and file length must match
exactly. The checksum covers dimensions and pixel bytes, including a changed
shape with the same byte length. Pixels are allocated through the shared memory
governor before reading. Reads and writes check cancellation in bounded chunks.

Writes use a temporary file, sync its contents and atomically persist it in the
cache directory. The configured `maxCacheMegabytes` preference controls the
disk quota, with the existing 64–1024 MiB range and 256 MiB default. Oldest-written
owned records are removed to fit the incoming record, with a 4096-record cap.
Reopening with a smaller quota also trims existing records on the first load,
without requiring a new render or cache write.
Unrelated filenames and symlinks are excluded from eviction. An interrupted
replacement may lose a recreatable old entry; it never changes the PDF or the
saved cover decision. Crash-left temporary files are not yet swept.

Missing, malformed, truncated, oversized or checksum-invalid records fall back
to rendering. A failed cache write does not discard a successfully rendered
cover. Memory admission failure remains a memory-budget error. Selecting a new
source page naturally uses a different cache identity; previous choices cannot
replace the new selected page.

This is a bounded 512-pixel native presentation cache. It does not populate the
legacy small/large thumbnail path fields or claim full Flutter thumbnail parity.

## Executed checks

Five storage tests cover pixel round trips after reopening the cache, rebinding
document/generation identity, live memory accounting, corruption and invalid
dimensions, changed source/page/path keys, cancellation, memory denial, eviction,
unrelated Unicode filenames and a quota smaller than one record.

Two real PDFium integration tests verify a cold render, a cache hit after worker
restart, identical returned pixels, a changed request identity, regeneration of
a damaged record and delivery when the cache directory is occupied by a file.
The restart test compares the original PDF bytes and checks that governed memory
returns to zero when results and workers are dropped.

The final workspace run passed 421 tests. Workspace/all-target Clippy with
locked dependencies and warnings denied passed, and the format check passed.
Two native synthetic captures show the gallery and a saved second-page cover.
The full gallery installs ten covers and writes ten valid cache records. With
the inspector open, five visible covers produce five records. Both captures
have zero pending cover replies. These cold-cache captures establish connected
presentation and disk writes; the native integration test establishes reuse.

Independent verification checks all 67 UI source hashes, executable, Cargo.lock,
image/control hashes, installed source pages, actual cache record dimensions,
identity filenames and checksums. Published PNG pixel chunks match the originals.
No personal library or cache was published.

Executable SHA-256:
`157D9B011BF2F801722C3F5FF6D86B36498EE4CC65EA565ACCFEE791DD4B0264`.

See the [capture manifest](../screenshots/rust-cover-cache/capture-manifest.json)
and [native gallery](../screenshots/rust-cover-cache/pdf-covers-dark.png).

Run the focused checks with:

```powershell
cargo test -p yamantaka-storage --test cover_cache --test cover_cache_native --locked
python tool/verify_rust_cover_cache.py build/rust-visual/cover-cache-final build/verification/yamantaka-cover-cache-final.exe --published docs/screenshots/rust-cover-cache
```

## Remaining acceptance

Automatic cover candidate scoring, small/large persistent thumbnails, explicit
cleanup/recovery controls and crash-left temporary-file removal remain pending.
The source key follows the bounded fingerprint convention; a modification beyond
the first 64 KiB that preserves size and timestamp requires the remaining file
revalidation flow. It is not a full-content equality proof.

The existing native two-handle admission gate still applies before cover work,
including a possible cache hit. Broader protected-file flows, cancellation during
every I/O phase, hostile external mutation, concurrent processes, disk pressure,
large-directory scheduling latency and paired Flutter/Rust performance remain
unverified. No measured speedup, physical accessibility certification or final
release acceptance is claimed.
