# Cover work and protected native readers

Cover rendering now distinguishes a disk-cache lookup from work that needs a
native PDF handle. Two occupied native slots no longer reject a cover whose
pixels are already available, and an unprotected warm handle can be reclaimed
before temporary cover rendering or automatic detection.

## Reproduced behavior

On the preceding checkpoint, two real PDFium regressions failed with
`Visible readers occupy the native PDF slots`:

- A persisted page-one cover was rejected while two reader IDs were protected,
  before its cache could be consulted.
- Both direct cover rendering and automatic detection were rejected while one
  of two native handles was merely warm and unprotected.

The tests were run before changing the worker. Both cases pass after the change.

## Admission and ownership

When both slots are occupied and the requested document has no open handle, a
cover request checks the persistent cache first. The shared cache loader admits
pixels through the memory governor, validates the source identity again after
reading, and checks cancellation before delivery. A valid hit returns without
opening a third document or evicting either reader.

If native work is needed, cover rendering, cover detection and normal document
opening now share the same victim selection. The worker selects and detaches an
unprotected live handle under the scheduler's protection lock. The native handle
is dropped before PDFium opens the temporary document. Pending metadata
inspection and visible reader IDs retain the existing protection policy.

The shared cleanup removes the evicted ID from open recency, page cache, tile
cache and text cache. Temporary PDF handles remain local to the serial worker
operation and are dropped before its result is delivered. Two protected native
handles still reject a cache miss or damaged record that would need another
open. Cache failure does not grant permission to evict a protected reader.

## Verification

- All 448 workspace tests passed, including 95 UI tests. Formatting, workspace
  Clippy with all targets and warnings denied, and the native application build
  passed.
- The cache regression compares the returned pixels with the original native
  raster, checks the new document/page/generation key and observes an actual
  persistent-cache hit with both reader IDs protected.
- Cache misses and corrupted records remain rejected under two protected
  handles; both readers continue to return page text afterward.
- The native admission regression tests direct rendering and automatic
  detection. Pre-cancelled work leaves the warm handle usable. Successful work
  evicts only the unprotected handle and removes its previously populated text
  cache. The protected reader remains usable.
- A subsequent open succeeds after temporary work, and two newly protected
  readers then reject further native cover work. Both readers remain usable.
- Source PDF bytes are compared before and after. The governor returns to zero
  after results and workers are dropped.

No UI component, book material, motion or reading-strip layout changes in this
checkpoint. Real native tests provide admission evidence; existing screenshots
are not presented as proof of simultaneous protected-reader behavior.

## Reproduce

```powershell
cargo test -p yamantaka-storage --test cover_cache_native --locked
cargo test -p yamantaka-pdf --test cover_admission_native --locked
cargo test --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## Remaining acceptance

Warm-handle file/path revision ownership and live invalidation of resident covers
remain separate work. The requested document ID can still reuse an existing
handle without a complete source-revision comparison. Physical DPI, Narrator,
paired performance traces and the remaining rewrite acceptance are unchanged.
This is not a replacement release or a measured performance claim.
