use yamantaka_domain::{Cancellation, Failure, LibraryRepository, quarantine::*};
use yamantaka_storage::Database;

pub fn verify(document: &str) {
    let mut db = Database::in_memory_at(11).unwrap();
    db.execute_batch(document).unwrap();
    let token = Cancellation::default();
    let original = db.get("original").unwrap().unwrap();
    assert!(db.automatic_restore_allowed("original", &token).unwrap());
    assert!(!db.automatic_restore_allowed("absent", &token).unwrap());
    for error in [
        Failure::Cancelled,
        Failure::PasswordRequired,
        Failure::MemoryBudget,
        Failure::QueueFull,
    ] {
        assert!(QuarantineReason::from_failure(&error).is_none());
    }
    assert!(
        QuarantineReason::from_failure(&Failure::Pdf("PRIVATE_FAILURE_CANARY".into())).is_some()
    );
    db.record_reader_health(
        &original,
        ReaderHealth::Failed(QuarantineReason::PdfOpenFailed),
        &token,
    )
    .unwrap();
    assert!(!db.automatic_restore_allowed("original", &token).unwrap());
    let page = db.quarantined_documents(0, &token).unwrap();
    assert_eq!((page.total, page.entries.len()), (1, 1));
    assert!(page.entries[0].reason == QuarantineReason::PdfOpenFailed);
    let old = page.entries[0].revision.clone();
    db.execute_batch(
        "UPDATE documents SET last_failure_code='PRIVATE_UNKNOWN_CODE' WHERE id='original'",
    )
    .unwrap();
    assert!(db.release_quarantine(&old, 0, &token).is_err());
    let page = db.quarantined_documents(0, &token).unwrap();
    assert!(page.entries[0].reason == QuarantineReason::InheritedOrUnknown);
    let expected = page.entries[0].revision.clone();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(db.release_quarantine(&expected, 0, &cancelled).is_err());
    db.execute_batch("CREATE TEMP TRIGGER reject_quarantine_release AFTER UPDATE OF restore_blocked ON documents WHEN NEW.restore_blocked=0 BEGIN SELECT RAISE(ABORT,'synthetic release failure'); END;").unwrap();
    assert!(db.release_quarantine(&expected, 0, &token).is_err());
    assert!(!db.automatic_restore_allowed("original", &token).unwrap());
    db.execute_batch("DROP TRIGGER reject_quarantine_release")
        .unwrap();
    assert_eq!(
        db.release_quarantine(&expected, 0, &token).unwrap().total,
        0
    );
    assert!(db.automatic_restore_allowed("original", &token).unwrap());
    db.record_reader_health(
        &original,
        ReaderHealth::Failed(QuarantineReason::PdfOpenFailed),
        &token,
    )
    .unwrap();
    let mut stale = original.clone();
    stale.path = "different.pdf".into();
    assert!(
        db.record_reader_health(&stale, ReaderHealth::Opened, &token)
            .is_err()
    );
    assert!(!db.automatic_restore_allowed("original", &token).unwrap());
    db.record_reader_health(&original, ReaderHealth::Opened, &token)
        .unwrap();
    assert!(db.automatic_restore_allowed("original", &token).unwrap());
    let current = db.get("original").unwrap().unwrap();
    assert_eq!(current.title, original.title);
    assert_eq!(current.reading_progress, original.reading_progress);
    db.execute_batch("WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<80) INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,restore_blocked,last_failure_code) SELECT printf('blocked-%03d',x),printf('blocked-%03d.pdf',x),printf('blocked-%03d.pdf',x),'fixture.pdf',2,10,'now','now','now','legacy',1,'unknownFutureCode' FROM n").unwrap();
    let page = db.quarantined_documents(0, &token).unwrap();
    assert_eq!((page.total, page.entries.len()), (81, 80));
    assert_eq!(
        db.quarantined_documents(80, &token).unwrap().entries.len(),
        1
    );
    db.execute_batch("UPDATE documents SET restore_blocked=1 WHERE id='original'; PRAGMA ignore_check_constraints=ON; UPDATE documents SET page_count=-1 WHERE id='blocked-000';").unwrap();
    let expected = db
        .quarantined_documents(80, &token)
        .unwrap()
        .entries
        .into_iter()
        .find(|entry| entry.document.id == "original")
        .unwrap()
        .revision;
    assert!(db.release_quarantine(&expected, 0, &token).is_err());
    assert!(!db.automatic_restore_allowed("original", &token).unwrap());
    db.execute_batch("UPDATE documents SET page_count=2 WHERE id='blocked-000'; PRAGMA ignore_check_constraints=OFF;").unwrap();
    db.integrity().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("quarantine.sqlite3");
    let mut disk = Database::open(&path).unwrap();
    disk.execute_batch(document).unwrap();
    let source = disk.get("original").unwrap().unwrap();
    disk.record_reader_health(
        &source,
        ReaderHealth::Failed(QuarantineReason::PdfOpenFailed),
        &token,
    )
    .unwrap();
    drop(disk);
    let mut reopened = Database::open(&path).unwrap();
    assert!(
        !reopened
            .automatic_restore_allowed("original", &token)
            .unwrap()
    );
    let revision = reopened
        .quarantined_documents(0, &token)
        .unwrap()
        .entries
        .remove(0)
        .revision;
    assert_eq!(
        reopened
            .release_quarantine(&revision, 0, &token)
            .unwrap()
            .total,
        0
    );
    drop(reopened);
    assert!(
        Database::open_read_only(&path)
            .unwrap()
            .automatic_restore_allowed("original", &token)
            .unwrap()
    );
}
