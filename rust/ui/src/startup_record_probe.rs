use std::{fs, path::Path};
use yamantaka_domain::Cancellation;
use yamantaka_storage::{
    Database,
    identity::full_sha256,
    preferences::Preferences,
    startup::{StartupRun, repair},
};
use yamantaka_windows::retained_file::RetainedFile;

pub fn verify(report: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let parent = report.parent().ok_or("Missing verification directory")?;
    fs::create_dir_all(parent)?;
    let root = parent.join(format!("biblioteca-ñ-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root)?;
    let database = Database::open(&root.join("yamantaka.sqlite3"))?;
    database.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,title_override,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('synthetic-record','synthetic.pdf','synthetic.pdf','synthetic.pdf','Original title','Manual title',2,900,'2026-11-03','2026-11-03','2026-11-03','synthetic')")?;
    drop(database);
    Preferences::default().save(&root.join("preferences.json"))?;
    let token = Cancellation::default();
    let before = ["yamantaka.sqlite3", "preferences.json"]
        .map(|name| full_sha256(&root.join(name), &token).unwrap());
    let marker = root.join("rust-startup-v1.json");
    let original = b"{damaged synthetic startup record";
    fs::write(&marker, original)?;
    let lease = RetainedFile::open(&marker)?;
    assert!(fs::OpenOptions::new().write(true).open(&marker).is_err());
    assert!(fs::rename(&marker, root.join("external-rename.json")).is_err());
    fs::write(root.join("collision.json"), b"keep collision")?;
    assert!(lease.retain_as("collision.json".as_ref()).is_err());
    assert!(lease.retain_as("../escape.json".as_ref()).is_err());
    assert_eq!(fs::read(&marker)?, original);
    assert_eq!(fs::read(root.join("collision.json"))?, b"keep collision");
    drop(lease);
    let stale = repair::review(&root, &token).map_err(|e| format!("Initial review: {e}"))?;
    fs::write(&marker, b"changed synthetic startup record")?;
    assert!(repair::apply(&stale, &token).is_err());
    assert_eq!(fs::read(&marker)?, b"changed synthetic startup record");
    fs::write(&marker, original)?;
    let reviewed = repair::review(&root, &token)?;
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(repair::apply(&reviewed, &cancelled).is_err());
    assert_eq!(fs::read(&marker)?, original);
    let repaired =
        repair::apply(&reviewed, &token).map_err(|e| format!("Apply reviewed record: {e}"))?;
    assert_eq!(fs::read(&repaired.preserved)?, original);
    assert!(repair::apply(&reviewed, &token).is_err());
    let next = StartupRun::begin(&root, false);
    assert!(!next.mode().is_safe());
    next.finish()?;
    let valid = fs::read(&marker)?;
    assert!(repair::review(&root, &token).is_err());
    assert_eq!(fs::read(&marker)?, valid);
    let gap = root.join("retention-gap");
    fs::create_dir(&gap)?;
    fs::write(gap.join("rust-startup-v1.json"), original)?;
    let retained = RetainedFile::open(&gap.join("rust-startup-v1.json"))?;
    retained.retain_as("preserved-gap.json".as_ref())?;
    drop(retained);
    assert!(!gap.join("rust-startup-v1.json").exists());
    let resumed = StartupRun::begin(&gap, false);
    assert!(!resumed.mode().is_safe());
    resumed.finish()?;
    assert_eq!(fs::read(gap.join("preserved-gap.json"))?, original);
    let after = ["yamantaka.sqlite3", "preferences.json"]
        .map(|name| full_sha256(&root.join(name), &token).unwrap());
    assert_eq!(before, after);
    fs::write(
        report,
        serde_json::to_vec_pretty(&serde_json::json!({
            "synthetic":true,"unicodeParent":true,"writeAndDeleteLease":true,"existingDestinationPreserved":true,
            "staleReviewRejected":true,"cancelledApplyPreserved":true,"originalRecordPreserved":true,
            "repeatedApplyRejected":true,"validRecordPreserved":true,"retentionGapRecovered":true,
            "nextStartupNormal":true,"libraryAndPreferencesUnchanged":true
        }))?,
    )?;
    Ok(())
}
