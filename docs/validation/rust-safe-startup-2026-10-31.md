# Safe startup checkpoint

Date: 2026-10-31. Scope: native startup records, automatic/manual safe mode and suppression of optional startup work. This is not complete recovery or release acceptance.

## Connected behavior

Rust owns a versioned startup sidecar in its independent store. It records a dirty run before production database/preferences opening and resets the counter only after normal application return and restore handover, before releasing the instance lock. Three consecutive unclean prior runs activate safe mode, matching Flutter's threshold. `--safe-mode` requests it explicitly. Dropping a run or returning through an error does not mark it clean.

The record reader caps input at 4 KiB and validates its version and run token. Unknown, malformed, oversized or unavailable state is preserved and starts safe mode without replacing it. A clean exit verifies ownership before updating the record, so an older run cannot mark a newer run clean. Flutter metadata and the v11 schema are unchanged.

Safe mode skips saved-tab loading, workspace startup restoration, automatic library-cover work, disk-cover-cache use, reading-session observation/synchronization and continuity writes on shutdown. Index opt-in remains intact for existing search and index maintenance, but new indexing/rebuild runs and resume commands are blocked. Procedural book presentation remains available. Explicit PDF reading and library/tool operations remain available; the library is not read-only.

The native title bar, header and Tools identify safe mode. A dismissible scrollable notice explains the reason, suspended work and preserved saved session. It distinguishes an unavailable startup record from ordinary manual/repeated-failure startup. Local diagnostic reports now include the typed startup reason. No original PDF is modified.

## Executed evidence

The retained storage compatibility test `overrides_win_and_queries_are_bounded_and_escape_wildcards` passed with a startup helper: one passed, five filtered, 6.75 seconds. It verifies the three-failure threshold, clean reset, manual entry, stale-owner rejection and byte-for-byte preservation of unsupported, malformed and oversized records. No standalone test was added.

Offline native build and binary Clippy with warnings denied passed. Four synthetic native scenarios passed their state/control assertions:

| Case | Observed result |
| --- | --- |
| safe-requested | Explicit safe mode, visible Spanish reason, no cover slots, no recording, no continuity load and idle indexing |
| safe-repeated | A synthetic dirty predecessor with two failures produces the third-failure reason in the English light interface |
| safe-unavailable-small-200 | An unsupported synthetic record remains unavailable; the Spanish notice fits a requested 640 × 480 logical window with 200% text and reduced motion, with Close accessible |
| safe-reader | After synthetic notice dismissal, explicit native PDF reading installs a page while recording remains absent and optional work stays stopped |

All safe fixtures retain index opt-in and attempt a rebuild through the normal UI command handler. Their observed index phase must remain idle. The reader fixture explicitly enables session capture to exercise the real recording path. Its first run exposed a separate session synchronization path that still began recording; the safe-mode guard was added at that entry point before final verification.

The final dark startup notice and enlarged light unavailable-state notice were visually inspected. Native artifacts use isolated synthetic stores. Publication verifies source, Cargo.lock and executable hashes against the [capture manifest](../screenshots/rust-safe-startup/capture-manifest.json). Published PNGs embed origin metadata. No Computer Use was used.

## Remaining acceptance

Storage checks simulate unfinished runs by leaving records dirty; they do not establish actual process-kill recovery under native PDF load. Real installed saved-session preservation, crash timing during marker replacement, failed shutdown writes, real index-query acceptance and physical focus/Narrator/DPI remain pending. Safe mode still initializes PDFium and the selected renderer. It cannot bypass a failing database or unsupported preferences before the window opens. Automatic quarantine, manual record repair, complete startup recovery and the final Windows release remain required work. No broad workspace suite was repeated for this unit.
