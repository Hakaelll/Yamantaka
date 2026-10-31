# Atomic comparison continuity

Date: 2026-10-31. Scope: default reader-session persistence and restoration of an active comparison.

## Behavior

The default session now saves open readings and the reference pane in one atomic JSON replacement. The reference retains its document, page, normalized position, layout, zoom, theme, rotation, margins, divider, swapped order, synchronization options and marked differences. Restoration uses the existing workspace comparison flow after the primary reading opens. It does not introduce a second native worker or change saved workspace ownership.

Ordinary startup respects Restore last session. A restart following a graphics-device failure restores the saved reading context even when that preference is disabled, without changing the preference. A missing reference leaves the primary reading available and retains the comparison record for later recovery. Explicitly closing the reference or removing its document clears that retained record. Failed automatic close retries do not clear it.

The default-session adapter reads versions 1 and 2. It writes version 2 when a comparison exists and retains version 2 on subsequent saves. Sessions that have never stored a comparison remain version 1. Unknown root, tab and reader-preference extensions survive replacement. Unsupported versions, unknown comparison fields and invalid primary/reference relationships fail without replacing the original snapshot. The four MiB bound remains in force. Embedded workspace reading contexts keep their separate version-one format.

## Executed evidence

Formatting, offline workspace checks for all targets and workspace Clippy with warnings denied passed. The retained historical-schema compatibility test passed with the new comparison assertions. These cover version-one loading, version-two round trips, extension preservation, rejection without replacement and document-removal reconciliation. No standalone Rust test was added. A pre-existing annotation-history test now passes a borrowed identifier instead of cloning it, as required by the current Clippy version.

Three native capture profiles passed: graphics recovery with a shared-document comparison, recovery with a distinct reference, and the existing primary-only recovery case. The controller changes page and zoom, configures reference settings, submits a favorite write, destroys the wgpu device and closes through the persistence barrier. The fresh Glow process must restore the exact comparison and primary position, present both panes and settle governed uploads within the page-payload budget. This is an injected device failure, not physical driver-removal acceptance.

The [capture manifest](../screenshots/rust-comparison-continuity/capture-manifest.json) records the executable, Cargo.lock, source and artifact hashes. Those hashes were checked against the captured build before publication. The later annotation-test-only Clippy correction does not alter the captured application. Published images contain only synthetic fixtures.

`tool/verify_reader_continuity.py` then makes three independent SQLite backup copies of the synthetic distinct-document store. It checks ordinary restoration with wgpu, a reference marked missing, and disabled restoration. All three passed. The missing-reference probe also checks the on-disk comparison after process exit. Hashes confirm that the original fixture database, preferences and continuity file remain unchanged. The [restart report](../screenshots/rust-comparison-continuity/continuity-restart-report.json) records results and artifact hashes; sibling controls JSON files are published, while restart PNGs remain local build artifacts.

To replay after producing the capture fixtures:

```powershell
python -X utf8 tool/verify_reader_continuity.py --executable build/verification/yamantaka-comparison-continuity.exe --capture-directory build/rust-visual/comparison-continuity
```

## Remaining acceptance

This checkpoint does not certify drafts, every in-flight tool operation, physical device loss, interrupted atomic replacement or all repeated close/reopen cycles. Migration from an installed Flutter library, complete memory accounting, performance baselines, physical DPI, Narrator and multi-monitor acceptance remain separate requirements. The rewrite is not ready for its final Release.
