# External PDF link policy and confirmation

Date: 2026-10-09

## Connected behavior

Page-link payloads now use a typed internal/external target rather than assuming every link has a page destination. Internal links keep the canonical navigation behavior. URI actions use the bounded native getter before allocation, then a validated ExternalUri contract.

The allowed schemes match Flutter's local input policy: http, https and mailto. HTTP destinations require an explicit nonempty authority and parsed host. Relative paths, local files, executable/custom protocols, control characters, empty hosts and slash/backslash forms repaired by the URL parser are rejected. Both input and normalized output must fit 4,096 UTF-8 bytes. Normalization uses the already locked url 2.5.8 dependency. Debug output is redacted and errors omit URI text.

URI string capacity contributes to the shared page Text lease. The native request limits aggregate retained URI capacity to 512 KiB, within its existing 2 MiB working reservation. Invalid, missing and overlong URI actions are counted as unsupported without opening anything; supported internal links on the page remain available. Rejected external regions now retain a typed reason and expose explanatory feedback without retaining or opening the rejected URI.

Activating an allowed external region opens a modal that displays the normalized destination. Cancel receives initial focus. The explicit Open action passes the validated URI as the Windows shell target, with no command parameters or executable interpolation. Escape or dismiss closes the modal. Leaving the source page, changing documents, leaving the reader or exiting drops the confirmation and its retained payload. Failed Windows opening reports a generic error and permits retry.

Synthetic native captures never dispatch to another application. URI values are not added to capture metadata, logs, repository data or backups. The normal application only invokes Windows after the user's explicit Open action.

## Executed verification

Two URI policy tests cover allowed schemes, rejected protocols and authority forms, control characters, exact byte limits, normalization growth and redacted diagnostics. One payload test verifies larger URI content increases tracked memory and the final shared owner releases the lease. One UI lifecycle test verifies a pending confirmation retains its page lease and releases it when the page leaves.

A real PDFium test covers http, https, mailto, local files, scripting actions, empty HTTP host and excessive URI length. It checks returned target counts, retained memory, lease release and original PDF SHA-256 preservation. No test opens an external application.

Workspace tests, warning-denied Clippy and formatting checks passed. Local logs are `build/verification/external-pdf-links-tests.log` and `build/verification/external-pdf-links-clippy.log`.

## Remaining acceptance

Four native dark/light/Spanish/compact captures now verify confirmation presentation and rejected-target feedback (see [the native report](rust-external-links-native-2026-10-09.md)). Actual Windows browser/mail-handler opening and failure/retry need explicit acceptance. A real aggregate URI-budget regression now proves partial memory release and worker recovery; rapid modal/document changes and broader interaction acceptance remain pending. Existing internal-link captures predate this target-contract change and are historical evidence. No complete rewrite, release or full external-link acceptance is claimed.
