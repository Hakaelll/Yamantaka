use super::*;

fn editor() -> MetadataEditor {
    let document:Document=serde_json::from_value(serde_json::json!({"id":"a","path":"a.pdf","file_name":"a.pdf","title":"Current title","author":"Current author","page_count":2,"last_page_index":0,"reading_progress":0.0,"favorite":false,"missing":false,"fingerprint":"fa","thumbnail_path":null})).unwrap();
    MetadataEditor {
        document,
        snapshot: None,
        reference: MetadataReference::default(),
        source_pending: None,
        pending: None,
        title: "Current title".into(),
        author: "Current author".into(),
        error: None,
        confirm_reset: false,
        saved: false,
        analyzing: false,
        analyzed: false,
        source_opened: false,
    }
}
fn snapshot() -> MetadataReview {
    MetadataReview {
        file: yamantaka_domain::metadata_review::MetadataFile {
            path: "a.pdf".into(),
            fingerprint: "fa".into(),
            page_count: 2,
            missing: false,
            size: 1,
            modified: "now".into(),
        },
        title_evidence: None,
        author_evidence: None,
        document: "a".into(),
        revision: "revision-1".into(),
        title: "Current title".into(),
        author: "Current author".into(),
        source_title: Some("Raw title".into()),
        automatic_title: Some("Current title".into()),
        automatic_author: Some("Current author".into()),
        source_author: None,
        title_override: None,
        author_override: None,
        suggested_title: Some("Proposed title".into()),
        suggested_author: None,
    }
}

#[test]
fn opening_evidence_requires_a_saved_draft_and_serializes_editor_operations() {
    use yamantaka_domain::metadata_review::{MetadataEvidence, MetadataEvidenceKind};
    let mut editor = editor();
    let mut review = snapshot();
    review.title_evidence = Some(MetadataEvidence {
        value: "Proposed title".into(),
        page: 1,
        kind: MetadataEvidenceKind::ProminentText,
    });
    let read = editor.read();
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: false,
        cancel: read.cancel,
        result: Ok(review.clone()),
    });
    assert!(editor.source_request(MetadataField::Author).is_none());
    editor.title = "Unsaved title".into();
    assert!(editor.source_request(MetadataField::Title).is_none());
    assert_eq!(editor.title, "Unsaved title");
    editor.title = review.title.clone();
    let request = editor.source_request(MetadataField::Title).unwrap();
    assert_eq!(*request.expected, review);
    assert!(!request.cancel.is_cancelled());
    assert!(editor.source_request(MetadataField::Title).is_none());
    assert!(editor.apply(MetadataMutation::DismissSuggestions).is_none());
    assert!(editor.reanalyze().is_none());
    editor.read();
    assert!(request.cancel.is_cancelled());
    assert!(editor.source_pending.is_none());
}

#[test]
fn closing_editor_cancels_source_and_native_preview_requests() {
    let mut editor = editor();
    let source = Cancellation::default();
    let preview = Cancellation::default();
    editor.source_pending = Some((MetadataField::Title, source.clone()));
    editor.reference.pending = Some(preview.clone());
    drop(editor);
    assert!(source.is_cancelled());
    assert!(preview.is_cancelled());
}

#[test]
fn reanalysis_requires_a_clean_draft_and_retains_ownership_until_its_terminal_reply() {
    let mut editor = editor();
    let read = editor.read();
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: false,
        cancel: read.cancel,
        result: Ok(snapshot()),
    });
    editor.title = "Unsaved title".into();
    assert!(editor.reanalyze().is_none());
    editor.title = "Current title".into();
    let (expected, cancel) = editor.reanalyze().unwrap();
    assert_eq!(*expected, snapshot());
    assert!(editor.analyzing);
    assert!(editor.reanalyze().is_none());
    cancel.cancel();
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: true,
        cancel,
        result: Err(yamantaka_domain::Failure::Cancelled),
    });
    assert!(!editor.analyzing && !editor.analyzed && !editor.saved);
    assert_eq!(editor.title, "Current title");
    let (_, cancel) = editor.reanalyze().unwrap();
    let mut fresh = snapshot();
    fresh.revision = "new analysis".into();
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: true,
        cancel,
        result: Ok(fresh),
    });
    assert!(editor.analyzed && !editor.analyzing && !editor.saved);
}

#[test]
fn obsolete_read_and_wrong_operation_cannot_replace_a_newer_editor_draft() {
    let mut editor = editor();
    let old = editor.read();
    let current = editor.read();
    assert!(old.cancel.is_cancelled());
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: false,
        cancel: old.cancel,
        result: Ok(snapshot()),
    });
    assert!(editor.snapshot.is_none());
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: true,
        cancel: current.cancel.clone(),
        result: Ok(snapshot()),
    });
    assert!(editor.snapshot.is_none());
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: false,
        cancel: current.cancel,
        result: Ok(snapshot()),
    });
    assert!(editor.capture_ready());
    editor.title = "My own title".into();
    let request = editor
        .apply(MetadataMutation::Save(
            ManualMetadata::new(&editor.title, &editor.author).unwrap(),
        ))
        .unwrap();
    assert!(editor.apply(MetadataMutation::ResetOverrides).is_none());
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: true,
        cancel: request.cancel,
        result: Err(yamantaka_domain::Failure::Storage(
            "Synthetic failure".into(),
        )),
    });
    assert_eq!(editor.title, "My own title");
    assert_eq!(editor.author, "Current author");
    assert!(editor.pending.is_none());
    assert!(editor.error.is_some());
    assert!(!editor.saved);
    assert_eq!(editor.snapshot.as_ref().unwrap().title, "Current title");
}

#[test]
fn only_the_matching_successful_save_updates_snapshot_and_confirms_persistence() {
    let mut editor = editor();
    let read = editor.read();
    editor.accept(MetadataReply {
        document: "a".into(),
        writing: false,
        cancel: read.cancel,
        result: Ok(snapshot()),
    });
    let request = editor.apply(MetadataMutation::DismissSuggestions).unwrap();
    let mut saved = snapshot();
    saved.revision = "revision-2".into();
    saved.suggested_title = None;
    editor.accept(MetadataReply {
        document: "another".into(),
        writing: true,
        cancel: request.cancel.clone(),
        result: Ok(saved.clone()),
    });
    assert!(editor.pending.is_some());
    assert_eq!(
        editor.accept(MetadataReply {
            document: "a".into(),
            writing: true,
            cancel: request.cancel,
            result: Ok(saved.clone())
        }),
        Some(saved.clone())
    );
    assert_eq!(editor.snapshot, Some(saved));
    assert!(editor.saved);
}
