use yamantaka_domain::{
    Cancellation, Failure,
    annotations::repository::{AnnotationGrouping, AnnotationQuery, AnnotationRepository},
    memory::{GlobalMemoryLimits, MemoryGovernor},
};
use yamantaka_storage::Database;

fn seed(db: &Database) {
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,source_title,title_override,author,source_author,author_override,reading_progress,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf','A','Extracted A','A manual','Author A','Extracted author','Manual author',0.5,3,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf','B',NULL,NULL,NULL,NULL,NULL,0.25,3,1,'now','now','now','fb'); WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180) INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) SELECT printf('review-%03d',x),CASE WHEN x<2 THEN 'a' ELSE 'b' END,x%3,'pageComment','violet',CASE WHEN x<2 THEN 'Étude 学习' ELSE 'Other' END,CASE WHEN x<2 THEN 'fa' ELSE 'fb' END,'2026-10-10T12:00:00Z','2026-10-10T12:00:00Z' FROM n;").unwrap();
}

#[test]
fn review_snapshot_captures_the_filtered_page_and_canonical_metadata() {
    let db = Database::in_memory_at(11).unwrap();
    seed(&db);
    let governor = MemoryGovernor::default();
    let snapshot = db
        .annotation_review_snapshot(
            &AnnotationQuery {
                grouping: Some(AnnotationGrouping::Document),
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(snapshot.page().total, 181);
    assert_eq!(snapshot.page().entries().len(), 80);
    assert_eq!(snapshot.documents().len(), 2);
    let a = &snapshot.documents()[0];
    assert_eq!(a.document, "a");
    assert_eq!(a.title.as_deref(), Some("A manual"));
    assert_eq!(a.author.as_deref(), Some("Manual author"));
    assert_eq!(a.progress, 0.5);
    let b = &snapshot.documents()[1];
    assert_eq!(b.title.as_deref(), Some("B"));
    assert!(b.author.is_none());
    assert_eq!(b.progress, 0.25);
    drop(snapshot);
    assert_eq!(governor.usage().total, 0);
    let snapshot = db
        .annotation_review_snapshot(
            &AnnotationQuery {
                text: "学习".into(),
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(snapshot.page().total, 2);
    assert_eq!(snapshot.documents().len(), 1);
    drop(snapshot);
    let snapshot = db
        .annotation_review_snapshot(
            &AnnotationQuery {
                offset: 160,
                grouping: Some(AnnotationGrouping::Document),
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(snapshot.page().offset, 160);
    assert_eq!(snapshot.page().entries().len(), 21);
    assert_eq!(snapshot.documents()[0].document, "b");
    drop(snapshot);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn independent_wal_reader_keeps_the_returned_review_snapshot_frozen() {
    let directory = tempfile::tempdir().unwrap();
    let db = Database::open(&directory.path().join("library.sqlite3")).unwrap();
    seed(&db);
    let reader = db.export_reader().unwrap();
    let governor = MemoryGovernor::default();
    let query = AnnotationQuery {
        text: "学习".into(),
        ..Default::default()
    };
    let before = reader
        .annotation_review_snapshot(&query, &governor, &Cancellation::default())
        .unwrap();
    db.execute_batch("BEGIN IMMEDIATE; UPDATE documents SET title_override='Changed',author_override='Changed author',reading_progress=0.75 WHERE id='a'; UPDATE annotations SET note='Changed note' WHERE document_id='a'; COMMIT;").unwrap();
    assert_eq!(before.documents()[0].title.as_deref(), Some("A manual"));
    assert_eq!(
        before.documents()[0].author.as_deref(),
        Some("Manual author")
    );
    assert_eq!(before.documents()[0].progress, 0.5);
    assert!(
        before
            .page()
            .entries()
            .iter()
            .all(|r| r.data().values.note.as_deref() == Some("Étude 学习"))
    );
    let after = reader
        .annotation_review_snapshot(
            &AnnotationQuery {
                document_id: Some("a".into()),
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(after.documents()[0].title.as_deref(), Some("Changed"));
    assert_eq!(
        after.documents()[0].author.as_deref(),
        Some("Changed author")
    );
    assert_eq!(after.documents()[0].progress, 0.75);
    assert!(
        after
            .page()
            .entries()
            .iter()
            .all(|r| r.data().values.note.as_deref() == Some("Changed note"))
    );
    drop((before, after, reader));
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn cancelled_malformed_or_denied_metadata_releases_the_whole_review_snapshot() {
    let db = Database::in_memory_at(11).unwrap();
    seed(&db);
    let query = AnnotationQuery {
        document_id: Some("a".into()),
        ..Default::default()
    };
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.annotation_review_snapshot(&query, &governor, &cancel),
        Err(Failure::Cancelled)
    ));
    assert_eq!(governor.usage().total, 0);
    db.execute_batch(
        "UPDATE documents SET author_override=CAST(zeroblob(200000) AS TEXT) WHERE id='a';",
    )
    .unwrap();
    let limited = MemoryGovernor::new(GlobalMemoryLimits {
        total: 100000,
        text: 100000,
        ..Default::default()
    });
    let admitted_page = db
        .annotations(&query, &limited, &Cancellation::default())
        .unwrap();
    drop(admitted_page);
    assert_eq!(limited.usage().total, 0);
    assert!(matches!(
        db.annotation_review_snapshot(&query, &limited, &Cancellation::default()),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(limited.usage().total, 0);
    db.execute_batch("UPDATE documents SET author_override=CAST(X'FF' AS TEXT) WHERE id='a';")
        .unwrap();
    assert!(matches!(
        db.annotation_review_snapshot(&query, &governor, &Cancellation::default()),
        Err(Failure::Storage(_))
    ));
    assert_eq!(governor.usage().total, 0);
    db.execute_batch("UPDATE documents SET author_override='Recovered' WHERE id='a';")
        .unwrap();
    let snapshot = db
        .annotation_review_snapshot(&query, &governor, &Cancellation::default())
        .unwrap();
    assert_eq!(snapshot.documents()[0].author.as_deref(), Some("Recovered"));
    drop(snapshot);
    assert_eq!(governor.usage().total, 0);
}
