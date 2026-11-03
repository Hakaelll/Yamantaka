# Floating reader controls and automatic hiding

The primary and reference controls now float over their reading panes instead of reserving PDF space. Each pane honors its `controls_auto_hide` reading preference. Controls hide after three seconds without interaction, or two in fullscreen. A small keyboard-focusable button and the upper edge reveal them. Keyboard focus, an open menu, dragging, a navigator or a modal interaction prevent hiding active controls. Manual visibility and immersive commands still have their explicit exit paths.

Reading options now exposes the auto-hide preference and retains Previous and Next when narrow panes omit those toolbar buttons. Changing only this preference saves through the existing typed preferences action without rebuilding PDF geometry or issuing new render requests. Opening preparation measures the same full viewport used by the reader. The compact full-panel navigator keeps its ordinary toolbar because it displays no PDF.

PDF scroll areas no longer paint egui's edge fades over document content. These fades could use the dark application background while the reader used warm paper. Electronic ink also suppresses the custom control hover animations within the reader.

## Executed evidence

The [result artifact](artifacts/reader-auto-hide.json) records the executable, changed source hashes, control bounds and native snapshots. Captures cover timed hiding, recovery, disabled auto-hide, an open Reading options menu, the compact Spanish interface at 200% text zoom, comparison, electronic ink on OpenGL and settled-reader idle behavior. Snapshot equality checks viewport size, scroll, zoom, visible page rectangles and render generation before and after visibility changes. First-page empty-paint counters remain zero.

The native input probe drives raw egui keyboard events internally. It verifies timeout hiding, Tab/Enter recovery, focus transfer to the first control, retained focus preventing hiding, upper-edge recovery, disabled and blocked policies, document changes and unchanged parent layout. It does not send operating-system mouse or keyboard input. These assertions add no standalone Rust tests.

The before/after images use the same synthetic reader, host viewport and scaling. The new full-height viewport is an intentional design change; equality is required across hiding and recovery within that new viewport. Raw images, control records and the complete capture manifest stay local under `build/verification/reader-auto-hide-final`. The earlier reader reference is `build/verification/eink-profile-final-runtime/eink-standard-reader.png`.

Locked offline build, workspace formatting and scoped all-target Clippy with warnings denied passed. Each capture waits for its own process, rejects nonzero exits and cleans up its owned child on timeout. All capture processes were confirmed terminal before delivery.

## Reproduction and limits

```powershell
./tool/capture_rust_ui.ps1 -Executable build/verification/yamantaka-reader-auto-hide.exe `
  -OutputDirectory build/verification/reader-auto-hide-final `
  -CaseNames @('chrome-hidden','chrome-revealed','chrome-pinned','chrome-menu-held',
               'chrome-compact','chrome-reference','chrome-eink-glow','idle-reader-wgpu',
               'legacy-reader-commands')
```

The internal input probe uses `--demo --verify-reader-chrome=<result.json>`. A missing result or failed assertion invalidates the run. Capture-only modes `--chrome=hidden|revealed|held|pinned` observe the native loop and preserve independent result records.

These are synthetic debug-runtime observations. Physical Narrator, real monitor DPI, fullscreen interaction, prolonged pointer/touch use, every reader/tool combination, Release presentation latency and the remaining rewrite gates are still pending. The acceptance matrix remains Partial.
