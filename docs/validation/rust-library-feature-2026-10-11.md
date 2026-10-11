# Library reading feature and continuity integration

Date: 2026-10-11. This is an incremental checkpoint. The user has prioritized visual design before the remaining functional rewrite. The final Release and complete parity acceptance remain pending.

## Visual change

The library now uses its existing Source Serif display face for the full-size library heading. The continue-reading area presents a larger physical book, an editorial document title, its author, page and progress, and a clear continuation action. Its existing book mesh, rigid pointer physics, reduced-motion behavior and semantic controls are reused. No replacement logo, online asset, new font download or autonomous ambient animation is introduced.

The feature has a smaller cover when the inspector is open and moves its action below the text in narrower layouts. Compact windows retain the existing compact composition. Native inspection found an alignment problem and then a layout defect that stretched the feature across the viewport. The final code aligns content from the top and uses a compact vertical action container. Verification also checks that the continuation action remains in the upper half of the wide viewport so the feature cannot hide the catalogue again.

The feature is extracted into a presentation module instead of adding another large branch to the application shell. The visual direction preserves graphite/violet and warm ivory, and uses document content rather than decorative metrics or invented activity.

| Before | After | Why |
| --- | --- | --- |
| Small continuation thumbnail and a uniform heading | Larger 3D reading feature and editorial library heading | Give the current reading a clear place in the library |
| Continuation content shares a compact inline row | Cover, title, progress and action have deliberate spacing | Make the reading context easier to recognize |
| One continuation layout while details are visible | Smaller cover with inspector, action below text on narrow layouts | Protect the available catalogue space |

## Functional work already in progress

The library repository now connects the independent JSON continuity adapter to a sidecar named after its SQLite file. Separate libraries in the same directory have separate sidecars. In-memory stores do not persist continuity.

Scoped actor requests load and save snapshots. A monotonic revision fence prevents an older queued write from replacing a newer shutdown snapshot when the priority control channel overtakes data commands. Accepted writes retain their terminal success after presentation cancellation. Cancelled loads do not return data.

The application loads continuity outside demonstration mode, observes `restoreLastSession`, restores the active reading through the existing scoped tab-opening flow, and periodically submits changed snapshots. Its close handler uses the persistence barrier independently of UI event capacity. Failed initial loads preserve the file and prevent automatic replacement. If another reading is already active while an existing snapshot loads, the prior snapshot is preserved and the conflict is reported; explicit conflict recovery remains pending.

Demonstration captures intentionally do not exercise personal continuity files. These captures prove the presentation only. They do not prove successful or interrupted real-library restarts, recovery prompts, safe mode, missing/modified files, passwords or physical interaction acceptance.

## Verification

Three additional regressions cover per-library sidecar isolation, accepted-write cancellation/scoped loads, and a saturated UI queue with priority persistence and stale-write fencing. The previously verified storage regressions continue to cover full reader context, extension fields, invalid data and size limits.

The locked workspace passed 407 tests, including the three new regressions. After the final presentation-only layout fix, all 93 UI tests passed again. Workspace/all-target Clippy passed with warnings denied. Formatting was applied and checked separately. The unique verification executable was built with two expected output-path warnings; Clippy was clean.

Eight native captures cover the dark/light library, Spanish, details, a synthetic pointer tilt, reduced motion, 640×480 with 200% text and reader tabs. The final executable SHA-256 is `2917B6CA1868777036BEAD06640EF0AED50D6963EBAD2B713150395A19B33B99`. Independent verification matched all 61 UI source files and their complete inventory, Cargo.lock, the executable, eight PNG/control file pairs and eight successful process exits. It also checked that the continuation action is enabled, inside its clip and in the upper half of every full-size library capture. Dark/light/details images were visually inspected after the final fix; the catalogue remained visible.

Original final evidence is in the ignored `build/rust-visual/library-feature-verified` directory. The public `docs/screenshots/rust-reading-feature` directory includes all eight synthetic images, control snapshots, provenance text and the source/capture manifest. Provenance is embedded as PNG text without replacing screenshot pixels. The README uses the updated dark library capture. The mechanical design detector returned no findings, but it does not certify native Rust accessibility or layout. Its context loader misclassified the Windows-only project because of historical Flutter files and an unsupported platform label; the explicit Windows specification remained the design authority.

## Remaining visual work

Continue the visual pass through gallery composition and material finish, reader controls and transitions, the document inspector, tools, settings, empty/loading/error states and responsive variants. Motion must remain interruptible, finite and optional; the static 3D representation remains available. Real keyboard, Narrator, physical DPI, multiple monitors and performance acceptance are still required. The interface is not declared finished by this checkpoint.
