# Reference page text checkpoint

The reference navigator now offers Text. It displays the current page's extracted text, offers the existing Copy text action, and shares the primary reader's loading, empty-page and retry presentation. A single section selector keeps the popup compact as its tools grow.

Both readers use the same text request controller. A response must match document, page and cancellation identity. Reference responses are routed to their owner before primary text or selection handling. Changing the requested page cancels the previous request; closing the navigator, switching sections or closing the reference releases its text and cancels pending work. Extraction still runs through the existing serialized PDF worker and text resource leases.

## Evidence

The native binary compiled with `cargo build -p yamantaka --bin yamantaka-rust --offline`. Four focused native scenarios passed through `tool/capture_rust_ui.ps1`:

- `reader-reference-text`: both text views remain active for distinct documents. The reference changes pages while two superseded requests are cancelled, releases its text through navigator cleanup, then reloads page 2. Its actual payload contains `Mixed landscape page`; the main page-1 payload still contains `Yamantaka Rust`. Both views offer enabled copy controls.
- `reader-reference-text-small-200`: the reference reaches the same payload at a requested 640×480 window, 200% scaling, Spanish, light theme and reduced motion. The section selector and Copy text control are fully visible.
- `reader-reference-text-empty`: a scanned reference reaches a completed, empty-text state without an extraction error or copy control. The main reading stays on page 1.
- `reader-text-light-es`: the primary text panel still renders through the shared presentation.

The [native captures and manifest](../screenshots/rust-reference-text/capture-manifest.json) record source and executable hashes. Only synthetic fixtures were used. These scenarios validate extracted payloads and copy-control availability; they do not write to or inspect the operating system clipboard. No standalone Rust tests or broad workspace suite were added or run for this checkpoint.

Reference text selection and annotation tools, per-pane shortcut routing, overlays, full performance and physical accessibility acceptance, and Release packaging remain pending.
