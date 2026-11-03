# Multiple pending operation records

Recovery previously required exactly one restore or rollback record. It now
reviews the complete known record set, binding every ordinary file to its
physical version and complete SHA-256. A changed set or changed library rejects
the old review. Opaque records are hashed in bounded chunks; their JSON and paths
are not interpreted by this recovery flow.

Before archiving multiple records, recovery writes and synchronizes a separate
review marker. It retains the exact held record handles without replacing any
destination. The marker remains until all reviewed operation records have been
archived. A restart during partial preservation, or with only the marker left,
returns to explicit recovery. Restore and rollback entry points also reject an
unresolved marker. The current or previous validated library is still chosen
explicitly, with complete current-folder retention when returning to a previous
copy.

The recovery screen now puts the chosen source and inline record counts before
the explanation, with compact spacing for enlarged text. Its latest presentation
has compiled but has not received a subsequent visual acceptance run.

## Recorded evidence

Seven native synthetic scenarios passed before that final presentation change:
single-record compatibility, two-record current and previous choices,
cancellation, partially archived records, a remaining marker alone and a corrupt
marker. The scenarios preserve raw record bytes, current and previous library
inventories, and a WAL-only document. They also cover records larger than the
former 64 KiB limit and prevent starting another restore or rollback while the
marker remains. A positive control confirms that the remaining record would
otherwise be automatically cleared.

All five retained backup tests passed, covering historical formats, populated
round trips, original-store retention, interruption, cancellation and invalid
input. All-target Clippy with warnings denied and the native build passed. No
Rust test functions were added. The [result artifact](artifacts/multiple-pending-records-2026-11-03.json)
separates the exercised backend from the later presentation change.

Raw scenarios stay local under `build/verification/multiple-pending-records-final`.
These runs simulate persisted archival boundaries; they do not certify power-loss
durability. The recorded scenarios cover an existing current directory and ordinary record
files. [Missing-directory recovery](../architecture/rust-missing-current-recovery.md)
was implemented afterward and has separate pending acceptance. Non-file records,
physical accessibility and full rewrite acceptance remain open.
