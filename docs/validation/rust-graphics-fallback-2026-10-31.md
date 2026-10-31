# Graphics startup fallback checkpoint

Date: 2026-10-31. Scope: a bounded fresh-process retry after a typed eframe wgpu initialization error.

## Behavior

The top-level application entry point classifies the original error type. Only `eframe::Error::Wgpu` is eligible for automatic fallback. Explicit `--software`, `--opengl` and the internal retry marker prevent another attempt. Ordinary application, storage and OpenGL errors retain their existing failure path.

The failed `run()` returns before the child process starts, releasing its instance guard, database and native worker through their existing owners. The child receives the original `OsString` arguments plus `--software` and the retry marker. The parent waits for its exit and reports success only if the alternate process succeeds. No shell concatenation, preference rewrite, background monitor or recurring task is involved. Startup history is not reset merely because a compositor failed to initialize.

## Executed evidence

Offline formatting, Clippy with warnings denied and the native application build passed. Three native synthetic capture profiles exercise automatic library fallback, automatic reader fallback and an explicitly selected alternate compositor. The injected failure uses the actual egui-wgpu native adapter-selector error path, before application creation. The successful retry starts a new native Glow window, displays the synthetic library or PDF and produces the screenshot through the existing capture controller. The output path contains a space, exercising argument preservation across the child launch.

Capture assertions require the Glow presenter, the expected automatic-retry marker and a visible page slot for reader cases. [The capture manifest](../screenshots/rust-graphics-fallback/capture-manifest.json) records artifacts and source/executable hashes.

Two additional owned process probes inject an error from the application-creation callback. That error alone exits with failure and zero retry messages. Combined with the adapter-selector failure, it exits with failure after exactly one retry. Neither produces a successful screenshot. The fixed-field [probe result](../screenshots/rust-graphics-fallback/failure-results.json) records exit codes, retry counts and the executable hash without raw diagnostic logs.

## Limits

This checkpoint does not prove runtime device-loss recovery or physical adapter/driver failure. Actual installed-store instance handoff, active-session reconstruction after device loss, complete Glow memory accounting and comparison parity in the alternate compositor remain pending. The synthetic captures use separate QA stores; no personal library was opened or published. Physical DPI, Narrator, multiple monitors and performance acceptance were not exercised here.
