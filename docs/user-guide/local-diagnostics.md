# Review local diagnostics

Open **Settings → Privacy → Review diagnostics**. Yamantaka collects a report in the background using an independent read-only database connection. The window remains responsive during collection. Refresh the report when you need a newer snapshot.

The report includes the application version, platform, architecture, renderer, database schema, logical database size, library record counts, a SQLite quick check, foreign-key checks, and governed memory usage and limits. Expand **View full report** to inspect the JSON before sharing it.

It excludes document paths, filenames, titles, authors, search queries, note text, selected passages, passwords and raw errors. It contains no document IDs or machine/user names. It does include aggregate counts and memory values. No report is sent anywhere automatically.

Choose **Copy report** to copy that snapshot, or **Save JSON** to choose a new file outside the library folder. Saving uses a temporary file followed by publication without replacing an existing destination. If the selected name already exists, choose another name. The PDF originals and library data remain unchanged.

**Cancel** or closing the review requests cancellation. An existing file is preserved. If publication has already completed, the saved complete report remains. A failed refresh retains the previous snapshot so you can still inspect it or try again.

The logical database size excludes WAL, backups and caches. Governed memory does not measure all process memory, opaque PDFium/driver allocations, UI atlas or every text/Glow upload. Database and memory snapshots are collected separately. A successful quick check is not a full recovery or migration certificate. Reports are temporary until you explicitly copy or save them.
