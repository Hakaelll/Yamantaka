# Development checkpoint cleanup

Obsolete frozen verification executables were occupying most of the local
verification directory. This cleanup removes obsolete development artifacts;
it does not reduce the application package or change runtime memory behavior.

| Verification executables | Files | Logical bytes |
|---|---:|---:|
| Before cleanup | 48 | 2,709,147,136 |
| Removed | 46 | 2,594,315,776 |
| Retained | 2 | 114,831,360 |

The removed files total approximately 2.59 GB in decimal units. These are file
lengths, not a measurement of filesystem block allocation or a performance result.
The current `yamantaka-tags-ui.exe` and previous `yamantaka-tag-storage.exe` remain.
Their SHA-256 hashes are unchanged after cleanup. The running preview remains
responsive with the same native window and PID.

## Repeatable scope

`tool/prune_rust_checkpoints.ps1` defaults to inspection. `-Apply` removes only
matching ordinary files directly inside the workspace's `build/verification`
directory. It keeps the newest two executables and every inspected running
Yamantaka executable. The script rejects redirected workspace directories and
tracked/unverifiable content before removing anything. It checks each candidate's
parent, signature and metadata, and skips locked or changed files. Deletion uses
literal file paths and never recurses into a directory.

The tool does not clear Cargo dependencies, PDFium, Flutter baseline builds,
databases, reading/demo stores, cover decisions or source PDFs. Old executable
copies can be built again from their source checkpoints; exact byte-for-byte
reproduction of historical binaries is not claimed. Existing public screenshots
and their capture provenance remain intact.

## Executed verification

Inspection identified 46 candidates totaling 2,594,315,776 bytes. The authorized
apply run removed exactly those 46 files with no skips. A fresh inspection then
reported two scanned executables and zero candidates, showing convergence.
Retained executable hashes and the original private PDF hash remain unchanged.
No personal path, document, database or executable is included in this commit.

The audit also found an empty Cargo incremental directory. The retained Cargo
dependency directory contains 3,838,502,100 logical bytes. The demo/reading store
contains 1,005,704,921 logical bytes and is preserved as data, not treated as
executable cache. The normal installed native cover-cache directory is absent
on this host; preview data has a separate store. These are separate locations
and are not evidence of the final installed application's size.

No native rebuild or test-suite run is needed for this tooling/documentation
checkpoint. Package size, all other cleanup categories, complete performance
acceptance and the final Windows release remain pending.
