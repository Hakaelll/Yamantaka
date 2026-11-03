//! Explicit synthetic acceptance; never opens the installed library.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use yamantaka_domain::Cancellation;
use yamantaka_storage::{
    Database, backup,
    identity::full_sha256,
    managed_imports::{self, repair},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone)]
pub(crate) struct Fixture {
    pub root: PathBuf,
    pub imported: PathBuf,
    pub pointer: Vec<u8>,
}
pub(crate) fn fixture(parent: &Path, pointer: Vec<u8>) -> Result<Fixture> {
    fs::create_dir_all(parent)?;
    let root = parent.join(format!("selection-ñ-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root)?;
    let root = root.canonicalize()?;
    fs::write(
        root.join(".synthetic-selection-probe"),
        b"Yamantaka synthetic selection acceptance",
    )?;
    let imports = root.join("imports");
    fs::create_dir(&imports)?;
    let imported = imports.join(uuid::Uuid::new_v4().to_string());
    fs::create_dir(&imported)?;
    for folder in [&root, &imported] {
        let preview = backup::preview_payload(
            serde_json::from_str(include_str!(
                "../../storage/tests/fixtures/populated-backup-v9.json"
            ))?,
            &Cancellation::default(),
        )?;
        preview
            .database
            .snapshot(&folder.join("yamantaka.sqlite3"))?;
        preview.preferences.save(&folder.join("preferences.json"))?;
    }
    let db = Database::open(&imported.join("yamantaka.sqlite3"))?;
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('imported-only','synthetic-imported.pdf','synthetic-imported.pdf','synthetic-imported.pdf','Imported only title',2,900,'2026-11-03','2026-11-03','2026-11-03','imported-synthetic')")?;
    drop(db);
    fs::write(
        imported.join("migration-receipt.json"),
        b"{\"synthetic\":true}",
    )?;
    fs::write(
        root.join("original-resource"),
        b"keep complete original library",
    )?;
    fs::write(
        imported.join("imported-resource"),
        b"keep complete imported library",
    )?;
    fs::write(root.join("selected-library.json"), &pointer)?;
    Ok(Fixture {
        root,
        imported,
        pointer,
    })
}
pub(crate) fn hashes(root: &Path) -> Result<BTreeMap<String, String>> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                visit(root, &path, out)?;
            } else {
                let name = path.file_name().unwrap().to_string_lossy();
                if name == "selected-library.json"
                    || name == "selected-library-repair-v1.json"
                    || name == ".interrupted-stage"
                    || name.starts_with("yamantaka-selection-")
                {
                    continue;
                }
                out.insert(
                    path.strip_prefix(root)?.to_string_lossy().into_owned(),
                    full_sha256(&path, &Cancellation::default())?,
                );
            }
        }
        Ok(())
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result)?;
    Ok(result)
}
pub(crate) fn verify_choice(
    f: &Fixture,
    imported: bool,
    before: &BTreeMap<String, String>,
) -> Result<()> {
    let expected = if imported { &f.imported } else { &f.root };
    assert_eq!(&managed_imports::recover_and_select(&f.root)?, expected);
    assert!(!repair::recover(&f.root)?);
    let archived: Vec<_> = fs::read_dir(&f.root)?
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("yamantaka-selection-preserved-")
        })
        .collect();
    assert_eq!(archived.len(), 1);
    assert_eq!(fs::read(archived[0].path())?, f.pointer);
    assert!(!repair::pending(&f.root)?);
    assert!(
        !fs::read_dir(&f.root)?
            .filter_map(std::result::Result::ok)
            .any(|e| e
                .file_name()
                .to_string_lossy()
                .starts_with("yamantaka-selection-replacement-"))
    );
    assert_eq!(&hashes(&f.root)?, before);
    Ok(())
}
pub fn child(root: &Path, phase: &str) -> Result<()> {
    assert!(["IntentSaved", "PointerPreserved", "Installed"].contains(&phase));
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("selection-ñ-")
    );
    assert_eq!(
        fs::read(root.join(".synthetic-selection-probe"))?,
        b"Yamantaka synthetic selection acceptance"
    );
    let source = fs::read_dir(root.join("imports"))?
        .next()
        .ok_or("Missing synthetic import")??
        .path();
    let reviewed = repair::review(root, Some(&source), &Cancellation::default())?;
    repair::apply_progress(&reviewed, root, &Cancellation::default(), |stage| {
        if format!("{stage:?}") == phase {
            fs::write(root.join(".interrupted-stage"), phase).unwrap();
            loop {
                std::thread::park();
            }
        }
    })?;
    Err("Synthetic child did not reach its interruption point".into())
}
pub fn verify(report: &Path) -> Result<()> {
    let parent = report.parent().ok_or("Missing report parent")?;
    let f = fixture(parent, b"damaged synthetic selection".to_vec())?;
    let before = hashes(&f.root)?;
    let token = Cancellation::default();
    let original = repair::review(&f.root, None, &token)?;
    fs::write(
        f.imported.join("imported-resource"),
        b"unselected import changed",
    )?;
    repair::verify(&original, &token)?;
    fs::write(
        f.imported.join("imported-resource"),
        b"keep complete imported library",
    )?;
    assert!(managed_imports::recover_and_select(&f.root).is_err());
    assert!(
        matches!(managed_imports::recovery_target(&f.root)?,Some(managed_imports::RecoveryTarget::LibrarySelection(path)) if path==f.root)
    );
    let reviewed = repair::review(&f.root, Some(&f.imported), &token)?;
    assert_eq!(reviewed.counts()["documents"], 3);
    fs::write(
        f.root.join("selected-library.json"),
        b"newer synthetic selection",
    )?;
    assert!(repair::apply(&reviewed, &f.root, &token).is_err());
    assert_eq!(
        fs::read(f.root.join("selected-library.json"))?,
        b"newer synthetic selection"
    );
    fs::write(f.root.join("selected-library.json"), &f.pointer)?;
    let reviewed = repair::review(&f.root, Some(&f.imported), &token)?;
    fs::write(
        f.imported.join("imported-resource"),
        b"changed synthetic candidate",
    )?;
    assert!(repair::verify(&reviewed, &token).is_err());
    fs::write(
        f.imported.join("imported-resource"),
        b"keep complete imported library",
    )?;
    let reviewed = repair::review(&f.root, Some(&f.imported), &token)?;
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(repair::apply(&reviewed, &f.root, &cancel).is_err());
    assert!(!repair::pending(&f.root)?);
    assert_eq!(fs::read(f.root.join("selected-library.json"))?, f.pointer);
    let foreign = parent.join(f.imported.file_name().unwrap());
    fs::create_dir(&foreign)?;
    assert!(repair::review(&f.root, Some(&foreign), &token).is_err());
    fs::write(f.root.join("selected-library.json"), vec![b'X'; 65537])?;
    assert!(managed_imports::recover_and_select(&f.root).is_err());
    assert!(repair::review(&f.root, None, &token).is_err());
    assert_eq!(
        fs::read(f.root.join("selected-library.json"))?,
        vec![b'X'; 65537]
    );
    fs::write(f.root.join("selected-library.json"), &f.pointer)?;
    let reviewed = repair::review(&f.root, Some(&f.imported), &token)?;
    repair::apply(&reviewed, &f.root, &token)?;
    verify_choice(&f, true, &before)?;
    for phase in ["IntentSaved", "PointerPreserved", "Installed"] {
        let f = fixture(parent, b"preserve interrupted synthetic selection".to_vec())?;
        let before = hashes(&f.root)?;
        let mut child = Command::new(std::env::current_exe()?)
            .arg("--demo")
            .arg(format!("--selection-repair-kill-root={}", f.root.display()))
            .arg(format!("--selection-repair-kill-phase={phase}"))
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(15);
        while !f.root.join(".interrupted-stage").exists() {
            if child.try_wait()?.is_some() {
                return Err("Synthetic child exited before its interruption point".into());
            }
            if Instant::now() >= deadline {
                child.kill()?;
                child.wait()?;
                return Err("Synthetic interruption deadline expired".into());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        child.kill()?;
        assert!(!child.wait()?.success());
        assert!(repair::pending(&f.root)?);
        assert!(managed_imports::active_store(&f.root).is_err());
        verify_choice(&f, true, &before)?;
    }
    fs::write(
        report,
        serde_json::to_vec_pretty(
            &serde_json::json!({"synthetic":true,"staleRecordRejected":true,"staleLibraryRejected":true,"earlyCancellationPreserved":true,"foreignLibraryRejected":true,"oversizedRecordPreserved":true,"originalRecordPreserved":true,"libraryHashesUnchanged":true,"importedChoiceRestored":true,"processKillsRecovered":["IntentSaved","PointerPreserved","Installed"],"pendingIntentPreventsDefaultSelection":true,"idempotentRecovery":true}),
        )?,
    )?;
    Ok(())
}
