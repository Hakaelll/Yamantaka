use yamantaka_domain::{Cancellation, diagnostics::DiagnosticsRepository};
use yamantaka_storage::Database;

pub fn verify(document: &str) {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch(document).unwrap();
    db.execute_batch("UPDATE documents SET title='PRIVATE_DIAGNOSTIC_CANARY',author='PRIVATE_DIAGNOSTIC_CANARY',path='C:/private/PRIVATE_DIAGNOSTIC_CANARY.pdf',is_missing=1;").unwrap();
    let cancel = Cancellation::default();
    let report = db.diagnostics(&cancel).unwrap();
    assert_eq!(report.schema, 11);
    assert_eq!(report.counts.documents, 1);
    assert_eq!(report.counts.missing_documents, 1);
    assert_eq!(report.counts.annotations, 0);
    assert!(report.logical_database_bytes > 0);
    assert!(report.quick_check_ok && report.foreign_keys_ok);
    let encoded = serde_json::to_string(&report).unwrap();
    assert!(!encoded.contains("PRIVATE_DIAGNOSTIC_CANARY"));
    assert!(!encoded.contains("private"));
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("diagnostics.json");
    db.publish_local_export(&destination, &cancel, |file| {
        use std::io::Write;
        file.write_all(encoded.as_bytes())
            .map_err(|error| yamantaka_domain::Failure::Storage(error.to_string()))
    })
    .unwrap();
    assert_eq!(std::fs::read_to_string(&destination).unwrap(), encoded);
    assert!(
        db.publish_local_export(&destination, &cancel, |_| Ok(()))
            .is_err()
    );
    assert_eq!(std::fs::read_to_string(&destination).unwrap(), encoded);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(db.diagnostics(&cancelled).is_err());
    assert!(
        db.publish_local_export(&root.path().join("cancelled.json"), &cancelled, |_| Ok(()))
            .is_err()
    );
    assert!(!root.path().join("cancelled.json").exists());
    db.integrity().unwrap();
}
