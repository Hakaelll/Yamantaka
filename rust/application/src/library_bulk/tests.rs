use super::*;
use yamantaka_domain::{LibraryQuery, library_selection::LibrarySelection};
use yamantaka_storage::Database;
#[test]
fn page_selection_and_bulk_operations_keep_fixed_targets_and_partial_results() {
    let page: Vec<String> = ["a", "b", "c", "d"].map(String::from).to_vec();
    let mut selection = LibrarySelection::default();
    selection.select(&page, "a", true, false);
    selection.select(&page, "c", false, true);
    assert_eq!(selection.ids(), &["a", "b", "c"]);
    selection.select(&page, "b", true, false);
    selection.select(&page, "d", true, false);
    assert_eq!(selection.ids(), &["a", "c", "d"]);
    let targets = selection.ids().to_vec();
    selection.retain(&["a".into(), "b".into()]);
    assert_eq!(selection.ids(), &["a"]);
    selection.all(&page);
    assert!(selection.mode);
    assert_eq!(selection.ids(), &["a", "b", "c", "d"]);
    selection.clear();
    assert!(!selection.mode);
    let mut db = Database::in_memory_at(11).unwrap();
    for id in &page {
        db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,title,author,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,reading_progress,last_page_index,title_override) VALUES('{id}','{id}.pdf','{id}.pdf','fixture.pdf','Title','Author',2,900,'2026-10-20','2026-10-20','2026-10-20','synthetic',0.5,1,'My title')")).unwrap();
    }
    let cancel = Cancellation::default();
    let mut job = BulkJob::start(BulkRequest {
        ids: targets,
        action: BulkAction::Favorite(true),
        cancel: cancel.clone(),
    })
    .unwrap();
    assert!(!job.step(&mut db, IndexAccess::Disabled));
    assert!(db.get("a").unwrap().unwrap().favorite);
    assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 4);
    cancel.cancel();
    assert!(job.step(&mut db, IndexAccess::Disabled));
    let report = job.reply().result.unwrap();
    assert_eq!(report.succeeded, ["a"]);
    assert!(report.failed.is_empty());
    assert_eq!(report.remaining, ["c", "d"]);
    assert!(!db.get("c").unwrap().unwrap().favorite);
    let mut job = BulkJob::start(BulkRequest {
        ids: vec!["a".into(), "missing".into(), "d".into(), "a".into()],
        action: BulkAction::Favorite(false),
        cancel: Cancellation::default(),
    })
    .unwrap();
    while !job.step(&mut db, IndexAccess::Disabled) {}
    let report = job.reply().result.unwrap();
    assert_eq!(report.succeeded, ["a", "d"]);
    assert_eq!(report.failed, ["missing"]);
    assert!(report.remaining.is_empty());
    let doc = db.get("a").unwrap().unwrap();
    assert!(!doc.favorite);
    assert_eq!(doc.display_title(), "My title");
    assert_eq!(doc.last_page_index, 1);
    assert_eq!(doc.reading_progress, 0.5);
    assert!(
        BulkJob::start(BulkRequest {
            ids: vec!["a".into(); 81],
            action: BulkAction::Favorite(true),
            cancel: Cancellation::default()
        })
        .is_err()
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    let mut job = BulkJob::start(BulkRequest {
        ids: page,
        action: BulkAction::Favorite(true),
        cancel,
    })
    .unwrap();
    assert!(job.step(&mut db, IndexAccess::Disabled));
    assert_eq!(job.reply().result.unwrap().remaining, ["a", "b", "c", "d"]);
    let tag = db
        .change_tag(
            &TagChange::Ensure {
                name: "Shared study".into(),
            },
            IndexAccess::Disabled,
            &Cancellation::default(),
        )
        .unwrap();
    db.execute_batch(&format!("INSERT INTO annotations(id,document_id,page_index,type,color,document_fingerprint,created_at,updated_at) VALUES('note','a',0,'note','violet','synthetic','2026-10-20','2026-10-20'); INSERT INTO annotation_tags(annotation_id,tag_id) VALUES('note','{tag}')")).unwrap();
    let run = |db: &mut Database, action, ids: Vec<String>| {
        let mut job = BulkJob::start(BulkRequest {
            ids,
            action,
            cancel: Cancellation::default(),
        })
        .unwrap();
        while !job.step(db, IndexAccess::Disabled) {}
        job.reply().result.unwrap()
    };
    let ids = vec!["a".into(), "c".into(), "missing".into()];
    let tagged = run(&mut db, BulkAction::Tag(tag.clone()), ids.clone());
    assert_eq!(tagged.succeeded, ["a", "c"]);
    assert_eq!(tagged.failed, ["missing"]);
    let tags = db
        .tags(
            &yamantaka_domain::tags::TagQuery {
                ids: vec![tag.clone()],
                ..Default::default()
            },
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(tags.entries[0].document_count, 2);
    assert_eq!(tags.entries[0].annotation_count, 1);
    assert_eq!(
        db.count(&LibraryQuery {
            tag_ids: vec![tag],
            ..Default::default()
        })
        .unwrap(),
        2
    );
    let collection = db
        .change_collection(
            &CollectionChange::Create {
                name: "Study".into(),
                icon: "study".into(),
            },
            &Cancellation::default(),
        )
        .unwrap();
    let lookup = |db: &Database, id: &str| {
        db.collections(
            &yamantaka_domain::collections::CollectionQuery {
                id: Some(id.into()),
                ..Default::default()
            },
            &Cancellation::default(),
        )
        .unwrap()
        .entries
        .remove(0)
    };
    let initial = lookup(&db, &collection);
    let added = run(
        &mut db,
        BulkAction::Collection {
            expected: (&initial).into(),
            included: true,
        },
        ids.clone(),
    );
    assert_eq!(added.succeeded, ["a", "c"]);
    assert_eq!(added.failed, ["missing"]);
    assert_eq!(lookup(&db, &collection).document_count, 2);
    db.change_collection(
        &CollectionChange::Rename {
            expected: (&initial).into(),
            name: "Renamed study".into(),
        },
        &Cancellation::default(),
    )
    .unwrap();
    let stale = run(
        &mut db,
        BulkAction::Collection {
            expected: (&initial).into(),
            included: false,
        },
        vec!["a".into(), "c".into()],
    );
    assert!(stale.succeeded.is_empty());
    assert_eq!(stale.failed, ["a", "c"]);
    let fresh = lookup(&db, &collection);
    let removed = run(
        &mut db,
        BulkAction::Collection {
            expected: (&fresh).into(),
            included: false,
        },
        vec!["a".into(), "c".into()],
    );
    assert_eq!(removed.succeeded, ["a", "c"]);
    assert_eq!(lookup(&db, &collection).document_count, 0);
    let archived = run(&mut db, BulkAction::Archive, ids.clone());
    assert_eq!(archived.succeeded, ["a", "c"]);
    assert_eq!(archived.failed, ["missing"]);
    let archives = db
        .collections(
            &yamantaka_domain::collections::CollectionQuery {
                text: "Archivado".into(),
                ..Default::default()
            },
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(archives.total, 1);
    assert_eq!(archives.entries[0].document_count, 2);
    let archive_id = archives.entries[0].id.clone();
    let repeated = run(&mut db, BulkAction::Archive, vec!["a".into(), "c".into()]);
    assert_eq!(repeated.succeeded, ["a", "c"]);
    assert_eq!(lookup(&db, &archive_id).document_count, 2);
    db.execute_batch("CREATE TRIGGER reject_bulk_collection BEFORE INSERT ON collection_documents WHEN NEW.document_id='d' BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
    assert!(
        db.change_collection(
            &CollectionChange::EnsureForDocument {
                name: "Atomic collection".into(),
                icon: "archive".into(),
                document: "d".into()
            },
            &Cancellation::default()
        )
        .is_err()
    );
    assert_eq!(
        db.collections(
            &yamantaka_domain::collections::CollectionQuery {
                text: "Atomic collection".into(),
                ..Default::default()
            },
            &Cancellation::default()
        )
        .unwrap()
        .total,
        0
    );
    db.execute_batch("DROP TRIGGER reject_bulk_collection")
        .unwrap();
    assert_eq!(db.get("a").unwrap().unwrap().display_title(), "My title");
    assert_eq!(db.get("a").unwrap().unwrap().reading_progress, 0.5);
}
