use std::{fs, path::Path};
use yamantaka_domain::{Cancellation, LibraryRepository};
use yamantaka_storage::{
    Database,
    managed_imports::{self, ImportTarget},
    migration::{LegacySource, review},
    preferences::Preferences,
};
pub fn verify(parent: &Path) {
    let root = parent.join("managed-libraries");
    fs::create_dir(&root).unwrap();
    let original = Database::open(&root.join("yamantaka.sqlite3")).unwrap();
    original
        .execute_batch(&super::DOCUMENT.replace("'original'", "'rust-only'"))
        .unwrap();
    let source_path = parent.join("managed-source.sqlite3");
    let source = Database::open(&source_path).unwrap();
    source.execute_batch(super::DOCUMENT).unwrap();
    let prefs = parent.join("managed-source-preferences.json");
    fs::write(&prefs, r#"{"flutter.languageCode":"es"}"#).unwrap();
    let plan = review(
        &LegacySource {
            database: source_path,
            preferences: prefs,
        },
        || Ok(true),
        &Cancellation::default(),
    )
    .unwrap();
    assert_eq!(managed_imports::active_store(&root).unwrap(), root);
    let target = ImportTarget::new(root.clone());
    target
        .capture(&plan, || Ok(true), &Cancellation::default(), |_| {})
        .unwrap();
    assert_eq!(managed_imports::active_store(&root).unwrap(), root);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(target.activate(&cancel).is_err());
    assert_eq!(managed_imports::active_store(&root).unwrap(), root);
    target.activate(&Cancellation::default()).unwrap();
    assert_eq!(managed_imports::active_store(&root).unwrap(), target.path());
    let copy = Database::open(&target.path().join("yamantaka.sqlite3")).unwrap();
    copy.execute_batch("UPDATE documents SET title='Edited after activation'")
        .unwrap();
    assert_eq!(managed_imports::active_store(&root).unwrap(), target.path());
    assert_eq!(
        Preferences::load(&target.path().join("preferences.json"))
            .unwrap()
            .get("languageCode")
            .unwrap(),
        "es"
    );
    assert_eq!(
        original.get("rust-only").unwrap().unwrap().title.as_deref(),
        Some("Original")
    );
    managed_imports::use_original(&root, &Cancellation::default()).unwrap();
    assert_eq!(managed_imports::active_store(&root).unwrap(), root);
    assert!(target.path().is_dir());
    for invalid in [
        r#"{"version":1,"import":"../outside"}"#,
        r#"{"version":999,"import":null}"#,
        r#"{"version":1,"import":"00000000-0000-0000-0000-000000000000"}"#,
    ] {
        fs::write(root.join("selected-library.json"), invalid).unwrap();
        assert!(managed_imports::active_store(&root).is_err());
    }
    assert_eq!(
        original.get("rust-only").unwrap().unwrap().title.as_deref(),
        Some("Original")
    );
}
