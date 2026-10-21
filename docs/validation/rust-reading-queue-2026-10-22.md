# Reading queue checkpoint

Tools → Reading queue opens a temporary native dialog for the current open readings. Readers can reorder entries, choose Pending, In review, Blocked, Reviewed or Archived, save a blocking reason and next action, open the first pending or in-review entry, or mark the active reading reviewed and advance. Notes are trimmed and bounded to 240 characters.

The existing atomic reader-continuity snapshot now stores each entry. Older snapshots default to Pending. Active-context updates, closed-context refresh and duplicated readings preserve queue metadata. Navigation uses the existing reader admission flow.

Validation is limited to the affected feature: a native build and one synthetic native capture with assertions for ordering, eligibility, context-refresh retention and actual file-backed queue round trip. No full test suite was repeated. The first capture exposed a comparison of private preference provenance rather than queue behavior; the assertion now checks persisted queue values and order directly.

Named saved workspaces, physical keyboard interaction and complete restart acceptance remain pending. This checkpoint covers the current reading context only. No personal documents or library data are included in its evidence.
