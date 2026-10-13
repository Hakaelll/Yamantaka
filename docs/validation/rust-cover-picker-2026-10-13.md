# Choose a PDF page as the native cover

The document inspector now opens a native cover picker. It loads the saved
source page, renders a bounded PDFium preview and lets the user move through
the PDF or enter a page number. **Use as cover** saves the completed preview's
page. The library installs that texture on the existing physical 3D book,
including its spine imagery. The original PDF is never a write target.

## Behavior and ownership

Preview requests retain document, page, generation and cancellation identity.
Changing the page cancels the previous preview. Save stays disabled until the
selected page has a matching, successfully uploaded native preview. Cancelling
the picker releases its pending read and preview without changing the saved
choice. The previous library texture remains available during editing.

Saving uses a validated domain value and a SQLite transaction. It checks the
current catalogue path, fingerprint and page count, writes manual selection
metadata and clears recreatable thumbnail references. It reads the resulting
choice and cleared document thumbnail back before committing. Ignored,
aborted or mutated writes roll back the transaction. Progress and the PDF's
identity remain unchanged. This catalogue identity check does not replace
external file-change detection.

An accepted write retains its actual outcome even if its presentation token is
cancelled later. Navigation and window-close requests are held while the save
is pending. A save failure keeps the draft and preview, exposes Retry and
allows cancellation. A successful reply transfers the prepared GPU image into
the library cache and closes the picker.

## Presentation decisions

| Before | After | Reason |
| --- | --- | --- |
| Existing cover decisions could be rendered but not edited natively | Dedicated page with an editorial heading, actual PDF preview and explicit confirmation | Make the cover decision visible before writing it |
| A wide wrapped row could vertically misalign the form | Preview and form align at their top edges | Keep the page controls beside the preview |
| Enlarged text could place confirmation below the initial compact viewport | Compact layout places controls before the scrollable preview | Keep confirmation and cancellation visible at the captured size |

Dark graphite and warm ivory surfaces share the existing violet action color.
Spanish and English labels use the same control contracts. This change adds
no autonomous animation; the gallery retains its existing damped book physics
and reduced-motion presentation.

## Verified evidence

- 414 workspace tests passed, including all 95 UI-crate tests.
- Three new SQLite tests cover persisted manual intent, invalid or changed
  documents, and four trigger-induced rollback scenarios.
- A real PDFium actor integration test previews a later page, checks a cancelled
  save, persists a successful choice, cancels its presentation token after the
  actual success, renders that page through normal cover loading and compares
  the original PDF bytes.
- `cargo fmt --all --check` and workspace Clippy with all targets, locked
  dependencies and warnings denied passed.
- Six native synthetic captures cover dark, Spanish light with reduced motion,
  successful save, cancellation, failed save and compact 2× text scaling.
- The independent verifier checks all 67 UI source hashes, Cargo.lock,
  executable and capture hashes. It reads the isolated file-backed SQLite
  results for save, cancel and failure. It also checks confirmation/cancellation
  control bounds, including the compact viewport.
- Published PNGs retain identical pixel chunks. Publication adds origin
  metadata only. No personal library was opened or published.

Executable SHA-256:
`D5C559EEF2FC439B14C43C4D4F6AA51BE414D7E2E7FFE235B761D84CD80F1EE0`.

See the [capture manifest](../screenshots/rust-cover-picker/capture-manifest.json),
[dark picker](../screenshots/rust-cover-picker/cover-picker-dark.png),
[saved cover](../screenshots/rust-cover-picker/cover-picker-saved.png) and
[failed save](../screenshots/rust-cover-picker/cover-picker-failed-save.png).
The white preview is the fixture's actual second page, with a study heading.
It is deliberately different from the colored first-page artwork.

The capture driver invokes the connected controller and actor operations; it
does not establish end-to-end pointer, keyboard or screen-reader acceptance.
The compact case uses `--size=640x480 --text-scale=2`, records a 320×240 logical
viewport and reports 2.5 pixels per point on this host. It is not physical
monitor DPI certification.

The verifier can be rerun while the executable and isolated capture stores
remain available:

```powershell
python tool/verify_rust_cover_picker.py build/rust-visual/cover-picker-final build/verification/yamantaka-cover-picker-final.exe --published docs/screenshots/rust-cover-picker
```

## Remaining acceptance

Automatic candidate scoring, persistent thumbnail generation/cache recovery,
broader protected or changed-file cover flows and retry interaction acceptance
remain pending. The picker currently requires the native WGPU compositor;
Glow reports an unsupported preview instead of saving an unseen choice.
The existing two-handle PDF admission limit can reject a background preview
when both handles belong to other protected foreground documents.

Physical DPI, Narrator, multi-monitor behavior, large-library performance,
whole Flutter parity and final-release acceptance remain unverified. This is
a verified development checkpoint, not the completed rewrite or release.
