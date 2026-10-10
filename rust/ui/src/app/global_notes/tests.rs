use super::*;
use yamantaka_domain::{Failure, memory::MemoryGovernor, search::IndexAccess};
use yamantaka_storage::Database;

fn library() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,title_override,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf','Source A','Manual A',1,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf','B',NULL,1,1,'now','now','now','fb'); INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) VALUES('a1','a',0,'pageComment','violet','Étude 学习','fa','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z'),('b1','b',0,'pageComment','violet','Study','fb','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z');").unwrap();
    db
}
fn execute(db: &mut Database, request: AnnotationRequest) -> AnnotationReply {
    let result = match request.operation {
        AnnotationOperation::Read => {
            db.annotations(&request.query, &request.governor, &request.cancel)
        }
        AnnotationOperation::Batch { mutation } => {
            let change = match &mutation {
                AnnotationBatchMutation::Review { targets, state } => {
                    AnnotationBatchChange::Review {
                        targets,
                        state: *state,
                    }
                }
                AnnotationBatchMutation::AddTag { targets, tag } => {
                    AnnotationBatchChange::AddTag { targets, tag }
                }
            };
            db.change_annotation_batch(
                change,
                &request.query,
                IndexAccess::Disabled,
                &request.governor,
                &request.cancel,
            )
        }
        _ => panic!("Global fixture expects global operations"),
    };
    AnnotationReply {
        document_id: request.query.document_id,
        result,
        cancel: request.cancel,
    }
}
#[test]
fn global_reads_reject_stale_replies_and_keep_document_scope_separate() {
    let governor = MemoryGovernor::default();
    let mut state = GlobalNotes::default();
    let old = state.request(None, 0, &governor).unwrap();
    state.filters.text = " Study ".into();
    let current = state.request(None, 0, &governor).unwrap();
    assert!(old.cancel.is_cancelled());
    assert_eq!(current.query.text, "Study");
    assert!(current.query.document_id.is_none());
    let stale = AnnotationReply {
        document_id: None,
        result: Err(Failure::Cancelled),
        cancel: old.cancel,
    };
    assert!(!state.matches(&stale));
    assert!(!state.accept(stale));
    assert!(state.pending.is_some());
    let reader = AnnotationReply {
        document_id: Some("a".into()),
        result: Err(Failure::Cancelled),
        cancel: current.cancel.clone(),
    };
    assert!(!state.matches(&reader));
    state.leave();
    assert!(current.cancel.is_cancelled());
    assert!(state.pending.is_none());
    drop(state);
    assert_eq!(governor.usage().total, 0);
}
#[test]
fn global_review_freezes_documents_and_finishes_after_leaving_the_screen() {
    let governor = MemoryGovernor::default();
    let mut db = library();
    let mut state = GlobalNotes::default();
    let request = state.request(None, 0, &governor).unwrap();
    state.accept(execute(&mut db, request));
    assert!(state.ready());
    assert_eq!(state.page.as_ref().unwrap().total, 2);
    assert_eq!(
        state.page.as_ref().unwrap().entries()[1]
            .data()
            .document_title
            .as_deref(),
        Some("Manual A")
    );
    state.selected[0] = true;
    state.selected[1] = true;
    let targets = state.targets();
    assert_eq!(targets.len(), 2);
    assert_ne!(targets[0].document_id, targets[1].document_id);
    let request = state
        .request(
            Some(AnnotationBatchMutation::Review {
                targets,
                state: AnnotationReviewState::Accepted,
            }),
            0,
            &governor,
        )
        .unwrap();
    state.leave();
    assert!(state.writing());
    assert!(!request.cancel.is_cancelled());
    assert!(state.request(None, 0, &governor).is_none());
    assert!(state.accept(execute(&mut db, request)));
    assert!(state.saved);
    assert_eq!(state.count(), 0);
    assert!(
        state
            .page
            .as_ref()
            .unwrap()
            .entries()
            .iter()
            .all(|r| r.data().values.review_state == AnnotationReviewState::Accepted)
    );
    state.leave();
    assert!(state.page.is_none());
    drop(state);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn global_failed_writes_preserve_selection_and_draft_and_shutdown_cancels_reads() {
    let governor = MemoryGovernor::default();
    let mut db = library();
    let mut state = GlobalNotes::default();
    let request = state.request(None, 0, &governor).unwrap();
    state.accept(execute(&mut db, request));
    state.selected[0] = true;
    state.tag = "Study".into();
    let targets = state.targets();
    let request = state
        .request(
            Some(AnnotationBatchMutation::AddTag {
                targets,
                tag: state.tag.clone(),
            }),
            0,
            &governor,
        )
        .unwrap();
    state.accept(AnnotationReply {
        document_id: None,
        result: Err(Failure::MemoryBudget),
        cancel: request.cancel.clone(),
    });
    drop(request);
    assert_eq!(state.count(), 1);
    assert_eq!(state.tag, "Study");
    assert!(state.error.is_some());
    state.filters.kind = Some(AnnotationKind::PageComment);
    state.filters.color = Some(AnnotationColor::Violet);
    state.filters.review = Some(AnnotationReviewState::Pending);
    state.filters.days = Some(7);
    let request = state.request(None, 80, &governor).unwrap();
    assert!(request.query.document_id.is_none());
    assert_eq!(request.query.offset, 80);
    assert_eq!(request.query.kind, Some(AnnotationKind::PageComment));
    assert_eq!(request.query.color, Some(AnnotationColor::Violet));
    assert_eq!(request.query.review, Some(AnnotationReviewState::Pending));
    assert!(request.query.updated_after.as_ref().unwrap().ends_with('Z'));
    let date = chrono::DateTime::parse_from_rfc3339(request.query.updated_after.as_ref().unwrap())
        .unwrap();
    assert!(
        (chrono::Utc::now() - date.with_timezone(&chrono::Utc))
            .num_seconds()
            .abs_diff(7 * 24 * 3600)
            < 2
    );
    assert_eq!(state.count(), 0);
    assert!(state.tag.is_empty());
    drop(state);
    assert!(request.cancel.is_cancelled());
    drop(request);
    assert_eq!(governor.usage().total, 0);
}
