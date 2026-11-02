# Backup cover choices and native preferences

## Behavior

Native JSON export remains version 9 with SQLite schema 11. Two optional hashed categories preserve state that the historical JSON contract omitted:

- `tables.document_covers` contains manual cover-page decisions. Automatic detection results are recreatable and excluded. Small and large thumbnail paths are cleared on export and restore, so the saved choice requests fresh rendering rather than trusting a cache path from another machine.
- Root `rustPreferences` contains the versioned `legacy` and `rust` maps, preserving shadowed values and nested native settings. The existing flat `settings` field remains available, with Rust values taking precedence.

Restoration requires one manifest entry for each present extension, verifies its row count and hash, and rejects entries without payloads. Preference version, fields and agreement with flat settings are validated before installation. SQLite constraints reject invalid cover values and orphan references. Historical backups without either extension keep their previous behavior.

These extensions are backward-readable additions, not a promise that older Flutter clients restore the added state. Static inspection of the Flutter loader confirms unknown categories and root properties are accepted; an execution of that loader against this export was not performed in this checkpoint.

## Verification

The retained populated-backup test exercises production export, review, preparation, cover render-plan selection and re-export. It checks an exact manual cover row, automatic-cover exclusion, cache-path removal, exact preference namespaces including a shadowed setting, and old-format fallback. Eleven malformed variants cover hash tampering, an absent manifest, missing or duplicate manifest entries, invalid cover references and confidence, unsupported preference versions and contradictory settings. No standalone test was added.

Executed checks:

```powershell
cargo fmt --all --check
cargo test -p yamantaka-storage --test backups --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
```

The installed-library verification example now compares all 24 exported tables and the complete preference object. Its previous recorded run predates these extensions; updating the runner alone does not prove a new installed-library run.

## Limits

JSON backup does not include PDFs, image caches or every auxiliary Rust file. Existing historical document fields remain unchanged. This change has no UI composition changes and does not establish complete rewrite, physical accessibility, performance or Release acceptance. Whole-directory recovery remains documented in [Previous library](../user-guide/previous-library.md).
