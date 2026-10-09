use yamantaka_domain::{Cancellation, Failure, annotations::repository::*, memory::*};
use yamantaka_storage::Database;

fn fixture() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf',3,1,'now','now','now','fb');
    WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<181)
    INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state)
    SELECT 'm'||x,'a',2,CASE x%3 WHEN 0 THEN 'highlight' WHEN 1 THEN 'underline' ELSE 'strikeout' END,'violet',printf('%100000s','study'),'[{\"left\":0.1,\"top\":0.2,\"right\":0.3,\"bottom\":0.4}]',0,'fa','attached','now','now','pending' FROM n;
    INSERT INTO annotations(id,document_id,page_index,type,color,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state) VALUES('other-page','a',1,'highlight','green','[]',0,'fa','attached','now','now','pending'),('other-document','b',2,'highlight','green','[]',0,'fb','attached','now','now','pending'),('detached','a',2,'highlight','green','[]',0,'fa','reviewRequired','now','now','pending');").unwrap();
    db
}
#[test]
fn marks_include_every_attached_row_on_the_page_without_loading_note_payloads() {
    let db = fixture();
    let governor = MemoryGovernor::new(GlobalMemoryLimits {
        text: 128 * 1024,
        ..Default::default()
    });
    let marks = db
        .annotation_marks("a", 2, &governor, &Cancellation::default())
        .unwrap();
    assert_eq!(marks.page(), 2);
    assert_eq!(marks.marks().len(), 181);
    assert!(marks.marks().iter().all(|mark| mark.rects().len() == 1));
    assert!(
        db.annotations(
            &AnnotationQuery {
                document_id: Some("a".into()),
                page: Some(2),
                ..Default::default()
            },
            &governor,
            &Cancellation::default()
        )
        .is_err()
    );
    assert!(governor.usage().text < 128 * 1024);
    drop(marks);
    assert_eq!(governor.usage().text, 0);
    let page = db
        .annotations(
            &AnnotationQuery {
                document_id: Some("a".into()),
                page: Some(2),
                ..Default::default()
            },
            &MemoryGovernor::new(GlobalMemoryLimits {
                text: 32 * 1024 * 1024,
                total: 64 * 1024 * 1024,
                ..Default::default()
            }),
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(page.entries().len(), 80);
    assert_eq!(page.total, 182);
}
#[test]
fn cancelled_denied_and_late_corrupt_geometry_release_the_complete_snapshot() {
    let db = fixture();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.annotation_marks("a", 2, &governor, &cancel),
        Err(Failure::Cancelled)
    ));
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        text: 1,
        ..Default::default()
    });
    assert!(matches!(
        db.annotation_marks("a", 2, &denied, &Cancellation::default()),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(denied.usage().text, 0);
    db.execute_batch("UPDATE annotations SET rects_json='[{\"left\":-1,\"top\":0,\"right\":1,\"bottom\":1}]' WHERE id='m99';").unwrap();
    assert!(matches!(
        db.annotation_marks("a", 2, &governor, &Cancellation::default()),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(governor.usage().text, 0);
    assert_eq!(
        db.annotation_marks("b", 2, &governor, &Cancellation::default())
            .unwrap()
            .marks()
            .len(),
        1
    );
    assert_eq!(governor.usage().text, 0);
}
