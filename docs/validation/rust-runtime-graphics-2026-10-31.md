# Runtime graphics recovery checkpoint

Date: 2026-10-31. Scope: checked shutdown and a single alternate-compositor restart after a reported native wgpu fault.

## Connected behavior

A shared monitor observes unexpected device loss and uncaptured wgpu errors. It requests a repaint without recording driver messages or document paths. Intentional device destruction during ordinary shutdown is ignored. Once a fault is observed, the app requests close and stops building its reading interface.

The shutdown path fences accepted application commands before cancelling pane state. The fence waits for earlier queued or deferred commands to be handled, with bounded scheduling and response timeouts. It is not a completion guarantee for every long-running native job. Import recovery, reader continuity and reading-session persistence use their existing synchronous shutdown paths. A failed fence or persistence operation marks the handover unsafe. Only a completed `on_exit` and successful checkpoint permit a runtime-fault retry.

The original process releases its instance, database and worker before the new Glow process starts. Original file-import arguments are not replayed during runtime recovery. The existing tab-continuity loader restores the saved reading context for this recovery launch, including when normal session restoration is disabled, without rewriting that preference. Safe mode retains its restoration restrictions. The original one-retry guard also applies to runtime faults.

## Executed evidence

Offline formatting, Clippy with warnings denied, the native application build and compilation of existing application targets passed. Four native synthetic profiles cover runtime recovery, startup fallback, explicitly selected Glow and ordinary wgpu reading/close. Artifact, executable and source hashes are recorded in [the capture manifest](../screenshots/rust-runtime-graphics/capture-manifest.json).

The runtime profile uses a file-backed synthetic store. After displaying its second page, the controller submits a favorite change and calls the actual wgpu `Device::destroy` API after signalling the fault monitor. The original instance closes. The alternate process reopens that same store and instance namespace, uses the real continuity loader and presents page index 1. Assertions require the queued favorite change, restored page and unchanged disabled `restoreLastSession` preference. Demo-store reuse is restricted to canonical paths inside the repository's isolated capture-store directory.

An additional owned process probe injects a failed checkpoint flag into the same device-destruction sequence. It exits with failure, produces no completed screenshot and emits no retry message. The fixed-field [checkpoint result](../screenshots/rust-runtime-graphics/checkpoint-failure.json) records its executable hash and outcome. This checks the guard; it does not simulate a physical disk-write failure.

## Remaining acceptance

The profile explicitly signals the fault before intentional device destruction. It does not exercise a physical driver reset, adapter removal or every platform path that reports `DeviceLostReason::Unknown`. Those scenarios remain required. Unsaved editor drafts, every in-flight tool/restore workflow, protected-document reopening and rapid context changes need further acceptance. Complete Glow memory accounting, alternate-compositor comparison parity and performance measurements remain pending. No personal library, PDF, password or raw driver diagnostic is published.
