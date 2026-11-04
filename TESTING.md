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
