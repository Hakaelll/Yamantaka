use yamantaka_domain::{Cancellation, Failure, annotations::repository::*, memory::*};
use yamantaka_storage::Database;
fn fixture(db: &Database) {
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa'); WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180) INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state) SELECT printf('r%03d',x),'a',x/80,'pageComment','blue','Original','[]',0,'fa','attached','2026-10-08T12:00:00Z','2026-10-09T12:00:00Z','pending' FROM n; INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('old','Old','old','blue','now','now'),('new','New','new','blue','now','now'); INSERT INTO annotation_tags(annotation_id,tag_id) SELECT id,'old' FROM annotations;").unwrap();
}
#[test]
fn wal_snapshot_keeps_later_rows_and_tags_coherent_after_another_connection_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.sqlite3");
    let db = Database::open(&path).unwrap();
    fixture(&db);
    let writer = Database::open(&path).unwrap();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let mut seen = 0;
    let count=db.visit_annotations("a",&governor,&cancel,&mut |record| {
        if seen==0 {writer.execute_batch("UPDATE annotations SET note='Changed'; UPDATE annotation_tags SET tag_id='new'; DELETE FROM annotations WHERE id='r180';")?;}
        assert_eq!(record.data().values.note.as_deref(),Some("Original"));assert_eq!(record.data().tags,["Old"]);seen+=1;Ok(())
    }).unwrap();
    assert_eq!(count, 181);
    assert_eq!(seen, 181);
    assert_eq!(governor.usage().text, 0);
    let count = db
        .visit_annotations("a", &governor, &cancel, &mut |record| {
            assert_eq!(record.data().values.note.as_deref(), Some("Changed"));
            assert_eq!(record.data().tags, ["New"]);
            Ok(())
        })
        .unwrap();
    assert_eq!(count, 180);
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn callback_failure_cancellation_and_denied_memory_release_rows_and_allow_retry() {
    let db = Database::in_memory_at(11).unwrap();
    fixture(&db);
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let mut seen = 0;
    assert!(matches!(
        db.visit_annotations("a", &governor, &cancel, &mut |_| {
            seen += 1;
            if seen == 81 {
                Err(Failure::Storage("Synthetic writer failure".into()))
            } else {
                Ok(())
            }
        }),
        Err(Failure::Storage(_))
    ));
    assert_eq!(seen, 81);
    assert_eq!(governor.usage().text, 0);
    assert!(matches!(
        db.visit_annotations("a", &governor, &cancel, &mut |_| {
            cancel.cancel();
            Ok(())
        }),
        Err(Failure::Cancelled)
    ));
    assert_eq!(governor.usage().text, 0);
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        text: 1,
        ..Default::default()
    });
    assert!(matches!(
        db.visit_annotations("a", &denied, &Cancellation::default(), &mut |_| panic!(
            "visitor must not run"
        )),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(denied.usage().text, 0);
    assert_eq!(
        db.visit_annotations("a", &governor, &Cancellation::default(), &mut |_| Ok(()))
            .unwrap(),
        181
    );
    assert_eq!(governor.usage().text, 0);
    assert_eq!(
        db.visit_annotations(
            "missing",
            &governor,
            &Cancellation::default(),
            &mut |_| panic!("wrong document")
        )
        .unwrap(),
        0
    );
}
#[test]
fn historical_migrations_preserve_visit_payloads() {
    let governor = MemoryGovernor::default();
    for version in 1..=11 {
        let mut db = Database::in_memory_at(version).unwrap();
        db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('legacy-doc','legacy.pdf','legacy.pdf','legacy.pdf',3,1,'now','now','now','legacy-sha');").unwrap();
        if version >= 2 {
            db.execute_batch("INSERT INTO annotations(id,document_id,page_index,type,color,note,selected_text,rects_json,page_rotation,document_fingerprint,created_at,updated_at) VALUES('legacy','legacy-doc',1,'textNote','rose','Legacy 学习','Original passage','[]',270,'legacy-sha','2020-01-02T03:04:05Z','2020-01-02T03:04:05Z');").unwrap();
        }
        db.migrate_current().unwrap();
        let legacy_count = db
            .visit_annotations(
                "legacy-doc",
                &governor,
                &Cancellation::default(),
                &mut |record| {
                    let data = record.data();
                    assert_eq!(data.id, "legacy");
                    assert_eq!(data.values.note.as_deref(), Some("Legacy 学习"));
                    assert_eq!(
                        data.values.selected_text.as_deref(),
                        Some("Original passage")
                    );
                    assert_eq!(data.values.page_rotation, 270);
                    assert_eq!(data.values.document_fingerprint, "legacy-sha");
                    assert_eq!(data.created_at, "2020-01-02T03:04:05Z");
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(legacy_count, u64::from(version >= 2));
        fixture(&db);
        let mut previous = 0;
        let count = db
            .visit_annotations("a", &governor, &Cancellation::default(), &mut |record| {
                assert!(record.data().values.page_index >= previous);
                previous = record.data().values.page_index;
                assert_eq!(record.data().tags, ["Old"]);
                Ok(())
            })
            .unwrap();
        assert_eq!(count, 181);
        assert_eq!(governor.usage().text, 0);
    }
}
