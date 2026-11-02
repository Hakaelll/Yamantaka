# Duplicate verification after Flutter timestamp conversion

## Defect

The exact-duplicate rule simulation compared the stored modification date directly with the native filesystem timestamp. Flutter serializes dates at microsecond precision, while Windows can retain finer precision. A migrated record could therefore fail source verification even when its size, legacy fingerprint and actual bytes were unchanged.

A deterministic reproduction sets three synthetic files to a timestamp with a nonzero sub-microsecond component and stores microsecond JSON dates. The retained application test failed with three unverified sources instead of zero before the fix.

## Correction

Stored-date checks now share the same conversion and precision allowance used by file verification: RFC3339 or unambiguous local dates, with a maximum difference of 999 nanoseconds. Invalid or ambiguous dates are not accepted. This allowance applies only to matching inherited catalogue dates; native source revisions still require exact modification times.

Exact duplicate identity still requires matching file sizes and freshly computed complete SHA-256 hashes. A shared first-64-KiB fingerprint remains insufficient. Review and execution still reopen and reverify the exact native source revision and full digest. Original PDF files remain read-only.

## Evidence and limits

The retained application test covers the reproduction, UTC and local date representations, native precision, rejection of a full microsecond difference and malformed dates. Existing different-tail collision, cancellation, catalogue replacement, plan review and execution checks remain in that test path. No standalone test was added.

Formatting, all four retained application library tests and workspace all-target Clippy with warnings denied passed:

```powershell
cargo fmt --all --check
cargo test -p yamantaka-application --lib --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
```

This fixes the existing exact-duplicate automation flow. A dedicated library duplicate-review and consolidation workflow remains pending. No UI layout changed, and this checkpoint does not establish complete migration or Release acceptance.
