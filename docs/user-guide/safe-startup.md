# Start in safe mode

Yamantaka enters safe mode after three consecutive starts whose previous run did not close cleanly. A clean application shutdown resets that counter. Rust keeps its own small startup record beside its independent library; it does not change Flutter's startup metadata.

To request safe mode manually, close the running application and launch:

```powershell
.\yamantaka-rust.exe --safe-mode
```

The title bar and Tools identify safe mode. The startup notice explains why it is active. Close that notice or choose **Continue** to use the library.

Safe mode skips restoration of saved tabs and workspaces, automatic PDF cover work and disk-cover-cache use, new indexing runs and nearby page preparation. It leaves existing index data and the index preference intact. Governed reading-session recording is suspended, and the previous saved reading context is not replaced. Library books retain their procedural 3D representation until a normal startup.

You can explicitly open a PDF, read, search existing indexed content, use Notes, edit library data, create a backup or review local diagnostics. This mode is not a read-only library and does not repair PDF files. Original PDFs remain unchanged. Manual file/folder imports and explicit tool operations remain available.

After a clean shutdown, launch normally without `--safe-mode` to return to normal startup. If the startup record is unreadable, oversized, unsupported or cannot be updated, Yamantaka preserves it and keeps safe mode active.

For a damaged or unsupported record, choose **Review startup record** in the safe-mode notice. Review does not change the file. **Preserve and recover** then retains that exact file as `rust-startup-preserved-<unique-id>.json` and creates a new startup record. Cancel leaves the record unchanged. If the file changes after review, review it again before recovering. The library, preferences and PDFs are not part of this operation.

After recovery, **Show preserved file** locates the old record in Explorer. Close and reopen Yamantaka normally to leave safe mode; the current run stays in safe mode. A valid record, insufficient permissions, a nonordinary file or a record larger than 8 MiB prevents repair and preserves the files. Correct folder permissions before retrying. Recovery never replaces an existing preserved file.

Blocked books can be reviewed through [Document recovery](document-recovery.md). If the selected library or its preferences cannot open, the separate [startup recovery window](startup-recovery.md) offers reviewed JSON restoration without starting PDFium. Safe mode does not bypass storage validation.

See the [validation checkpoint](../validation/rust-safe-startup-2026-10-31.md) for the exercised startup states and remaining recovery acceptance.
The [record-repair checkpoint](../validation/rust-startup-record-repair-2026-11-03.md) covers explicit preservation and the next normal startup.
