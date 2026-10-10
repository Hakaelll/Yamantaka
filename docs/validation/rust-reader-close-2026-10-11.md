# Scoped native reader closure

The application actor now returns a document identity, request token and result for every reader-close request. This replaces the generic Close write acknowledgement; existing callers and the real protected-PDF regression use the typed reply. The domain tab controller can use these facts when its native/UI ownership adapter is connected.

## Admission and native ownership

A cancelled request or empty identity is rejected with its own scoped reply before native work. Accepted native closure remains independent of subsequent presentation cancellation. If the native worker completes the close successfully, the reply reports success even when its request token was cancelled later. A native error or worker disconnection is also reported against the original request. A failed close does not remove the actor's visible document identity.

Successful native admission cancels an older pending opening or unaccepted ownership offer for that same document. This prevents an old opening from later becoming visible after the close. A new opening for that document waits until all matching native close tickets resolve; unrelated document operations and repository work remain admissible. Only confirmed successful closure removes the identity from the actor's visible set. These rules retain the separate close/control admission reservation when all data-event reservations are occupied.

The request token identifies the presentation operation; it does not reverse a native close already admitted. Native handles stay in the PDF worker. This checkpoint does not yet connect a visible Close tab control, persisted closure snapshots or the twelve-tab selector.

## Verification

Three actor regressions cover pre-admission cancellation/invalid identity, unusual quoted/Unicode IDs, repository work while closure is held, cancellation after admission, revocation of an old opening, a deferred same-document replacement, native failure, explicit native cancellation, disconnected close tickets and preservation of visible ownership before another opening. The replacement test uses an actor query barrier after native admission rather than assuming that receiving the fake engine call means the actor has finished its transition. It confirms that the old opening cannot be adopted after its close and that the subsequent request retains its own live token.

The existing saturated-admission regression now checks the typed successful close reply alongside all pending writes. The protected-PDF integration regression receives a successful typed close from real PDFium and compares the original PDF bytes after worker shutdown.

All 384 locked workspace tests pass, including the migrated real-PDF closure test and the saturated close admission regression. Formatting and warning-denied workspace/all-target Clippy pass. Evidence remains in `build/verification/reader-close-tests.log`.

## Remaining acceptance

Connect native opening/closing facts to tab state without committing failed or superseded intentions. Add visible selection, closure/reopen, keyboard/context-menu commands, context synchronization and persisted recently-closed/workspace/queue adapters. Native rapid-switch, multi-tab ownership, physical interaction and final Release acceptance remain pending. No screenshot or UI tab parity is claimed by this actor checkpoint.
