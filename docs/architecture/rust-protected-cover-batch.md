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
item rather than replacing the user's newer decision. Native slot or cache
pressure can still prevent a protected item from completing; cancellation and
foreground-preemption acceptance remain pending.

This is development work after published review 10. No new test cases were added.
Existing fixture constructors and calls were adapted to the new request/context
fields. The locked, offline native development build (`cargo build -p yamantaka`)
completed successfully. This confirms compilation, not runtime behavior.
No application tests or captures were run for this change. The interactive
review-10 executable and its user data were left unchanged.
