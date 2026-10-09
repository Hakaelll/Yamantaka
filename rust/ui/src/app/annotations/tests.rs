use super::*;
use yamantaka_domain::{
    memory::*,
    pdf_geometry::NormalizedPageRect,
    pdf_text::{PdfPageText, TextCharacter},
    text_selection::{SelectionPosition, TextSelection},
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
            snapshot: TextSelectionSnapshot::new(
                TextSelection::new(
                    SelectionPosition::new(0, 0).unwrap(),
                    SelectionPosition::new(0, 4).unwrap(),
                ),
                [Ok(page)],
                governor,
            )
            .unwrap(),
            fingerprint: "fa".into(),
            rotation: PageRotation::Deg90,
        },
        kind: AnnotationKind::Highlight,
        color: AnnotationColor::Violet,
        note: "Study".into(),
        tags: String::new(),
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
            let change = match &mutation {
                AnnotationMutation::Create {
                    draft,
                    confirm_page_note,
                    tags,
                } => AnnotationChange::Create {
                    draft,
                    confirm_page_note: *confirm_page_note,
                    tags,
                },
                AnnotationMutation::CreateBatch {
                    batch,
                    confirm_page_note,
                    tags,
                } => AnnotationChange::CreateBatch {
                    batch,
                    confirm_page_note: *confirm_page_note,
                    tags,
                },
                AnnotationMutation::Edit {
                    id,
                    kind,
                    color,
                    note,
                    review,
                    tags,
                } => AnnotationChange::Edit {
                    id,
                    kind: *kind,
                    color: *color,
                    note: note.as_deref(),
                    review: *review,
                    tags: tags.as_deref(),
                },
                _ => panic!("create fixture"),
            };
            db.change_annotation(
                &document_id,
                change,
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

#[test]
fn editor_saves_a_frozen_multi_page_passage_with_per_page_fallback() {
    let governor = MemoryGovernor::default();
    let mut db = database();
    let mut first = editor(&governor, false);
    let second = editor(&governor, true);
    let EditorSource::Selection {
        snapshot: first_source,
        ..
    } = &first.source
    else {
        panic!("selection")
    };
    let EditorSource::Selection {
        snapshot: second_source,
        ..
    } = second.source
    else {
        panic!("selection")
    };
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .unwrap();
    let page = Arc::new(
        PdfPageText::from_reserved(
            1,
            "café".into(),
            second_source.pages()[0]
                .characters()
                .iter()
                .map(|character| TextCharacter {
                    bytes: character.bytes.clone(),
                    region: character.region,
                })
                .collect(),
            lease,
        )
        .unwrap(),
    );
    let range = TextSelection::new(
        SelectionPosition::new(0, 1).unwrap(),
        SelectionPosition::new(1, 4).unwrap(),
    );
    let snapshot = TextSelectionSnapshot::new(
        range,
        [Ok(first_source.pages()[0].clone()), Ok(page)],
        &governor,
    )
    .unwrap();
    assert_eq!(snapshot.text(), "afé\ncafé");
    first.source = EditorSource::Selection {
        snapshot,
        fingerprint: "fa".into(),
        rotation: PageRotation::Deg90,
    };
    drop(second_source);
    let mut state = AnnotationState::default();
    state.bind("a");
    state.editor = Some(first);
    assert!(state.fallback());
    assert!(state.mutation(&governor).is_err());
    state.editor.as_mut().unwrap().confirmed = true;
    let mutation = state.mutation(&governor).unwrap();
    let request = state.request(&governor, Some(mutation), 0).unwrap();
    state.accept(execute(&mut db, request));
    let page = state.page.as_ref().unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(
        page.entries()[0].data().values.selected_text.as_deref(),
        Some("afé")
    );
    assert_eq!(
        page.entries()[1].data().values.kind,
        AnnotationKind::PageComment
    );
    assert!(page.entries()[1].data().values.rects.is_empty());
    drop(state);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn page_note_editor_saves_without_a_text_snapshot_and_rejects_blank_notes() {
    let governor = MemoryGovernor::default();
    let mut db = database();
    let mut state = AnnotationState::default();
    state.bind("a");
    state.editor = Some(AnnotationEditor {
        source: EditorSource::PageNote {
            page: 0,
            fingerprint: "fa".into(),
            rotation: PageRotation::Deg270,
        },
        kind: AnnotationKind::PageComment,
        color: AnnotationColor::Blue,
        note: "  ".into(),
        tags: String::new(),
        review: AnnotationReviewState::Pending,
        confirmed: false,
    });
    assert!(!state.fallback());
    assert_eq!(state.editor.as_ref().unwrap().passage(), "");
    assert!(state.mutation(&governor).is_err());
    state.editor.as_mut().unwrap().note = "  Étude de cette page  ".into();
    let change = state.mutation(&governor).unwrap();
    let request = state.request(&governor, Some(change), 0).unwrap();
    state.accept(execute(&mut db, request));
    assert!(state.saved && state.editor.is_none());
    let values = &state.page.as_ref().unwrap().entries()[0].data().values;
    assert_eq!(values.kind, AnnotationKind::PageComment);
    assert_eq!(values.page_rotation, 270);
    assert_eq!(values.note.as_deref(), Some("Étude de cette page"));
    assert!(
        values.selected_text.is_none()
            && values.context_before.is_none()
            && values.context_after.is_none()
            && values.rects.is_empty()
    );
    drop(state);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn tag_editor_validates_unicode_limits_and_preserves_replaces_or_clears_stored_tags() {
    let governor = MemoryGovernor::default();
    let mut db = database();
    let mut state = AnnotationState::default();
    state.bind("a");
    let mut draft = editor(&governor, false);
    draft.tags = "  Étude, optics, ÉTUDE, , optics ".into();
    assert_eq!(draft.tag_changes().unwrap().unwrap(), ["Étude", "optics"]);
    draft.tags = "𐐷".repeat(81);
    assert!(draft.tag_changes().is_err());
    draft.tags = (0..13)
        .map(|n| format!("t{n}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(draft.tag_changes().is_err());
    draft.tags = "Étude, optics".into();
    state.editor = Some(draft);
    let request = state
        .request(&governor, Some(state.mutation(&governor).unwrap()), 0)
        .unwrap();
    state.accept(execute(&mut db, request));
    db.execute_batch("UPDATE tags SET name='group,one' WHERE normalized_name='optics';")
        .unwrap();
    let request = state.request(&governor, None, 0).unwrap();
    state.accept(execute(&mut db, request));
    let page = state.page.as_ref().unwrap().clone();
    let data = page.entries()[0].data();
    let geometry = serde_json::to_value(&data.values.rects).unwrap();
    state.editor = Some(AnnotationEditor {
        source: EditorSource::Stored {
            page: page.clone(),
            index: 0,
        },
        kind: data.values.kind,
        color: data.values.color,
        note: "Edited note".into(),
        tags: data.tags.join(", "),
        review: data.values.review_state,
        confirmed: false,
    });
    assert!(
        state
            .editor
            .as_ref()
            .unwrap()
            .tag_changes()
            .unwrap()
            .is_none()
    );
    let request = state
        .request(&governor, Some(state.mutation(&governor).unwrap()), 0)
        .unwrap();
    state.accept(execute(&mut db, request));
    assert_eq!(
        state.page.as_ref().unwrap().entries()[0].data().tags,
        data.tags
    );
    state.editor = Some(AnnotationEditor {
        source: EditorSource::Stored {
            page: page.clone(),
            index: 0,
        },
        kind: data.values.kind,
        color: data.values.color,
        note: "Edited note".into(),
        tags: data.tags.join(", "),
        review: data.values.review_state,
        confirmed: false,
    });
    state.editor.as_mut().unwrap().tags = "Memory, mémoire".into();
    let request = state
        .request(&governor, Some(state.mutation(&governor).unwrap()), 0)
        .unwrap();
    state.accept(execute(&mut db, request));
    let updated = state.page.as_ref().unwrap().clone();
    let data = updated.entries()[0].data();
    assert_eq!(data.tags, ["Memory", "mémoire"]);
    assert_eq!(serde_json::to_value(&data.values.rects).unwrap(), geometry);
    state.editor = Some(AnnotationEditor {
        source: EditorSource::Stored {
            page: updated.clone(),
            index: 0,
        },
        kind: data.values.kind,
        color: data.values.color,
        note: "Edited note".into(),
        tags: String::new(),
        review: data.values.review_state,
        confirmed: false,
    });
    let request = state
        .request(&governor, Some(state.mutation(&governor).unwrap()), 0)
        .unwrap();
    state.accept(execute(&mut db, request));
    assert!(
        state.page.as_ref().unwrap().entries()[0]
            .data()
            .tags
            .is_empty()
    );
    drop(page);
    drop(updated);
    drop(state);
    assert_eq!(governor.usage().text, 0);
}
