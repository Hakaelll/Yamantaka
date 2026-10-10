use yamantaka_domain::{
    Cancellation, Failure,
    annotations::{AnnotationReviewState, repository::*},
    memory::{GlobalMemoryLimits, MemoryGovernor},
    search::{IndexAccess, SearchKind, SearchQuery, SearchRepository},
};
use yamantaka_storage::Database;

fn library() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',1,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf',1,1,'now','now','now','fb');
        INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) VALUES('a1','a',0,'pageComment','violet','Étude 学习','fa','original','original'),('a2','a',0,'pageComment','blue','Other','fa','original','original'),('b1','b',0,'pageComment','violet','Study','fb','original','original');").unwrap();
    db
}
fn targets() -> Vec<AnnotationTarget> {
    vec![
        AnnotationTarget {
            document_id: "a".into(),
            annotation_id: "a1".into(),
        },
        AnnotationTarget {
            document_id: "b".into(),
            annotation_id: "b1".into(),
        },
    ]
}
fn assert_original(db: &Database) {
    let governor = MemoryGovernor::default();
    let page = db
        .annotations(&Default::default(), &governor, &Cancellation::default())
        .unwrap();
    for record in page.entries() {
        assert_eq!(
            record.data().values.review_state,
            AnnotationReviewState::Pending
        );
        assert_eq!(record.data().updated_at, "original");
        assert!(record.data().tags.is_empty());
    }
    drop(page);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn global_review_and_tags_update_selected_documents_and_the_consented_index() {
    let mut db = library();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let mut targets = targets();
    targets.push(targets[0].clone());
    let query = AnnotationQuery {
        review: Some(AnnotationReviewState::Pending),
        ..Default::default()
    };
    let page = db
        .change_annotation_batch(
            AnnotationBatchChange::Review {
                targets: &targets,
                state: AnnotationReviewState::Accepted,
            },
            &query,
            IndexAccess::Enabled,
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.entries()[0].data().id, "a2");
    drop(page);
    for tag in [" Étude ", " étude "] {
        let page = db
            .change_annotation_batch(
                AnnotationBatchChange::AddTag {
                    targets: &targets,
                    tag,
                },
                &Default::default(),
                IndexAccess::Enabled,
                &governor,
                &cancel,
            )
            .unwrap();
        assert_eq!(page.total, 3);
        for record in page.entries() {
            let selected = record.data().id != "a2";
            assert_eq!(
                record.data().values.review_state,
                if selected {
                    AnnotationReviewState::Accepted
                } else {
                    AnnotationReviewState::Pending
                }
            );
            assert_eq!(record.data().tags.len(), usize::from(selected));
            if selected {
                assert_eq!(record.data().tags, ["Étude"]);
            }
        }
    }
    let hits = db
        .search(
            &SearchQuery {
                text: "Étude".into(),
                kinds: vec![SearchKind::Annotation],
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(hits.len(), 2);
    drop(hits);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn missing_or_moved_late_targets_roll_back_every_document() {
    for missing in [true, false] {
        let mut db = library();
        let governor = MemoryGovernor::default();
        let mut targets = targets();
        targets[1].annotation_id = if missing { "missing" } else { "a2" }.into();
        for tagging in [false, true] {
            let change = if tagging {
                AnnotationBatchChange::AddTag {
                    targets: &targets,
                    tag: "New",
                }
            } else {
                AnnotationBatchChange::Review {
                    targets: &targets,
                    state: AnnotationReviewState::Resolved,
                }
            };
            assert!(
                db.change_annotation_batch(
                    change,
                    &Default::default(),
                    IndexAccess::Enabled,
                    &governor,
                    &Cancellation::default()
                )
                .is_err()
            );
            assert_original(&db);
        }
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn cancelled_denied_or_oversized_batches_leave_original_rows() {
    let mut db = library();
    let governor = MemoryGovernor::default();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    let targets = targets();
    assert!(matches!(
        db.change_annotation_batch(
            AnnotationBatchChange::Review {
                targets: &targets,
                state: AnnotationReviewState::Accepted
            },
            &Default::default(),
            IndexAccess::Disabled,
            &governor,
            &cancelled
        ),
        Err(Failure::Cancelled)
    ));
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        text: 0,
        ..Default::default()
    });
    assert!(matches!(
        db.change_annotation_batch(
            AnnotationBatchChange::Review {
                targets: &targets,
                state: AnnotationReviewState::Accepted
            },
            &Default::default(),
            IndexAccess::Disabled,
            &denied,
            &Cancellation::default()
        ),
        Err(Failure::MemoryBudget)
    ));
    for targets in [vec![], vec![targets[0].clone(); 81]] {
        assert!(
            db.change_annotation_batch(
                AnnotationBatchChange::Review {
                    targets: &targets,
                    state: AnnotationReviewState::Accepted
                },
                &Default::default(),
                IndexAccess::Disabled,
                &governor,
                &Cancellation::default()
            )
            .is_err()
        );
    }
    assert_original(&db);
    assert_eq!(denied.usage().total, 0);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn late_tag_index_and_refresh_failures_roll_back_cross_document_changes() {
    for failure in ["tags", "index", "decode"] {
        let mut db = library();
        match failure {
            "tags" => db.execute_batch("WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<11) INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) SELECT printf('tag-%02d',x),printf('Tag%d',x),printf('tag%d',x),'blue','original','original' FROM n; INSERT INTO annotation_tags(annotation_id,tag_id) SELECT 'b1',id FROM tags;").unwrap(),
            "index" => db.execute_batch("DROP TABLE library_search").unwrap(),
            "decode" => db.execute_batch("UPDATE annotations SET rects_json='invalid' WHERE id='a2'").unwrap(),
            _ => unreachable!(),
        }
        let governor = MemoryGovernor::default();
        assert!(
            db.change_annotation_batch(
                AnnotationBatchChange::AddTag {
                    targets: &targets(),
                    tag: "New"
                },
                &Default::default(),
                IndexAccess::Enabled,
                &governor,
                &Cancellation::default(),
            )
            .is_err()
        );
        if failure == "decode" {
            db.execute_batch("UPDATE annotations SET rects_json='[]' WHERE id='a2'")
                .unwrap();
        }
        let page = db
            .annotations(&Default::default(), &governor, &Cancellation::default())
            .unwrap();
        assert_eq!(page.total, 3);
        assert!(
            page.entries()
                .iter()
                .all(|r| !r.data().tags.iter().any(|tag| tag == "New"))
        );
        assert!(
            page.entries()
                .iter()
                .all(|r| r.data().updated_at == "original")
        );
        if failure == "tags" {
            assert_eq!(
                page.entries()
                    .iter()
                    .map(|r| r.data().tags.len())
                    .sum::<usize>(),
                12
            );
        }
        drop(page);
        assert_eq!(governor.usage().total, 0);
    }
}
