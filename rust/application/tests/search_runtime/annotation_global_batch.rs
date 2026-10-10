use super::*;
use yamantaka_application::annotations::*;
use yamantaka_domain::{
    Failure,
    annotations::{AnnotationReviewState, repository::*},
    memory::MemoryGovernor,
};

#[test]
fn cross_document_annotation_batches_keep_request_identity_during_held_pdf_work() {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf','A',3,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf','B',3,1,'now','now','now','fb');
        INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) VALUES('a1','a',0,'pageComment','violet','Study','fa','original','original'),('b1','b',0,'pageComment','violet','Étude','fb','original','original');").unwrap();
    let (runtime, native, notified) = start_database(db);
    start(&runtime);
    let Native::Open(_, _, held) = next(&native) else {
        panic!("Expected held index open")
    };
    let governor = MemoryGovernor::default();
    let targets = vec![
        AnnotationTarget {
            document_id: "a".into(),
            annotation_id: "a1".into(),
        },
        AnnotationTarget {
            document_id: "b".into(),
            annotation_id: "b1".into(),
        },
    ];
    for mutation in [
        AnnotationBatchMutation::Review {
            targets: targets.clone(),
            state: AnnotationReviewState::Accepted,
        },
        AnnotationBatchMutation::AddTag {
            targets: targets.clone(),
            tag: "Study".into(),
        },
    ] {
        let cancel = Cancellation::default();
        runtime
            .submit(Action::Annotation(Box::new(AnnotationRequest {
                operation: AnnotationOperation::Batch { mutation },
                query: Default::default(),
                governor: governor.clone(),
                cancel: cancel.clone(),
            })))
            .unwrap();
        let Update::Annotations(reply) = update(&runtime) else {
            panic!("Expected batch reply")
        };
        assert!(reply.document_id.is_none());
        assert!(reply.cancel.same_request(&cancel));
        let page = reply.result.unwrap();
        assert_eq!(page.total, 2);
        assert!(
            page.entries()
                .iter()
                .all(|row| row.data().values.review_state == AnnotationReviewState::Accepted)
        );
        assert!(!cancel.is_cancelled());
    }
    let cancel = Cancellation::default();
    cancel.cancel();
    runtime
        .submit(Action::Annotation(Box::new(AnnotationRequest {
            operation: AnnotationOperation::Batch {
                mutation: AnnotationBatchMutation::Review {
                    targets,
                    state: AnnotationReviewState::Resolved,
                },
            },
            query: Default::default(),
            governor: governor.clone(),
            cancel: cancel.clone(),
        })))
        .unwrap();
    let Update::Annotations(reply) = update(&runtime) else {
        panic!("Expected cancelled batch reply")
    };
    assert!(reply.document_id.is_none());
    assert!(reply.cancel.same_request(&cancel));
    assert!(matches!(reply.result, Err(Failure::Cancelled)));
    runtime
        .submit(Action::Annotation(Box::new(AnnotationRequest {
            operation: AnnotationOperation::Read,
            query: Default::default(),
            governor: governor.clone(),
            cancel: Cancellation::default(),
        })))
        .unwrap();
    let Update::Annotations(reply) = update(&runtime) else {
        panic!("Expected global read")
    };
    let page = reply.result.unwrap();
    for row in page.entries() {
        assert_eq!(
            row.data().values.review_state,
            AnnotationReviewState::Accepted
        );
        assert_eq!(row.data().tags, ["Study"]);
    }
    drop(page);
    assert_eq!(governor.usage().total, 0);
    assert!(
        native.try_recv().is_err(),
        "SQL batches must not request additional PDF work"
    );
    runtime.submit(Action::Index(IndexCommand::Cancel)).unwrap();
    committed(&runtime, WriteKind::Index);
    held.send(Err(Failure::Cancelled)).unwrap();
    let Native::Close(_, reply) = next(&native) else {
        panic!("Expected index cleanup")
    };
    reply.send(Ok(())).unwrap();
    phase(&runtime, &notified, IndexPhase::Cancelled);
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}
