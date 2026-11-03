# Reviewed startup-record recovery

A damaged or unsupported startup marker previously kept Yamantaka in safe mode without an in-app recovery path. The safe-mode notice now offers review, cancellation and explicit preservation before creating a new marker. The current run stays in safe mode. A normal subsequent startup can leave it.

Storage holds the reviewed file against writing and deletion, checks its Windows identity/version and complete SHA-256 again at confirmation, and renames that exact open file without replacing a destination. A replacement is prepared and flushed before preservation, then published without clobbering an existing active marker. Cancellation is honored before preservation; publication completes after that boundary. Only the derived startup marker is involved.

The Windows adapter uses `NtSetInformationFile` with a simple filename and null root, which retains the source's actual parent directory. An initial `SetFileInformationByHandle` implementation with a relative name and directory handle returned invalid parameter on this host and was replaced. The native probe exercises the final implementation, including destination collision and traversal rejection. The call and structure follow Microsoft's [NtSetInformationFile](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ntsetinformationfile) and [FILE_RENAME_INFORMATION](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information) documentation. No new crate or standalone unit test was added; two features of the existing pinned Windows crate expose those bindings.

Executed evidence:

- A native storage probe in a synthetic Unicode directory verifies denied competing writes/renames, collision preservation, stale-review rejection, cancellation, byte-identical preserved content, rejection of repeated application and protection of a valid marker.
- The same probe reconstructs the gap between preservation and publication. Startup recreates the missing derived marker, while the preserved file remains intact. This reconstructs a filesystem state; it is not an actual process-kill test.
- A real SQLite v11 library with a manual title override and a preferences file have identical SHA-256 before and after recovery.
- Four native captures exercise review, successful application, changed-file rejection and cancellation. Each reports its expected state. Successful application verifies the next normal startup through the real startup adapter; the running UI still reports `stateUnavailable`.
- The dark English review and light Spanish success captures have fully visible recovery actions and Continue. At 640×480 with 200% text, the header and Continue remain visible while the body scrolls. Recovery actions precede the longer safe-mode explanation. The previous executable was captured under the same 1024×960 dark conditions for comparison.
- Offline build, workspace formatting and all-target workspace Clippy with warnings denied pass. Captures were reviewed as actual raster images. No personal documents were used.

The [status artifact](artifacts/startup-record-repair.json) records the executable hash and selected native results. Images and synthetic stores remain local under `build/verification/startup-record`. Reproduce the storage assertions with `yamantaka-rust.exe --demo --verify-startup-record-repair=<report.json>`; native UI fixtures use `--demo --safe-unknown --startup-record-repair=review|apply|stale|cancel --capture=<image.png>`.

Physical keyboard/Narrator interaction, process-kill/power-loss acceptance, other filesystem configurations and ambiguous restore-journal recovery remain pending. This checkpoint does not establish full rewrite or Release readiness.
