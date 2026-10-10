# Reader annotation export controls

The reader Notes panel now initiates complete-document Markdown and JSON exports through the independent application worker. Format controls use the existing navigation components, palette, hover feedback and keyboard focus. A native Windows save dialog selects the destination. Selecting or cancelling that dialog does not write a file.

The application request freezes the effective title, author, current reader progress and a UTC export time. The worker rejects a file extension that does not match the selected format before streaming output. Export includes every persisted annotation rather than the current 80-row list or visible page.

The panel presents pending and cancelling states, a cancellation action, exported record count, cancelled output, occupied-destination recovery and recoverable failure details in ES/EN. It keeps the request identity across document changes and labels results for a previous document. A stale reply cannot finish another request. A successful publication remains a success if cancellation arrives before its terminal reply is received. Starting a retry clears the previous outcome.

An active export keeps its cancellation controls available when the note editor is open. Application exit cancels the export before releasing reader resources; the runtime then joins its worker. The UI invokes neither SQL nor PDFium to export notes.

## Verification

Two UI controller regressions check document/token matching, stale replies, retry state, cancellation and a success that was already published when cancellation arrived. The existing SQLite-backed runtime test now also rejects a JSON request with a Markdown extension without creating output.

The synthetic native capture path now uses a per-process file-backed library for export cases, exercising the production independent-connection path. Synthetic annotations use complete UTC timestamps. The capture manifest includes each published file's name, hash and record count. Its compact readiness gate does not require marks for a PDF hidden by the panel drawer; visible PDFs still require complete loaded mark snapshots.

Four final native wgpu captures were inspected:

- Dark English Notes with a successful 181-record JSON export.
- Light Spanish Notes with a successful 181-record Markdown export.
- The 640 × 480 window with text at 200%, with a successful 181-record JSON export. The initial drawer only shows its heading; this does not prove scrolling, keyboard access to the export controls or physical monitor DPI acceptance.
- Dark Library, retaining the physical book covers and catalogue composition.

Both JSON outputs parse with all 181 IDs, including the final ID, and UTC creation/update timestamps. Markdown contains all 181 notes and one anchor for the represented page. The three published output hashes match their manifest entries. Executable, Cargo.lock, UI-source, screenshot and control-file hashes also match the final manifest.

Local capture directory: `build/rust-visual/annotation-export-native`.
Manifest SHA-256: `1A95FD6A11458C51C89A92B6CEE155A3FDFEE031D105EF1A2C2100AB889AB72E`.
Separate preview executable: `build/verification/yamantaka-annotation-export-preview.exe`.

Formatting, warning-denied workspace/all-target Clippy and the full locked workspace tests pass. The preview build's two expected output-path warnings arise from the explicit alternative executable output; the separate Clippy run is warning-free. Test log: `build/verification/annotation-export-controls-tests.log`.

## Remaining scope

The save-dialog call is connected, but actual native dialog interaction, cancellation through that dialog, keyboard/scrolling acceptance, Narrator and monitor DPI acceptance have not been exercised. Output publication currently requires a new file. Explicit replacement confirmation/policy, live per-record progress, a single transactional metadata/annotation snapshot, crash cleanup, SQLite lock contention, large-export/intra-row cancellation and full resource accounting remain pending. Synthetic success screenshots do not prove the native error and cancellation presentations. Global annotation review and broader annotation/selection parity also remain incomplete. No final release parity is claimed.
