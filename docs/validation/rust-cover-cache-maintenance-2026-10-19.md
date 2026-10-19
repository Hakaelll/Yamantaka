# Cover cache measurement and cleanup

Settings → Storage measures the disk space occupied by recognized cover pixel
files and explicitly frees those images. The size is an observation from that
scan, not a live total. PDFs, catalogue metadata, saved cover page decisions,
file-hash references and resident GPU textures are outside this operation.
Cover images regenerate on demand and the disk cache can grow again.

## Ownership and scheduling

A dedicated application worker holds the cache capability, not a database or
PDFium handle. The runtime admits one maintenance job, retains its command
reservation through a single terminal reply, and waits on its bounded channel
when idle. Shutdown cancels and joins the worker before releasing the engine.
Matching cancelled requests still deliver their partial removal totals.

Inspection and cleanup stream the flat directory without collecting its entries.
A missing cache directory reports zero without creating it. Cleanup holds the
cache writer lock; native cover load admission and save use try-lock and fall
back to rendering or skipping the disk write rather than waiting for that lock.
Already loaded images remain available. Inspection can race with ordinary cache
activity, so errors are reported and the displayed inventory is not transactional.

Only ordinary files with the expected hexadecimal identity name, version-1
header, matching embedded identity, bounded dimensions and exact pixel length
are removed. Recognition reads an 80-byte header, not all pixels. It does not
certify their pixel checksum. Other names, future formats, directories and links
are preserved. The cache root must be absolute and ordinary; Windows maintenance
opens it without following its final reparse point and holds a directory handle
that denies root deletion. This is flat cache maintenance, not recursive cleanup
or a defence against every hostile ancestor-path replacement race.

Cancellation is checked between entries and while awaiting the writer. It
retains counts already removed and leaves the remaining size unknown. Files
that cannot be inspected or deleted contribute to the failure count. The UI
keeps cancellation and errors local to Storage and allows another measurement.
Filesystem calls already in progress cannot be forcibly interrupted.

## Executed verification

One retained storage unit test runs the real bundled PDFium engine. It generates
a cover from PDF page 1, verifies its disk inventory, and holds the cache writer
while another native request renders identical pixels within a five-second
deadlock guard. A cancelled cleanup preserves the cache. A completed cleanup
removes the recognized image while keeping the source PDF byte hash, an unknown
format, a Unicode filename, a text file and a nested directory. The protected
reader still supplies text, and page 1 regenerates with identical pixels.

The test passes after the final storage changes. Workspace all-target Clippy
with warnings denied, Cargo formatting and the offline native development build
pass. The suite contains 54 test cases, compared with the 536-case baseline;
this checkpoint adds only the deletion and native-contention check. The full
suite and broad performance benchmarks were not rerun.

[Native synthetic captures](../screenshots/rust-cover-cache-maintenance) exercise
the real runtime command and worker in dark, Spanish light with reduced motion,
and a small viewport with 200% text. Inspection finds one 786,512-byte image;
cleanup removes that image, reports zero owned bytes remaining and keeps the
neighboring synthetic file. The small capture scrolls the actual settings area
to expose the amount and both actions. Source and executable hashes match the
capture manifest. No personal library or PDF appears in these public artifacts.

## Remaining acceptance

This does not clear full-content references, search indexes, stale temporary
files, compiled binaries or every application cache. Global orphan pruning,
large-directory timing, forced-shutdown and permission fault injection, physical
DPI/Narrator operation and paired Flutter/Rust performance acceptance remain
pending. This checkpoint is not the completed rewrite or a release build.
