use super::*;
use yamantaka_domain::{
    memory::*, pdf_geometry::NormalizedPageRect, pdf_text::TextCharacter,
    text_selection::SelectionPosition,
};
use yamantaka_storage::Database;

fn editor(governor: &MemoryGovernor, missing: bool) -> AnnotationEditor {
    let text = "café";
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .unwrap();
    let region = NormalizedPageRect::new([0.1, 0.2], [0.3, 0.4]).unwrap();
    let characters = text
        .char_indices()
        .map(|(index, value)| TextCharacter {
            bytes: index as u32..(index + value.len_utf8()) as u32,
            region: (!missing).then_some(region),
        })
        .collect();
    let page = Arc::new(PdfPageText::from_reserved(0, text.into(), characters, lease).unwrap());
    AnnotationEditor {
        source: EditorSource::Selection {
            page,
            range: TextSelection::new(
                SelectionPosition::new(0, 0).unwrap(),
                SelectionPosition::new(0, 4).unwrap(),
            ),
            fingerprint: "fa".into(),
            rotation: PageRotation::Deg90,
        },
        kind: AnnotationKind::Highlight,
        color: AnnotationColor::Violet,
        note: "Study".into(),
        review: AnnotationReviewState::Pending,
        confirmed: false,
    }
}
fn database() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',1,1,'now','now','now','fa');").unwrap();
    db
}
fn execute(db: &mut Database, request: AnnotationRequest) -> AnnotationReply {
    let document_id = request.query.document_id.clone();
    let result = match request.operation {
        AnnotationOperation::Read => {
            db.annotations(&request.query, &request.governor, &request.cancel)
        }
        AnnotationOperation::Change {
            document_id,
            mutation,
        } => {
            let AnnotationMutation::Create {
                draft,
                confirm_page_note,
                tags,
            } = mutation
            else {
                panic!("create fixture")
            };
            db.change_annotation(
                &document_id,
                AnnotationChange::Create {
                    draft: &draft,
                    confirm_page_note,
                    tags: &tags,
                },
                &request.query,
                yamantaka_domain::search::IndexAccess::Disabled,
                &request.governor,
                &request.cancel,
            )
        }
    };
    AnnotationReply {
        document_id,
        result,
        cancel: request.cancel,
    }
}
#[test]
fn editor_freezes_passage_validates_fallback_and_releases_saved_payloads() {
    for missing in [false, true] {
        let governor = MemoryGovernor::default();
        let mut db = database();
        let mut state = AnnotationState::default();
        state.bind("a");
        state.editor = Some(editor(&governor, missing));
        assert_eq!(state.editor.as_ref().unwrap().passage(), "café");
        assert_eq!(state.fallback(), missing);
        if missing {
            assert!(state.mutation(&governor).is_err());
            state.editor.as_mut().unwrap().confirmed = true;
        }
        let mutation = state.mutation(&governor).unwrap();
        let request = state.request(&governor, Some(mutation), 0).unwrap();
        assert!(state.request(&governor, None, 0).is_none());
        let reply = execute(&mut db, request);
        state.accept(reply);
        assert!(state.saved);
        assert!(state.editor.is_none());
        let data = state.page.as_ref().unwrap().entries()[0].data();
        assert_eq!(data.values.selected_text.as_deref(), Some("café"));
        assert_eq!(data.values.page_rotation, 90);
        assert_eq!(
            data.values.kind,
            if missing {
                AnnotationKind::PageComment
            } else {
                AnnotationKind::Highlight
            }
        );
        drop(state);
        assert_eq!(governor.usage().text, 0);
    }
}
#[test]
fn reader_switch_cancels_reads_preserves_accepted_writes_and_reports_departed_failures() {
    let governor = MemoryGovernor::default();
    let mut state = AnnotationState::default();
    state.bind("a");
    let read = state.request(&governor, None, 0).unwrap();
    state.bind("b");
    assert!(read.cancel.is_cancelled());
    state.bind("a");
    state.editor = Some(editor(&governor, false));
    let mutation = state.mutation(&governor).unwrap();
    let write = state.request(&governor, Some(mutation), 0).unwrap();
    state.bind("b");
    assert!(!write.cancel.is_cancelled());
    assert!(state.editor.is_none());
    let token = write.cancel.clone();
    drop(write);
    let error = state.accept(AnnotationReply {
        document_id: Some("a".into()),
        result: Err(Failure::Storage("fixture failure".into())),
        cancel: token,
    });
    assert!(error.is_some());
    assert!(state.pending.is_none());
    assert!(state.page.is_none());
    drop(state);
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn stale_replies_and_failed_save_preserve_the_current_editor_for_retry() {
    let governor = MemoryGovernor::default();
    let mut state = AnnotationState::default();
    state.bind("a");
    state.editor = Some(editor(&governor, false));
    state.editor.as_mut().unwrap().note = "𐐷".repeat(2001);
    assert!(state.mutation(&governor).is_err());
    state.editor.as_mut().unwrap().note = "Retry this passage".into();
    let mutation = state.mutation(&governor).unwrap();
    let request = state.request(&governor, Some(mutation), 0).unwrap();
    state.accept(AnnotationReply {
        document_id: Some("a".into()),
        result: Err(Failure::Cancelled),
        cancel: Cancellation::default(),
    });
    assert!(state.pending.is_some());
    let token = request.cancel.clone();
    drop(request);
    state.accept(AnnotationReply {
        document_id: Some("wrong".into()),
        result: Err(Failure::Cancelled),
        cancel: token.clone(),
    });
    assert!(state.pending.is_some());
    state.accept(AnnotationReply {
        document_id: Some("a".into()),
        result: Err(Failure::MemoryBudget),
        cancel: token,
    });
    assert!(state.pending.is_none());
    assert!(state.editor.is_some());
    assert!(state.error.is_some());
    let retry = state
        .request(&governor, Some(state.mutation(&governor).unwrap()), 0)
        .unwrap();
    drop(retry);
    drop(state);
    assert_eq!(governor.usage().text, 0);
}
