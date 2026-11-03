# Native idle update-loop observation

The settled synthetic library and reader produced no intermediate egui updates during the observed two-second intervals, on both native wgpu and OpenGL compositors. The positive controls produced 89 and 95 updates respectively. All six executions completed with exit code 0 and their processes were confirmed terminal before delivery. One existing interactive preview remained open.

This is one execution per configuration in a debug build. It establishes that these native windows can stop updating after their initial work settles. It does not establish Release CPU use, GPU presentation intervals, energy use, prolonged idle behavior, physical input latency or a comparison with Flutter.

## Method

`--demo --verify-idle --capture=<image.png>` enables an isolated observation through the ordinary native application. The existing capture readiness checks must remain satisfied for two seconds before observation starts. These checks include installed reader images, completed library-cover requests and settled managed page resources. The library cases each retained ten PDF covers. The reader screenshots show the installed PDF page, with zero empty-page paints recorded.

During observation the capture harness stops its usual 100 ms repaint polling. A separate sleeping deadline thread wakes the native context after two seconds. The probe counts completed egui frames through `cumulative_frame_nr`, excluding the initial and final boundary updates. It rejects input events and lost readiness rather than silently restarting the measurement. `--idle-positive-control` deliberately schedules updates during the same interval and must produce at least three intermediate updates. The screenshot is requested after observation, so screenshot activity is outside the interval.

The existing serial capture runner rejects missing outputs, abnormal exits, interrupted observations, unexpected counts and intervals outside 2–5 seconds. Its 45-second timeout cleans up and awaits its own child. Failure to wake or render cannot pass as idle. These checks add no standalone Rust tests.

## Evidence and reproduction

The [result artifact](artifacts/native-idle-loop.json) records the executable hash, changed runtime source hashes, actual durations, compositor identities, frame counts and image/control hashes. Raw screenshots, logs and the complete capture manifest remain local under `build/verification/idle-loop`.

```powershell
./tool/capture_rust_ui.ps1 `
  -Executable build/verification/yamantaka-idle-loop.exe `
  -OutputDirectory build/verification/idle-loop `
  -CaseNames @('idle-library-wgpu','idle-reader-wgpu','idle-control-wgpu',
               'idle-library-glow','idle-reader-glow','idle-control-glow')
```

The frozen executable was built with locked offline dependencies. Scoped all-target Clippy with warnings denied passed. This change adds opt-in evidence collection; it makes no production scheduling change. The complete resource accounting, physical-device and comparative performance requirements remain open in the acceptance matrix.
