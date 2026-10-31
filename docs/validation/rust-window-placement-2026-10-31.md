# Native window placement checkpoint

Date: 2026-10-31. Scope: normal-window geometry, maximization, fullscreen preservation, independent persistence and backup-setting fallback. Full Windows integration acceptance remains open.

## Connected behavior

The native Windows adapter accepts a borrowed window handle, captures `WINDOWPLACEMENT` and restores through the matching API. Stored placement contains only a version, normal rectangle and maximized flag. Decode rejects unknown shapes/versions, non-integers, out-of-range coordinates and invalid dimensions. Native handles never enter the file or the writer thread.

Production startup uses imported Flutter size/position values as the initial fallback, normalizes that initial placement through Windows, and then applies a valid Rust placement. This normalization also precedes rejecting an unsupported window-state file. Normal mode configures the adapter before the first application UI frame; safe mode skips it. A restored backup can supply `nativeWindowPlacement` when the separate file is absent.

A single background writer owns `window-state-v1.json`. Its request queue holds one placement, the UI keeps the latest unaccepted request, and changes are grouped after 500 milliseconds without another change. Completion notifications retain only the latest result. The drawing path does no file writes. Shutdown flushes a pending request and joins the writer before a library restore can move the active store. Window state uses the existing atomic versioned-JSON publisher. Startup reads at most 65,537 bytes and rejects state larger than 64 KiB, redirected/non-file entries and unsupported settings versions.

Normal observations enter the live application settings snapshot used by backup requests. Minimized and fullscreen observations do not replace normal placement. The main application now accepts `--software` as an alias of the existing `--opengl` option.

## Executed evidence

Offline binary Clippy with warnings denied and an offline native build passed. Formatting was applied. No standalone tests or broad workspace suite were added or run for this unit. `raw-window-handle` was already locked transitively; the lockfile adds it as a direct dependency of the native integration and UI packages without upgrading it.

Three isolated native scenarios passed. Each uses the application's own viewport commands and native handles, not external Computer Use:

| Case | Observed result |
| --- | --- |
| window-placement-cycle | Windows adjusts a seeded rectangle at −500,000/−500,000 to a visible normal rectangle. The scenario maximizes, enters fullscreen, exits fullscreen and restores normal mode. The original normal rectangle survives the cycle. The final placement equals the saved settings snapshot and a separate disk read after shutdown |
| window-placement-unknown | Unsupported version 99 state disables placement recording. The file remains byte for byte equal to its retained synthetic source copy |
| window-placement-backup-glow | A real v9 backup export/preview round trip carries the native placement setting without a separate window file. Its restored setting feeds the same native cycle; the final disk read matches the application snapshot. The capture reports the Glow presenter after the `--software` alias |

The normal cycle and backup fallback finish with normal bounds `[0, 0, 900, 700]` and `maximized=false` on this host. These are native placement coordinates, not portable DPI-independent size measurements. Custom-control horizontal bounds passed for all three captures. The cycle and unsupported-state screenshots were visually inspected. Synthetic PDF covers in the captures come from the generated acceptance fixture, not personal documents.

The [capture manifest](../screenshots/rust-window-placement/capture-manifest.json) records Rust-source, executable, lockfile and artifact hashes. Published PNGs include origin metadata.

## Remaining acceptance

Physical monitor removal/reconnection, mixed DPI, every required scale, Narrator and user-driven move/resize/fullscreen timing remain unverified. The viewport cycle does not prove physical F11 interaction. Restoring a maximized state in a separate application process and a real installed Flutter migration still need acceptance. Continuous drag/forced shutdown/write-failure stress and idle/performance measurements remain pending. The compact library layout also needs further visual refinement; these captures establish window-state behavior rather than a completed visual redesign. Device-loss recovery, the broader parity matrix and the final Windows release remain open.
