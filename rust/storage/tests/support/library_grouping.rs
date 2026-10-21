use yamantaka_domain::{Cancellation, LibraryRepository, MAX_LIBRARY_PAGE};
use yamantaka_storage::Database;

pub fn verify() {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch(super::DOCUMENT).unwrap();
    db.execute_batch("INSERT INTO collections VALUES('a','Alphabetic','alphabetic','folder',0,0,'2026-10-21','2026-10-21'),('b','Pinned later','pinned later','folder',9,1,'2026-10-21','2026-10-21'),('c','Pinned first','pinned first','folder',1,1,'2026-10-21','2026-10-21'); INSERT INTO collection_documents SELECT id,'original','2026-10-21' FROM collections; INSERT INTO tags VALUES('z','Zebra','zebra','violet','2026-10-21','2026-10-21'),('a','alpha','alpha','violet','2026-10-21','2026-10-21'); INSERT INTO document_tags SELECT 'original',id FROM tags;").unwrap();
    let ids = vec!["original".into(), "absent".into()];
    let cancel = Cancellation::default();
    let page = db.library_memberships(&ids, &cancel).unwrap();
    assert_eq!(page.len(), 1);
    assert_eq!(
        (
            page[0].document_id.as_str(),
            page[0].collection.as_deref(),
            page[0].tag.as_deref()
        ),
        ("original", Some("Pinned first"), Some("alpha"))
    );
    db.execute_batch("DELETE FROM collection_documents; DELETE FROM document_tags;")
        .unwrap();
    let page = db.library_memberships(&ids, &cancel).unwrap();
    assert_eq!(
        (page[0].collection.as_deref(), page[0].tag.as_deref()),
        (None, None)
    );
    assert!(db.library_memberships(&[], &cancel).unwrap().is_empty());
    assert!(
        db.library_memberships(&vec!["original".into(); MAX_LIBRARY_PAGE + 1], &cancel)
            .is_err()
    );
    cancel.cancel();
    assert!(db.library_memberships(&ids, &cancel).is_err());
}
