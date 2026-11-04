# Contributing to Yamantaka

The active product is native Rust for Windows. The owner accepted the replacement on 2026-11-04; recover Flutter from the [archived checkpoint](docs/legacy/README.md). Start with the [product brief](PRODUCT.md), [design system](DESIGN.md) and [acceptance matrix](docs/architecture/rust-rewrite-parity.md).

## Set up the native workspace

Use Windows x64, rustup and the Microsoft C++ build tools with a Windows SDK. The repository pins its Rust toolchain and dependencies.

```powershell
./tool/fetch_rust_pdfium.ps1
cargo build --locked -p yamantaka
```

For generated PDF fixtures and screenshot tools, use Python with `Pillow`, `reportlab` and `pypdf`. Prepare the native corpus before running its tests:

```powershell
python ./tool/create_rust_pdf_fixtures.py
python ./tool/create_rust_metadata_fixtures.py
python ./tool/create_rust_cover_fixtures.py
python ./tool/create_rust_links_fixture.py
```

Keep generated output under `build`. Never replace a fixture generator's output with a personal PDF.

## Make a focused change

- Keep SQL in storage and native PDFium calls on the owning PDF worker. UI components dispatch commands and receive state.
- Preserve source PDF bytes, identifiers, manual corrections, persisted relationships and cancellation ownership.
- Reuse existing components and tokens. Follow the project UI workflow and respect reduced motion, keyboard focus and text enlargement.
- Explain new dependencies and native patches in the relevant decision or patch documentation. Keep upstream changes separate from application fixes.
- Update the acceptance matrix when behavior or verified scope changes. A successful build alone does not prove a visual or functional change.
- Keep tests focused on meaningful behavior. Reuse retained coverage rather than adding large overlapping suites.

## Verify the affected behavior

Run the smallest relevant check while implementing. Before a complete checkpoint, run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked -- --test-threads=1
```

Use `--offline` after dependencies are available to keep verification independent of network access. If the host's temporary directory denies native file operations, point `TMP` and `TEMP` to an owned directory under `build/verification` for that process only. Preserve nonzero exit codes and inspect the result.

Native checks require the pinned PDFium DLL and generated corpus. Physical DPI, Narrator, monitor changes and paired performance measurements are separate acceptance work; document what actually ran.

## Preview without accumulating windows

The demo uses synthetic data and does not open the personal library. With fixtures prepared, create a frozen executable and use the managed launcher:

```powershell
New-Item -ItemType Directory -Force build/verification | Out-Null
$preview = 'build/verification/yamantaka-preview-' + [guid]::NewGuid().ToString('N') + '.exe'
Copy-Item target/debug/yamantaka-rust.exe $preview
./tool/open_rust_preview.ps1 -Executable $preview
```

The launcher replaces managed synthetic previews and leaves ordinary application instances intact. For captures, select only cases relevant to the change:

```powershell
./tool/capture_rust_ui.ps1 -CaseNames library-dark,chrome-hidden
```

Capture processes are awaited and cleaned up before another case starts. [Native close checks](docs/validation/rust-native-close-2026-11-03.md) record the tested limits. Review old frozen executables with `tool/prune_rust_checkpoints.ps1`; use `-Apply` only for its reviewed candidates. Its scope excludes source, PDFs, libraries and compiler dependencies.

Checkpoint cleanup queries Windows process paths through CIM, including applications outside the current shell job. It stops without removing files if process paths cannot be inspected, and retains the two newest frozen executables plus every executable currently in use.

## Keep checkpoints public-safe

Never commit personal PDFs, library databases, credentials, raw document diagnostics or machine-specific paths. Publish only generated demonstration captures with their provenance. Do not log passwords or add accounts, telemetry or network calls to normal library use.

Before committing, inspect the diff and exact staged files, then run:

```powershell
python ./tool/check_public_checkpoint.py --staged
```

The guard complements human review. Describe the changed behavior, relevant checks and remaining limits in English. Preserve a user's unrelated working-tree changes.
