# Protected PDF import

## Connected behavior

File, folder and resumed imports now retain their current source guard and queue state when PDFium reports that a password is required. A separate bounded prompt channel carries at most one outstanding challenge per admitted import. The existing command/result reservations remain unchanged; a waiting challenge does not occupy an unreserved event slot or wait on the application actor.

The masked reader dialog now also handles import challenges. Reader documents and import requests are distinct enum variants. An incorrect credential returns a rejected-password challenge. A successful opening resumes the existing source-revision, metadata, confirmed-close, full-SHA-256 and repository checkpoint pipeline. The next document continues normally.

Credentials move through a one-shot bounded channel as the existing non-clonable, non-serializable zeroizing password type. They are never added to persisted queue state. Dropping a challenge cancels the import and wakes the actor. Closing the application cancels waiting challenges without requiring a UI reply. Asynchronous cancellation clears the password widget's focus and undo state. An interrupted native owner requests another credential on reopening and does not label a previously correct password as rejected.

The dialog reuses the canonical Yamantaka controls, palette, focus and masked-input handling. Its filename and actions stay outside the scrollable body. Compact spacing and initial field scrolling keep the password entry and fixed actions reachable with enlarged text. Incorrect-password feedback remains in the compact heading even when the body scrolls. Cancel explicitly identifies the import batch. No new visual system or dependency was introduced.

## Executed checks

Offline all-target workspace Clippy with warnings denied, formatting and the native build passed. One retained import-preemption test was extended; no standalone test case was added. Its real-PDFium helper checks a wrong password followed by success and a following ordinary PDF, catalogue queries while waiting, abandoned challenges, cancellation, shutdown and persistence without credential strings. A separate controlled source-revision handoff closes the actual native owner and reports its unavailability. The resumed import asks again and completes both files exactly once. The protected source bytes remain unchanged.

Five final native synthetic captures passed: the initial import prompt, rejected-password feedback at 640×480 and 200% application text, successful retry/import, cancellation and the existing compact reader password prompt. Import action rectangles must remain inside both their clip region and the viewport with bottom clearance. Capture metadata is checked for credential literals. Normal and compact captures were compared with their earlier versions: the compact footer now remains visible and the normal layout retains its proportions. No physical keyboard, DPI or Narrator acceptance is claimed. The executable, Cargo.lock and every manifest source hash were checked against the current tree. See the [capture manifest](../screenshots/rust-protected-import/capture-manifest.json).

## Remaining acceptance

Physical password entry, Narrator, actual DPI changes, multiple simultaneous protected batches, capacity-pressure reopening, protected-file corruption and hostile-input stress need broader acceptance. Persistent import recovery retains its ordinary file/folder intent and no credential; a complete protected restart matrix remains pending. Automatic covers, full-text indexing and other background reopening of closed encrypted PDFs still need explicit unlock coordination and acceptance. Resource accounting does not yet include every UI/password/channel allocation. This checkpoint does not certify the entire rewrite or Release readiness.

A later [metadata-resume correction](rust-protected-metadata-resume-2026-10-31.md) also handles native-owner loss during the front-matter scan. The earlier source-handoff scenario above did not cover that separate reopening path. The focused real-PDFium check now compares resumed bibliographic evidence with uninterrupted import and confirms the interrupted page is retried.
