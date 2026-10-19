# Native collection management

Tools → Collections opens a dedicated, scrollable manager. It supports creation,
search, bounded paging, pinning, renaming and collection deletion. Each document's
details also exposes Organize in collections, which opens the same manager with
membership checkboxes for that document. Saved IDs and schema-v11 relationships
use the [collection storage contracts](rust-collections-storage-2026-10-19.md).

## Commands and state

The UI emits typed collection read/write actions through the application actor.
It executes no SQL and makes no PDFium call for organization. Queries return at
most 80 rows, including counts and optional membership for one scoped document.
A newer read cancels the previous request; only its matching token can replace
the displayed page. Changing context clears old rows. Matching terminal write
results are accepted even after leaving the manager, so a committed change can
still refresh the catalogue and active filter. Shutdown cancels pending work.

Create and edit controls are disabled while a write is pending. Failed writes
retain drafts and a local error; refreshing the page does not erase that error.
Deleted or renamed collections update the active filter after commit. Collection
deletion requires an inline review with explicit Delete collection and Keep
collection actions. The review states that books and PDFs remain intact.

View books applies the selected collection through the existing repository
predicate, combined with text, favorites and reading/file-state filters. Its
name and clear control appear in the existing filter row; Reset filters and
All books also remove it. No permanent sidebar or additional full-width bar is
introduced. Compact library layouts keep these controls in the existing filter
menu. Book imagery, 3D physics and reader presentation remain unchanged.

The active collection ID and label are stored in versioned atomic preferences.
Startup queries that exact ID, without restricting the lookup to the first page
of collections. A successful lookup replaces a stale label with its current name;
a missing collection clears the filter. An inspection error preserves the saved
selection rather than deleting it. A failed submission leaves it available for
manual clearing or selection through the manager.

## Executed verification

The retained bounded-query storage test passes after adding exact lookup checks
for a collection on the second page and for an absent ID. No test case is added;
the reduced Rust suite remains at 54 cases. Cargo formatting, workspace all-target
Clippy with warnings denied and the offline native development build pass.

[Four native synthetic captures](../screenshots/rust-collections-ui) drive the
normal actor commands to create Reading seminar, assign demo-0 and pin it.
The manager reports one collection and one member. The filter scenario saves
an isolated preferences file with an outdated label, loads it through the normal
preferences decoder, and invokes the same ID verification as startup. The final
filter shows the current name and exactly one library document. This is a
preferences round trip in one process, not a full application restart test.

Other captures show the deletion review and a small viewport with 200% text.
The latter scrolls the real manager to its membership checkbox, View books and
actions menu. Dark, Spanish light and reduced motion are represented. Source and
executable hashes match the capture manifest; all fixtures and public screenshots
are synthetic. No personal store or PDF is published. These scenarios invoke
the application's control methods; physical mouse/keyboard/Narrator operation
remains an acceptance task.

## Remaining parity

Collection reorder, icon editing, multi-document membership operations, tags,
smart views, collapsible grouping and broader organization workflows remain
incomplete. Full restart/crash acceptance, permission faults, large-catalogue
latency, physical DPI/accessibility and paired Flutter/Rust measurements remain
pending. This checkpoint does not complete the rewrite or release requirements.
