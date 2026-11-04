# Protected PDFs in automatic cover batches

The cover-regeneration coordinator retains each planned source and cover snapshot
while its ordinary native cover job runs. A password-required result now switches
that item to the existing explicit cache-rebuild job. The fallback starts only
after source and saved-selection checks; ordinary books retain their existing
native path.

The rebuild request can carry the batch's frozen automatic-reset snapshot.
That snapshot must still match before the hash worker starts. This mode forces
automatic detection and uses the guarded automatic replacement transaction,
including when the reviewed original cover was manual. Individual cache repair
keeps its earlier behavior and saved page.

The fallback reuses full-content hashing, transient password channels, native
source checks, cached-image readback and confirmed temporary-document closure.
The coordinator registers those readiness sources with the actor and advances
the PDF-capacity generation after a confirmed native release. It does not open
a second reader or store passwords in batch intent.

A distinct password purpose gives the same ES/EN dialog an explicit Cancel batch
action. It cancels the shared batch token; completed items remain successful,
while unfinished items remain in the report. A changed saved choice fails its
item rather than replacing the user's newer decision.

Protected batch items now wait when native opening reports that the document
slots are busy. Each open ticket retains its admission-capacity generation, so
a release that arrives before the busy reply is consumed still permits a retry.
The waiting state registers no repeated native request and resumes only after
that generation changes or cancellation is requested. Before reopening, it
rechecks the frozen source and selection while retaining the completed full hash.
Passwords are not held across that wait; another unlock may be required.

The batch progress exposes this capacity wait through its existing status.
Individual cache-repair requests keep their earlier busy-slot behavior. Cache
pressure can still prevent an item from completing. Runtime cancellation,
foreground-preemption and capacity-pressure acceptance remain pending.

This is development work after published review 10. No new test cases were added.
Existing fixture constructors and calls were adapted to the new request/context
fields. The locked, offline native development build (`cargo build -p yamantaka`)
completed successfully. This confirms compilation, not runtime behavior.
No application tests or captures were run for this change. The interactive
review-10 executable and its user data were left unchanged.
