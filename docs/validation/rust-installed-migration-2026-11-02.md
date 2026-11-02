# Installed-library migration and backup fidelity

## Executed source

A real installed Flutter v11 store was reviewed and captured into a fresh directory below ignored `build/verification`. Native process checks ran around review and capture. The installed source and installed Rust selection were not activated or replaced. Library records, source paths, raw hashes, preferences, screenshots and error details remain private.

The reusable runner is [verify_installed_migration.rs](../../rust/ui/examples/verify_installed_migration.rs). From the repository root, with Flutter closed, run:

```powershell
cargo run -p yamantaka --example verify_installed_migration --locked --offline
```

It creates a fresh private evidence directory. Its structured output identifies that local directory and reports status and schema versions. Keep the directory private: it contains actual library copies and JSON exports. Failed checks retain their private error record.

## Defect found and fixed

The installed-store check found that a v9 export/reimport discarded `documents.last_validated_at`. The exporter includes current SQLite columns, while restoration used Flutter's historical JSON column contract. That contract omits two existing v11 document fields: `is_modified` and `last_validated_at`.

Restoration now includes both optional fields. Existing values survive; older backups that omit them receive zero and NULL respectively. The historical required-column contracts remain unchanged, so earlier v1–v9 backups continue to load. This fixes data fidelity rather than normalizing or rewriting the installed source.

The retained populated-backup test now supplies non-default values for both optional fields in its v9 input, checks their exact exported values, and checks older-format defaults. Its existing full historical projections, populated relationships and settings checks still run. No standalone test was added.

## Evidence

[The redacted result](artifacts/installed-migration-2026-11-02.json) records:

- reviewed migration and receipt verification;
- unchanged repeated capture returning the same receipt;
- exact v9 round trip for all 23 exported categories and settings;
- unchanged original database and preference file hashes;
- a separate normal production startup on a portable copy, with the library query loaded, matching record count and clean startup state;
- unchanged installed Rust library selection.

The production startup used the current native executable and bundled PDFium in an isolated portable directory. Its screenshot and controls were kept private. The source's referenced PDF files were unavailable, so no original-PDF byte check was executed; that field is explicitly unverified in the public result. This run does not prove reader behavior for those documents.

The focused populated-backup test passed, along with workspace all-target Clippy with warnings denied, offline example/native builds and formatting checks. This change does not alter UI composition.

## Remaining acceptance

This verifies one installed v11 library, not every user's data. Installed historical database copies, real portable Flutter stores, source-start races and process-kill capture remain to be exercised. Known JSON categories are checked; this does not establish backup coverage for every auxiliary Rust file or unknown future SQL extension. Available-original PDF hashes, reader/corpus acceptance, physical accessibility, full resource/performance acceptance and the Windows Release remain pending.
