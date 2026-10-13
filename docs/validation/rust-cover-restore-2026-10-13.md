# Restore automatic cover selection

The native cover picker now has **Restore automatic cover**. It requests a new
selection while retaining the saved decision. A successful result replaces that
decision and installs the image on the existing physical 3D book. Calculation
failure, cancellation before completion and persistence failure preserve the
stored cover. An accepted atomic write retains its actual outcome if its
presentation token is cancelled later.

## Comparison and persistence

The actor captures an opaque revision of all eleven persisted cover fields
before submitting native detection. Storage hashes each field with its SQLite
type, length where applicable and exact numeric bits. Confidence changes below
normal display precision are still distinct revisions.

The replacement transaction checks the current catalogue identity and compares
the saved revision. A newer decision rejects the replacement. The original row
is retained until detection completes; restoration never deletes it first.
Successful writes clear recreatable thumbnail references and verify all resulting
metadata before committing. Ignored, aborted or mutated writes roll back together.
A revision that has already been replaced cannot be replayed to overwrite the
new decision.

## Presentation and recovery

Restoration is a secondary action below manual confirmation and cancellation.
The English and Spanish picker labels use the existing theme controls. An
accepted request holds navigation and window-close requests while saving, as
manual confirmation already does, and shows **Restoring…**.

If persistence fails, Retry starts another guarded calculation and the saved
choice remains intact. If the choice was saved but GPU presentation fails, the
message reports the saved state accurately. Retry reloads the stored source page
and its preview. The obsolete resident library image is invalidated so leaving
the picker cannot indefinitely present the replaced manual cover.

The compact layout retains visible manual confirmation and cancellation. The
secondary restoration action is below the initial viewport and requires scrolling
at the captured compact size. This checkpoint does not certify keyboard or
screen-reader behavior.

## Executed verification

- 442 workspace tests passed, including all 95 UI-crate tests.
- Two new storage regressions verify guarded replacement, replay rejection,
  changes to persisted metadata, a confidence change at floating-point precision,
  and rollback of ignored, aborted or mutated writes.
- Three real PDFium actor regressions keep a manual choice while detection is
  delayed, restore the selected page, reject a newer manual choice, and preserve
  prior metadata after cancellation, native failure or rejected storage.
- Workspace/all-target Clippy with locked dependencies and warnings denied and
  `cargo fmt --all --check` passed.
- Five successful native synthetic captures cover the enabled action, restored
  dark and Spanish light/reduced-motion galleries, a failed replacement and
  compact 2× text scaling.
- Independent verification reads the actual isolated SQLite choices and checks
  progress, modes, versions, confidence, control states and pixel-cache records.
  It matches all 68 UI source hashes, executable, Cargo.lock, fixture and raw
  image/control hashes. Published PNG pixel chunks match the originals; only
  origin metadata was added. No personal library was opened or published.

Executable SHA-256:
`6505B43D419A23464742B012430EE9DE7A183C2D47BC3DA912BB40656292629A`.

See the [capture manifest](../screenshots/rust-cover-restore/capture-manifest.json),
[enabled restoration control](../screenshots/rust-cover-restore/cover-restore-controls.png)
and [preserved choice after failure](../screenshots/rust-cover-restore/cover-restore-failed.png).

Raw evidence is under `build/rust-visual/cover-restore-final`. The independent
check can be rerun while its executable and isolated stores remain available:

```powershell
python tool/verify_rust_cover_restore.py build/rust-visual/cover-restore-final build/verification/yamantaka-cover-restore-final.exe --published docs/screenshots/rust-cover-restore
```

## Remaining acceptance

Evidence publication includes synthetic screenshots and their source manifests.
Generated PDFs, cache binaries and personal libraries are excluded.

Automatic source/algorithm invalidation, cache cleanup controls, small/large
thumbnail parity, broader protected/changed-file flows and finer native scheduling
remain pending. Post-commit GPU failure handling, physical DPI, Narrator,
multi-monitor behavior, large-library performance and whole-rewrite release
acceptance require broader live verification.
