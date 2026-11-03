# Native presentation capture preparation

Status: collection tooling is prepared; positive-frame validation and performance acceptance remain pending. No p50/p95, speedup, idle-rendering result or comparison with Flutter is established by this checkpoint.

The existing Dart workflow measures repository operations and Flutter's internal widget/engine work. The Rust catalog example measures repository query/count operations. These do not provide equivalent native display-presentation measurements. The local Flutter SDK is currently absent following the requested cache cleanup, so the complete paired workflow is not ready to run on this checkout.

## Pinned tracing tool

`tool/fetch_presentmon.ps1` retrieves only the portable PresentMon 2.6.0 x64 console executable into the ignored workspace cache. The official [release asset](https://github.com/GameTechDev/PresentMon/releases/tag/v2.6.0) is 980,320 bytes. Its GitHub-reported SHA-256, verified after download, is `b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af`. Existing mismatched files are preserved and rejected. `-VerifyOnly` never downloads.

No MSI, service, account-group change, application dependency or automatic elevation is added. The validation tool is not part of the distributed Yamantaka bundle.

`tool/capture_native_frames.ps1` binds an existing process to its expected workspace executable and records its executable hash/start time. It uses a unique ETW session, a process-ID filter, disabled input tracking and a bounded duration. It does not stop other sessions or generate mouse/keyboard actions. The flags follow the pinned [console documentation](https://github.com/GameTechDev/PresentMon/blob/v2.6.0/README-ConsoleApplication.md).

The command preserves raw CSV and logs under a fresh named verification directory. It rejects an existing output directory and checks process identity after recording. Rows from another PID are rejected. Failed tracing preserves the tracer's nonzero exit code; no-frame or changed/exited-target outcomes return code 2. Every observation keeps `performanceVerdict` as `not-established`, including successful event collection. A controlled workload and its analysis are separate requirements.

## Executed evidence

- The official binary download and cached verification pass. Both PowerShell files parse successfully.
- A five-second probe of the opened Release process fails with PresentMon exit code 6. Windows reports access denied when starting the ETW session and requires administrative privileges or membership in Performance Log Users. No frame evidence is produced.
- The capture script's one-second permission check preserves exit code 6, records `capture-failed` and reports no performance verdict. Zero rows from this failure are not evidence of idle rendering.
- A different executable path is rejected before starting a trace. Reusing the permission-check output is rejected, and its observation retains its original SHA-256.

The [setup artifact](artifacts/presentation-trace-setup.json) records these outcomes. Raw permission-check logs stay local under `build/verification/presentation-permission-check`.

## Next validation step

A one-time UAC-authorized capture has been requested. That authorization is still required before attempting elevation. The scripts do not request it themselves. After permissions are available, establish a positive control with known rendered frames, then define equivalent Flutter/Rust workloads, separate CPU/GPU/display work and collect five interleaved passes. Quiet-process observations or startup screenshots alone cannot establish the 16.7 ms interaction target.

From a suitable PowerShell session, use a live Yamantaka PID and its exact executable path:

```powershell
./tool/fetch_presentmon.ps1 -VerifyOnly
./tool/capture_native_frames.ps1 -TargetProcessId <pid> -ExpectedExecutablePath <workspace-executable> -CaptureName presentation-pass-01 -DurationSeconds 10
```

This checkpoint changes no application runtime code and adds no standalone Rust tests. The rewrite's full performance and physical-accessibility acceptance remain open.
