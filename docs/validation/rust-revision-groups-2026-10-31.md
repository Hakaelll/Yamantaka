# Revision group checkpoint

Document connections now includes a Revision groups section. It reads inherited groups, creates a named group from the current document and a chosen edition, opens its members, and changes its current edition. The creation form stays behind New group. Wide windows place group selection and edition details in two columns; narrow windows stack them. The selected group and member snapshots remain independent of the PDF reader.

The implementation keeps the existing v11 document_revision_groups and document_revision_members tables. Group identity, name, current-document identity, membership, creation time, and inherited revision timestamps remain unchanged during reads. Group and member queries each use their own SQLite read transaction and return at most 80 rows with a total and offset. Members load only when a group is opened. Immutable Arc snapshots avoid cloning record lists per frame, and an existing member snapshot stays visible until its replacement arrives.

Creation validates a trimmed name of 1 to 120 Unicode scalar values, 2 to 80 distinct catalogue members, and an optional current member. One immediate transaction inserts the group and all memberships, with cancellation before commit. The UI creates two-member groups, matching the connected Flutter creation form; inherited larger groups remain readable through member pagination.

Changing the current edition validates membership and the expected group timestamp and current-document identity. The latter also detects an automatic foreign-key clear whose timestamp did not change. A successful update preserves the creation time and advances a monotonic revision timestamp. Scoped cancellation tokens reject stale list/member/write replies. Context and section changes cancel reads while accepted writes retain their application state until acknowledgment.

## Evidence

The retained storage compatibility test was extended through support/revisions.rs. Its focused run passed. It checks name and membership validation, duplicate-member normalization, cancelled creation, a failure on the second member and rollback, inherited member identity, document title overrides, visibility from both members, current-edition changes, stale-version rejection, failed update rollback, automatic current clearing after document deletion, rejection of the old snapshot after that clear, an 80-member group, the next member page, rejection of an 81-member create request, and SQLite integrity. No standalone tests were added.

The final application binary compiled offline. cargo fmt --all --check and Clippy for the binary and dependencies passed with warnings denied. Four native scenarios completed through tool/capture_rust_ui.ps1:

- revision-groups-legacy opens a synthetic inherited group with two members and demo-1 as current. Its original ID, membership, and current choice survive the native reads. The capture uses Spanish and dark theme.
- revision-groups-created retains that inherited group, creates Study editions with two members, and selects it after the write and list/member refresh. The source demo-0 is current. The English light-theme capture records the collapsed creation form and two-column group/detail layout.
- revision-groups-current-small-200 creates the same group, changes its current edition to demo-1, and verifies the refreshed group and member snapshots agree. Spanish, light theme, reduced motion, a requested 640 by 480 window, and 200% scaling exercise the stacked layout. The capture scrolls to an enabled Make current action for the other member and checks its bounds.
- relations-backlinks retains the prior directional relation creation, duplicate admission, removal, and incoming-context workflow after adding the section switch.

The [capture manifest](../screenshots/rust-revision-groups/capture-manifest.json) records executable, Rust sources, image, and control hashes. They were checked against the final binary and working tree before publication. The native sequences set values and invoke controller actions programmatically, while writes and reads use the real application actor and SQLite. They do not certify physical input, Narrator, or a real Flutter installation. No Computer Use or personal library data were used. No broad workspace suite was run.

## Remaining acceptance

Group rename, deletion, and membership editing are not part of this flow. Additional evidence is needed for the physical chooser and creation form, opening an edition from its row, full restart, rapid group/context changes, close during an accepted write, externally changing groups, multi-page group lists and inherited groups exceeding 80 members, memory pressure, physical DPI, and Narrator. Automation rules, journals, undo, full performance comparison, and Release packaging remain pending. This checkpoint does not establish complete advanced-organization parity or rewrite acceptance.
