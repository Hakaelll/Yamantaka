# Reading history and A/B positions checkpoint

Both reading navigators now expose Positions. They share the existing main reading menu through typed position actions rather than a second history implementation. Back, Forward and temporary A/B marks operate on the owning ReaderNavigation; callers retain their canonical render, persistence and comparison synchronization bookkeeping.

Alt+Left and Alt+Right traverse the active pane's history. Input controls retain their keys, modal guards remain in place, and history arrows suppress egui's simultaneous spatial focus movement. Reference admission still consumes reading commands while busy. A deliberate navigation after going back replaces the forward branch through the existing domain controller.

The main navigator's Positions section is mapped through tab context, session restoration, continuity JSON and saved workspace adapters. It uses the existing string fields and schema v11. Temporary marks and in-memory history are not persisted. Document bookmarks remain a separate saved feature.

The main navigator uses one section selector and a close control instead of wrapping its sections over multiple tab rows. The compact view uses a history row and one row per A/B mark. Each save icon has a localized accessible label and tooltip. Reference menu height depends on viewport height rather than its cursor, removing a feedback loop between popup placement and scroll height. The 200% case retains scrolling for controls beyond the visible area.

## Evidence

The native binary compiled offline and formatting passed. Four focused scenarios ran through tool/capture_rust_ui.ps1:

- reader-reference-history swaps the panes, runs the existing input sequence, and traverses reference history with actual Alt-arrow events. It saves A on reference page 1 and B on page 2, returns to each, and verifies the main reading's separate A on page 2 and empty B. Main history then traverses backward and forward while preserving the reference. New page navigation in each reading clears its forward branch. The reference Positions popup is captured.
- reader-reference-history-small-200 runs the same behavior in Spanish, light theme, reduced motion, a requested 640×480 window and 200% scaling. Recorded bounds require the Positions selector and Save position A to be fully visible.
- reader-primary-history opens the main Positions panel in English after the sequence. Its save controls remain visible alongside an independent reference.
- reader-reference-shortcuts-close retains the earlier focused-reference close path and exactly one scoped native close acknowledgement.

The native preview injects keyboard and pointer input through egui's raw-input hook. A/B save and jump assertions invoke the same typed actions returned by the UI. The check establishes controller ownership and visible controls; it does not claim physical pointer or Narrator acceptance. Only synthetic PDFs were used. No Computer Use was used.

One retained storage compatibility test was extended and passed: all_eleven_historical_schemas_preserve_ids_relationships_and_preferences. Its workspace sequence now round-trips the Positions section through SQLite snapshots, deletion undo and continuity JSON, and checks session-context decoding. No standalone test or broad workspace suite was added.

The [capture manifest](../screenshots/rust-reference-history/capture-manifest.json) records the native executable and source hashes. Reference selection and annotations, shortcut customization, overlays, full history parity, physical accessibility, performance acceptance and Release packaging remain pending.
