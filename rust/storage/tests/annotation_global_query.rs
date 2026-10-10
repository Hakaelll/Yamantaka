use yamantaka_domain::{
    Cancellation,
    annotations::{AnnotationReviewState, repository::*},
    memory::MemoryGovernor,
};
use yamantaka_storage::Database;

fn library() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,source_title,title_override,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf','Source A','Extracted A','Zulu manual',3,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf','Alpha',NULL,NULL,3,1,'now','now','now','fb'); INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at,review_state) VALUES('a1','a',2,'pageComment','violet','Étude 学习','fa','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z','accepted'),('a2','a',0,'pageComment','blue','Other','fa','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z','pending'),('b1','b',1,'pageComment','violet','Study','fb','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z','resolved');").unwrap();
    db
}
#[test]
fn global_grouping_uses_manual_catalogue_titles_pages_and_editorial_order() {
    let db = library();
    let governor = MemoryGovernor::default();
    for (group, expected) in [
        (AnnotationGrouping::Document, ["b1", "a1", "a2"]),
        (AnnotationGrouping::Page, ["a2", "b1", "a1"]),
        (AnnotationGrouping::Review, ["a2", "a1", "b1"]),
    ] {
        let page = db
            .annotations(
                &AnnotationQuery {
                    grouping: Some(group),
                    ..Default::default()
                },
                &governor,
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(
            page.entries()
                .iter()
                .map(|r| r.data().id.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        for row in page.entries() {
            assert_eq!(
                row.data().document_title.as_deref(),
                Some(if row.data().values.document_id == "a" {
                    "Zulu manual"
                } else {
                    "Alpha"
                })
            );
        }
    }
    assert_eq!(governor.usage().total, 0);
}
#[test]
fn global_grouping_happens_before_pagination_and_titles_remain_governed() {
    let db = library();
    let governor = MemoryGovernor::default();
    db.execute_batch("WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<179) INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) SELECT printf('synthetic-%03d',x),CASE WHEN x%2=0 THEN 'a' ELSE 'b' END,0,'pageComment','blue','Study','synthetic','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z' FROM n;").unwrap();
    for (offset, count, b_count) in [(0, 80, 80), (80, 80, 11), (160, 23, 0)] {
        let page = db
            .annotations(
                &AnnotationQuery {
                    grouping: Some(AnnotationGrouping::Document),
                    offset,
                    ..Default::default()
                },
                &governor,
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(page.total, 183);
        assert_eq!(page.entries().len(), count);
        assert_eq!(
            page.entries()
                .iter()
                .filter(|r| r.data().values.document_id == "b")
                .count(),
            b_count
        );
        assert!(governor.usage().text > 0);
        assert!(
            page.entries()
                .iter()
                .all(|r| r.data().document_title.is_some())
        );
    }
    let mut visited = 0;
    db.visit_annotations("a", &governor, &Cancellation::default(), &mut |row| {
        visited += 1;
        assert!(
            row.data().document_title.is_none(),
            "Export traversal does not need presentation metadata"
        );
        Ok(())
    })
    .unwrap();
    assert_eq!(visited, 92);
    assert_eq!(governor.usage().total, 0);
    let page = db
        .annotations(
            &AnnotationQuery {
                review: Some(AnnotationReviewState::Accepted),
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(page.total, 1);
}

#[test]
fn global_notes_can_page_beyond_one_hundred_thousand_annotations() {
    let db = library();
    let governor = MemoryGovernor::default();
    db.execute_batch("WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<100119) INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) SELECT printf('synthetic-%06d',x),'a',0,'pageComment','blue','Study','synthetic','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z' FROM n;").unwrap();
    let page = db
        .annotations(
            &AnnotationQuery {
                grouping: Some(AnnotationGrouping::Document),
                offset: 100080,
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(page.total, 100123);
    assert_eq!(page.offset, 100080);
    assert_eq!(page.entries().len(), 43);
    assert_eq!(page.entries()[0].data().id, "synthetic-100077");
    drop(page);
    assert_eq!(governor.usage().total, 0);
}
