# Validation

The owner confirmed that all validation was complete on 2026-11-04 and instructed
completion of the release. Review 13 was explicitly tested without reported
faults. The [final acceptance record](docs/validation/artifacts/owner-final-acceptance-2026-11-04.json)
records the source and limits of that confirmation.

No new agent-run application tests, captures, physical checks or comparative
measurements were added for release finalization, as requested. Historical
reports preserve what actually ran. Owner acceptance is not a new measured
p50/p95 dataset or an agent-run DPI/Narrator result.

## Contributor checks

Use the pinned Rust toolchain, MSVC build tools and Windows SDK. The workspace
retains 56 essential tests; the suite was deliberately reduced at the owner's
request. PDFium cases use the pinned native library and synthetic PDFs.

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
```

Native temporary-file cases require a writable local temporary directory. Use a
workspace directory for `TMP` and `TEMP` when the host restricts system temp.
Do not use a personal library or PDF as a destructive test fixture.

Distribution packaging separately validates executable resources, pinned native
libraries, inventory, hashes, licensing and SBOM. Use
`tool/package_rust_windows.ps1 -VerifyOnly -OutputDirectory <bundle-directory>`.
The detailed [parity record](docs/architecture/rust-rewrite-parity.md) retains
individual execution results and their original scope.

## Native interface regression checks

Use the synthetic study collection to compare library, book details, reader
and Settings. Run each revision's capture harness in a separate output
directory and match the fixture flags. The capture manifest records executable
and source hashes.

```powershell
./tool/capture_rust_ui.ps1 -OutputDirectory build/verification/redesign-smoke -CaseNames @(
    'readme-library-dark', 'readme-library-light', 'library-1280',
    'pdf-covers-small-200-es', 'pdf-covers-details-small-200-es',
    'chrome-revealed', 'chrome-compact', 'chrome-reference',
    'find-short-wide', 'reader-tabs-small-200', 'eink-settings-small'
)
```

Inspect PNGs together with their control records. Check the reported viewport
and pixels per point against the requested size. A window larger than the
monitor can be clamped. Compare compact layouts at 200% application text and
use physical Windows scaling and Narrator for desktop accessibility sign-off.

The `--demo --verify-reader-chrome=<result.json>` and
`--demo --verify-reader-profile-input=<result.json>` probes exercise synthetic
keyboard, focus, viewport and scroll/zoom behavior in egui. Native captures
also cover protected import, source relocation and annotation recovery with
isolated data. Use the release acceptance record for the scope of older tests.

### Redesign evidence, 2026-11-07

| Workflow | Native evidence |
| --- | --- |
| Library | Light and dark, ES and EN, first book visible at 640 × 480 and 200% application text. Catalogue menus, empty library, empty results and Continue with filters. |
| Book and file | Full book details, metadata edit, cover choice and save/cancel, source verification and relocation review, protected import retry and cancellation. |
| Reader | Matching before/after cases for page turns, zoom and tab continuity. Chrome visibility preserves the PDF geometry. Search retains its highlight after dismissal, including compact and short-wide study layouts. |
| Study and comparison | Saved annotation in a rotated document, compact page-note editor, comparison and synchronization, and review of an unsaved reference note. |
| Settings and management | Horizontal categories, compact selector, electronic ink, backup export, and compact collection, tag, rule, history, note and duplicate views. |

1280 × 720 was measured at the requested viewport. The host clamped 2560 × 1080,
so the comparison used the same aspect ratio at 1536 × 648. These runs used
125% host scaling. Narrator, other physical scales and first-use pointer
walkthroughs remain separate checks. Compact selection-note, compact reference
shortcuts and reference history timed out on both the baseline and a redesign
build. The passing note and comparison cases above used separate fixtures.
