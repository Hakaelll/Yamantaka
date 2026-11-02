//! Local-only acceptance runner. Raw library evidence stays beneath ignored build/verification.
use serde_json::{Value, json};
use std::{fs, path::Path, process::ExitCode};
use yamantaka_domain::{Cancellation, Failure, Result};
use yamantaka_storage::{
    Database, backup,
    migration::{self, LegacySource},
    preferences::Preferences,
};
use yamantaka_windows::{AppPaths, flutter_closed};

fn file_error(_: impl std::fmt::Display) -> Failure {
    Failure::Storage("Private verification file operation failed".into())
}
fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(Failure::Invalid(message.into()))
    }
}
fn verify(root: &Path) -> Result<Value> {
    require(
        flutter_closed()?,
        "Close Flutter before migration verification",
    )?;
    let executable = std::env::current_exe().map_err(file_error)?;
    let paths = AppPaths::discover(&executable)?;
    let source = LegacySource {
        database: paths.legacy_installed.join("yamantaka.sqlite3"),
        preferences: paths.legacy_installed.join("shared_preferences.json"),
    };
    let cancel = Cancellation::default();
    let original_database = yamantaka_storage::identity::full_sha256(&source.database, &cancel)?;
    let original_preferences =
        yamantaka_storage::identity::full_sha256(&source.preferences, &cancel)?;
    fs::write(
        root.join("source-before.json"),
        serde_json::to_vec(
            &json!({"databaseSha256":original_database,"preferencesSha256":original_preferences}),
        )
        .map_err(file_error)?,
    )
    .map_err(file_error)?;
    let review = migration::review(&source, flutter_closed, &cancel)?;
    let destination = root.join("captured");
    let receipt =
        migration::capture_reviewed(&review, &destination, flutter_closed, &cancel, |_| {})?;
    let verified = migration::verify_capture(&destination, &cancel)?;
    require(
        serde_json::to_value(&receipt).map_err(file_error)?
            == serde_json::to_value(&verified).map_err(file_error)?,
        "Migration receipt changed during verification",
    )?;
    let repeated =
        migration::capture_reviewed(&review, &destination, flutter_closed, &cancel, |_| {})?;
    require(
        serde_json::to_value(&receipt).map_err(file_error)?
            == serde_json::to_value(&repeated).map_err(file_error)?,
        "Repeated migration did not return the same receipt",
    )?;
    let db = Database::open_read_only(&destination.join("yamantaka.sqlite3"))?;
    let preferences = Preferences::load(&destination.join("preferences.json"))?;
    let export = root.join("export-v9.json");
    db.export_backup(&export, &preferences, &cancel)?;
    let restored = backup::preview(&export, &cancel)?;
    require(
        serde_json::to_value(&restored.preferences).map_err(file_error)?
            == serde_json::to_value(&preferences).map_err(file_error)?,
        "Backup preference namespaces changed on round trip",
    )?;
    let second = root.join("round-trip-v9.json");
    restored
        .database
        .export_backup(&second, &restored.preferences, &cancel)?;
    let payload: Value =
        serde_json::from_slice(&fs::read(&export).map_err(file_error)?).map_err(file_error)?;
    let round_trip: Value =
        serde_json::from_slice(&fs::read(&second).map_err(file_error)?).map_err(file_error)?;
    require(payload["version"] == 9, "Export did not use backup v9")?;
    require(
        payload["settings"] == round_trip["settings"],
        "Backup settings changed on round trip",
    )?;
    for table in backup::exported_tables() {
        require(
            payload["tables"][table] == round_trip["tables"][table],
            "Backup records changed on round trip",
        )?;
    }
    require(
        flutter_closed()?,
        "Flutter started during migration verification",
    )?;
    require(
        yamantaka_storage::identity::full_sha256(&source.database, &cancel)? == original_database,
        "Original Flutter database bytes changed",
    )?;
    require(
        yamantaka_storage::identity::full_sha256(&source.preferences, &cancel)?
            == original_preferences,
        "Original Flutter preferences bytes changed",
    )?;
    migration::verify_capture(&destination, &cancel)?;
    Ok(
        json!({"version":1,"source":"installed Flutter store","sourceSchema":receipt.source_schema,"targetSchema":receipt.target_schema,"reviewedCopyVerified":true,"repeatedCaptureIdempotent":true,"allBackupTablesRoundTrip":true,"exportedTableCount":backup::exported_tables().count(),"preferenceNamespacesRoundTrip":true,"backupSettingsRoundTrip":true,"originalDatabaseUnchanged":true,"originalPreferencesUnchanged":true,"nativeClosedChecksUsed":true,"activated":false,"privacy":"Only status and schema versions are public; raw copies, records, hashes and preferences remain local."}),
    )
}
fn main() -> ExitCode {
    let base =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/verification");
    let root = base.join(format!("installed-migration-{}", uuid::Uuid::new_v4()));
    if fs::create_dir_all(&base)
        .and_then(|_| fs::create_dir(&root))
        .is_err()
    {
        eprintln!("Could not create a fresh private verification directory.");
        return ExitCode::FAILURE;
    }
    let result = verify(&root).and_then(|mut report| {
        report["privateEvidenceId"] = root
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
            .into();
        fs::write(
            root.join("verification-summary.json"),
            serde_json::to_vec_pretty(&report).map_err(file_error)?,
        )
        .map_err(file_error)?;
        Ok(report)
    });
    match result {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            let _ = fs::write(root.join("private-error.txt"), format!("{error:?}"));
            eprintln!(
                "Installed migration verification failed; details remain in the private verification directory."
            );
            ExitCode::FAILURE
        }
    }
}
