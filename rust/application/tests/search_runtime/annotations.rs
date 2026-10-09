use super::*;
use yamantaka_application::annotations::*;
use yamantaka_domain::{
    Failure,
    annotations::{repository::*, *},
    memory::*,
    pdf_geometry::NormalizedPageRect,
    pdf_text::{PdfPageText, TextCharacter},
    reader_preferences::PageRotation,
    text_selection::{SelectionPosition, TextSelection},
};

fn draft(governor: &MemoryGovernor, missing: bool) -> Arc<AnnotationDraft> {
    let text = "café";
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .unwrap();
    let region = NormalizedPageRect::new([0.1, 0.2], [0.2, 0.3]).unwrap();
    let characters = text
        .char_indices()
        .map(|(index, value)| TextCharacter {
            bytes: index as u32..(index + value.len_utf8()) as u32,
            region: (!missing).then_some(region),
        })
        .collect();
    let page = PdfPageText::from_reserved(2, text.into(), characters, lease).unwrap();
    let selection = TextSelection::new(
        SelectionPosition::new(2, 0).unwrap(),
        SelectionPosition::new(2, 4).unwrap(),
    );
    Arc::new(
        AnnotationDraft::from_selection(
            AnnotationSource {
                document_id: "a",
                fingerprint: "fa",
                rotation: PageRotation::Deg90,
            },
            &selection.page(&page).unwrap().unwrap(),
            AnnotationKind::Highlight,
            AnnotationColor::Violet,
            Some("Study passage"),
            governor,
        )
        .unwrap(),
    )
}
fn request(
    runtime: &Runtime,
    governor: &MemoryGovernor,
    mutation: Option<AnnotationMutation>,
    token: &Cancellation,
) -> AnnotationReply {
    let operation = mutation.map_or(AnnotationOperation::Read, |mutation| {
        AnnotationOperation::Change {
            document_id: "a".into(),
            mutation,
        }
    });
    runtime
        .submit(Action::Annotation(Box::new(AnnotationRequest {
            operation,
            query: AnnotationQuery {
                document_id: Some("a".into()),
                ..Default::default()
            },
            governor: governor.clone(),
            cancel: token.clone(),
        })))
        .unwrap();
    let Update::Annotations(reply) = update(runtime) else {
        panic!("annotation response")
    };
    assert_eq!(reply.document_id.as_deref(), Some("a"));
    assert!(reply.cancel.same_request(token));
    reply
}
fn change(
    runtime: &Runtime,
    governor: &MemoryGovernor,
    mutation: AnnotationMutation,
) -> AnnotationPage {
    request(runtime, governor, Some(mutation), &Cancellation::default())
        .result
        .unwrap()
}
#[test]
fn owned_annotation_commands_preserve_snapshots_and_work_during_held_pdf_indexing() {
    let (runtime, native, notified) = setup();
    start(&runtime);
    let Native::Open(_, _, held) = next(&native) else {
        panic!("held native operation")
    };
    let governor = MemoryGovernor::default();
    let source = draft(&governor, false);
    let page = change(
        &runtime,
        &governor,
        AnnotationMutation::Create {
            draft: source.clone(),
            confirm_page_note: false,
            tags: vec!["Study".into()],
        },
    );
    assert_eq!(page.total, 1);
    let original = Arc::new(page.into_entries().pop().unwrap());
    let id = original.data().id.clone();
    assert_eq!(
        original.data().values.selected_text.as_deref(),
        Some("café")
    );
    assert_eq!(Arc::strong_count(&source), 1);
    let changed = change(
        &runtime,
        &governor,
        AnnotationMutation::Edit {
            id: id.clone(),
            kind: AnnotationKind::Underline,
            color: AnnotationColor::Green,
            note: Some("Résumé".into()),
            review: AnnotationReviewState::Accepted,
        },
    );
    assert_eq!(
        changed.entries()[0].data().values.note.as_deref(),
        Some("Résumé")
    );
    drop(changed);
    runtime
        .submit(Action::Search(
            SearchQuery {
                text: "Résumé".into(),
                kinds: vec![yamantaka_domain::search::SearchKind::Annotation],
                ..Default::default()
            },
            Cancellation::default(),
        ))
        .unwrap();
    assert!(
        matches!(update(&runtime),Update::Search(results,_) if results.len()==1 && results[0].page==Some(2))
    );
    let tagged = change(
        &runtime,
        &governor,
        AnnotationMutation::AddTag {
            ids: vec![id.clone()],
            tag: "Optics".into(),
        },
    );
    assert!(
        tagged.entries()[0]
            .data()
            .tags
            .iter()
            .any(|tag| tag == "Optics")
    );
    drop(tagged);
    let reviewed = change(
        &runtime,
        &governor,
        AnnotationMutation::Review {
            ids: vec![id.clone()],
            state: AnnotationReviewState::Resolved,
        },
    );
    assert_eq!(
        reviewed.entries()[0].data().values.review_state,
        AnnotationReviewState::Resolved
    );
    drop(reviewed);
    let detached = change(
        &runtime,
        &governor,
        AnnotationMutation::CheckAttachment {
            fingerprint: "changed".into(),
        },
    );
    assert_eq!(
        detached.entries()[0].data().values.status,
        AttachmentStatus::ReviewRequired
    );
    drop(detached);
    assert_eq!(
        change(
            &runtime,
            &governor,
            AnnotationMutation::Remove { id: id.clone() }
        )
        .total,
        0
    );
    let restored = change(
        &runtime,
        &governor,
        AnnotationMutation::Save {
            record: original.clone(),
        },
    );
    let data = restored.entries()[0].data();
    assert_eq!(data.id, id);
    assert_eq!(data.created_at, original.data().created_at);
    assert_eq!(data.updated_at, original.data().updated_at);
    assert_eq!(data.values.note, original.data().values.note);
    assert_eq!(data.tags, original.data().tags);
    drop(restored);
    query_barrier(&runtime);
    assert!(native.try_recv().is_err());
    drop((source, original));
    assert_eq!(governor.usage().text, 0);
    runtime.submit(Action::Index(IndexCommand::Cancel)).unwrap();
    committed(&runtime, WriteKind::Index);
    held.send(Err(Failure::Cancelled)).unwrap();
    let Native::Close(_, reply) = next(&native) else {
        panic!("index cleanup")
    };
    reply.send(Ok(())).unwrap();
    phase(&runtime, &notified, IndexPhase::Cancelled);
    drop(runtime);
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn annotation_failures_keep_request_identity_and_never_commit_cancelled_or_unconfirmed_drafts() {
    let (runtime, _, _) = setup();
    let governor = MemoryGovernor::default();
    let source = draft(&governor, true);
    let token = Cancellation::default();
    let reply = request(
        &runtime,
        &governor,
        Some(AnnotationMutation::Create {
            draft: source.clone(),
            confirm_page_note: false,
            tags: vec![],
        }),
        &token,
    );
    assert!(matches!(reply.result, Err(Failure::Invalid(_))));
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        request(
            &runtime,
            &governor,
            Some(AnnotationMutation::Create {
                draft: source.clone(),
                confirm_page_note: true,
                tags: vec![]
            }),
            &cancelled
        )
        .result,
        Err(Failure::Cancelled)
    ));
    let page = request(&runtime, &governor, None, &token).result.unwrap();
    assert_eq!(page.total, 0);
    drop(page);
    let page = change(
        &runtime,
        &governor,
        AnnotationMutation::Create {
            draft: source.clone(),
            confirm_page_note: true,
            tags: vec![],
        },
    );
    assert_eq!(
        page.entries()[0].data().values.kind,
        AnnotationKind::PageComment
    );
    drop(page);
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        text: 1,
        ..Default::default()
    });
    assert!(matches!(
        request(&runtime, &denied, None, &token).result,
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(denied.usage().text, 0);
    drop(source);
    drop(runtime);
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn annotation_response_saturation_is_bounded_and_releases_governed_pages_on_shutdown() {
    let (runtime, _, _) = setup();
    let governor = MemoryGovernor::default();
    drop(change(
        &runtime,
        &governor,
        AnnotationMutation::Create {
            draft: draft(&governor, false),
            confirm_page_note: false,
            tags: vec![],
        },
    ));
    for _ in 0..128 {
        runtime
            .submit(Action::Annotation(Box::new(AnnotationRequest {
                operation: AnnotationOperation::Read,
                query: AnnotationQuery {
                    document_id: Some("a".into()),
                    ..Default::default()
                },
                governor: governor.clone(),
                cancel: Cancellation::default(),
            })))
            .unwrap();
    }
    assert!(matches!(
        runtime.submit(Action::Query(Default::default(), Cancellation::default())),
        Err(Failure::QueueFull)
    ));
    let Update::Annotations(reply) = update(&runtime) else {
        panic!("bounded response")
    };
    assert_eq!(reply.result.unwrap().total, 1);
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    drop(runtime);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn annotation_batch_command_keeps_both_page_records_and_releases_shared_drafts() {
    let (runtime, native, _) = setup();
    let governor = MemoryGovernor::default();
    let pages: Vec<_> = (0..2)
        .map(|index| {
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
                .map(|(index, c)| TextCharacter {
                    bytes: index as u32..(index + c.len_utf8()) as u32,
                    region: Some(region),
                })
                .collect();
            Arc::new(PdfPageText::from_reserved(index, text.into(), characters, lease).unwrap())
        })
        .collect();
    let selection = TextSelection::new(
        SelectionPosition::new(0, 1).unwrap(),
        SelectionPosition::new(1, 4).unwrap(),
    );
    let batch = Arc::new(
        yamantaka_domain::annotations::batch::AnnotationDraftBatch::from_selection(
            AnnotationSource {
                document_id: "a",
                fingerprint: "fa",
                rotation: PageRotation::Deg0,
            },
            selection,
            &pages,
            AnnotationKind::Underline,
            AnnotationColor::Green,
            Some("Batch study"),
            &governor,
        )
        .unwrap(),
    );
    drop(pages);
    let saved = change(
        &runtime,
        &governor,
        AnnotationMutation::CreateBatch {
            batch: batch.clone(),
            confirm_page_note: false,
            tags: vec!["Study".into()],
        },
    );
    assert_eq!(saved.total, 2);
    assert_eq!(saved.entries()[0].data().values.page_index, 0);
    assert_eq!(saved.entries()[1].data().values.page_index, 1);
    assert_eq!(
        saved.entries()[0].data().values.selected_text.as_deref(),
        Some("afé")
    );
    assert_eq!(
        saved.entries()[1].data().values.selected_text.as_deref(),
        Some("café")
    );
    assert_eq!(saved.entries()[0].data().tags, ["Study"]);
    assert_eq!(Arc::strong_count(&batch), 1);
    assert!(native.try_recv().is_err());
    drop(saved);
    drop(batch);
    drop(runtime);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn page_note_actor_saves_and_indexes_the_note_without_text_extraction() {
    let (runtime, native, _) = setup();
    let governor = MemoryGovernor::default();
    runtime
        .submit(Action::Index(IndexCommand::Enable(true)))
        .unwrap();
    committed(&runtime, WriteKind::Index);
    let draft = Arc::new(
        AnnotationDraft::page_note(
            AnnotationSource {
                document_id: "a",
                fingerprint: "fa",
                rotation: PageRotation::Deg0,
            },
            2,
            AnnotationColor::Yellow,
            Some("Scanned study observation"),
            &governor,
        )
        .unwrap(),
    );
    let saved = change(
        &runtime,
        &governor,
        AnnotationMutation::Create {
            draft: draft.clone(),
            confirm_page_note: false,
            tags: vec![],
        },
    );
    assert_eq!(saved.total, 1);
    let values = &saved.entries()[0].data().values;
    assert_eq!(values.kind, AnnotationKind::PageComment);
    assert!(values.rects.is_empty() && values.selected_text.is_none());
    runtime
        .submit(Action::Search(
            SearchQuery {
                text: "observation".into(),
                kinds: vec![yamantaka_domain::search::SearchKind::Annotation],
                ..Default::default()
            },
            Cancellation::default(),
        ))
        .unwrap();
    assert!(
        matches!(update(&runtime),Update::Search(results,_) if results.len()==1 && results[0].page==Some(2))
    );
    assert!(native.try_recv().is_err());
    assert_eq!(Arc::strong_count(&draft), 1);
    drop(saved);
    drop(draft);
    drop(runtime);
    assert_eq!(governor.usage().text, 0);
}
