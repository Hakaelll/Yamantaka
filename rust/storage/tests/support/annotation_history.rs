use yamantaka_domain::{
    Cancellation,
    annotations::{repository::*, *},
    memory::{GlobalMemoryLimits, MemoryGovernor},
    reader_preferences::PageRotation,
    search::{IndexAccess, SearchKind, SearchQuery, SearchRepository},
};
use yamantaka_storage::Database;

fn change(
    db: &mut Database,
    change: AnnotationChange<'_>,
    governor: &MemoryGovernor,
) -> yamantaka_domain::Result<AnnotationPage> {
    db.change_annotation(
        "original",
        change,
        &AnnotationQuery {
            document_id: Some("original".into()),
            ..Default::default()
        },
        IndexAccess::Enabled,
        governor,
        &Cancellation::default(),
    )
}
fn found(db: &Database, text: &str) -> usize {
    db.search(
        &SearchQuery {
            text: text.into(),
            document_id: Some("original".into()),
            kinds: vec![SearchKind::Annotation],
            ..Default::default()
        },
        &Cancellation::default(),
    )
    .unwrap()
    .len()
}
fn revision(
    db: &Database,
    governor: &MemoryGovernor,
) -> yamantaka_domain::annotations::revision::AnnotationRevision {
    db.annotations(
        &AnnotationQuery {
            document_id: Some("original".into()),
            ..Default::default()
        },
        governor,
        &Cancellation::default(),
    )
    .unwrap()
    .entries()[0]
        .revision()
        .clone()
}
pub fn verify(document: &str) {
    let mut db = Database::in_memory_at(11).unwrap();
    db.execute_batch(document).unwrap();
    let governor = MemoryGovernor::new(GlobalMemoryLimits::default());
    let draft = AnnotationDraft::page_note(
        AnnotationSource {
            document_id: "original",
            fingerprint: "legacy",
            rotation: PageRotation::Deg0,
        },
        2,
        AnnotationColor::Violet,
        Some("initialword"),
        &governor,
    )
    .unwrap();
    let page = change(
        &mut db,
        AnnotationChange::Create {
            draft: &draft,
            confirm_page_note: false,
            tags: &["Original tag".into()],
        },
        &governor,
    )
    .unwrap();
    let id = page.entries()[0].data().id.clone();
    assert_eq!(
        (page.total, page.history.undo, page.history.redo),
        (1, 1, 0)
    );
    assert_eq!(found(&db, "initialword"), 1);
    drop(page);
    let page = change(&mut db, AnnotationChange::Undo, &governor).unwrap();
    assert_eq!(
        (page.total, page.history.undo, page.history.redo),
        (0, 0, 1)
    );
    assert_eq!(found(&db, "initialword"), 0);
    drop(page);
    let page = change(&mut db, AnnotationChange::Redo, &governor).unwrap();
    assert_eq!(page.entries()[0].data().id, id);
    assert_eq!(page.entries()[0].data().tags, ["Original tag"]);
    assert_eq!(found(&db, "initialword"), 1);
    drop(page);
    db.execute_batch("ALTER TABLE annotations ADD COLUMN inherited_blob BLOB; UPDATE annotations SET inherited_blob=X'001122FF',rects_json='[ {\"left\":0.1,\"top\":0.2,\"right\":0.4,\"bottom\":0.3} ]',origin_document_id='inherited';").unwrap();
    let expected = revision(&db, &governor);
    db.execute_batch("CREATE TEMP TABLE annotation_title_fixture AS SELECT title_override FROM documents WHERE id='original'; UPDATE documents SET title_override='Changed catalogue title' WHERE id='original';").unwrap();
    assert_eq!(revision(&db, &governor), expected);
    db.execute_batch("UPDATE documents SET title_override=(SELECT title_override FROM annotation_title_fixture) WHERE id='original'; DROP TABLE annotation_title_fixture;")
        .unwrap();
    let edit = || AnnotationChange::Edit {
        id: &id,
        expected: &expected,
        kind: AnnotationKind::PageComment,
        color: AnnotationColor::Green,
        note: Some("editedword"),
        review: AnnotationReviewState::Accepted,
        tags: Some(&[]),
    };
    let page = change(&mut db, edit(), &governor).unwrap();
    assert_eq!(
        page.entries()[0].data().values.note.as_deref(),
        Some("editedword")
    );
    assert_eq!(found(&db, "initialword"), 0);
    let history = page.history;
    drop(page);
    assert!(matches!(
        change(&mut db, edit(), &governor),
        Err(yamantaka_domain::Failure::StaleAnnotation)
    ));
    let unchanged = db
        .annotations(
            &AnnotationQuery {
                document_id: Some("original".into()),
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(
        unchanged.entries()[0].data().values.note.as_deref(),
        Some("editedword")
    );
    assert_eq!(
        (unchanged.history.undo, unchanged.history.redo),
        (history.undo, history.redo)
    );
    assert_eq!(found(&db, "editedword"), 1);
    drop(unchanged);
    let before_blob = revision(&db, &governor);
    let editor_source = db
        .annotation_for_edit("original", &id, &governor, &Cancellation::default())
        .unwrap()
        .unwrap();
    assert_eq!(editor_source.revision(), &before_blob);
    drop(editor_source);
    assert!(
        db.annotation_for_edit(
            "different-document",
            &id,
            &governor,
            &Cancellation::default()
        )
        .unwrap()
        .is_none()
    );
    db.execute_batch("UPDATE annotations SET inherited_blob=X'10';")
        .unwrap();
    assert!(matches!(
        change(
            &mut db,
            AnnotationChange::Edit {
                id: &id,
                expected: &before_blob,
                kind: AnnotationKind::PageComment,
                color: AnnotationColor::Green,
                note: Some("lostoverwrite"),
                review: AnnotationReviewState::Accepted,
                tags: None
            },
            &governor
        ),
        Err(yamantaka_domain::Failure::StaleAnnotation)
    ));
    assert_eq!(found(&db, "lostoverwrite"), 0);
    db.execute_batch("UPDATE annotations SET inherited_blob=X'001122FF';")
        .unwrap();
    let page = change(&mut db, AnnotationChange::Undo, &governor).unwrap();
    let record = page.entries()[0].data();
    assert_eq!(record.values.note.as_deref(), Some("initialword"));
    assert_eq!(record.tags, ["Original tag"]);
    assert_eq!(record.origin_document_id.as_deref(), Some("inherited"));
    assert!(
        record
            .extensions
            .iter()
            .any(|(name, value)| name == "inherited_blob"
                && matches!(value,AnnotationColumnValue::Blob(bytes) if bytes == &[0,17,34,255]))
    );
    drop(page);
    db.execute_batch("CREATE TEMP TRIGGER reject_history BEFORE UPDATE ON annotations BEGIN SELECT RAISE(ABORT,'synthetic history failure'); END;").unwrap();
    assert!(change(&mut db, AnnotationChange::Redo, &governor).is_err());
    db.execute_batch("DROP TRIGGER reject_history;").unwrap();
    let page = change(&mut db, AnnotationChange::Redo, &governor).unwrap();
    assert_eq!(
        page.entries()[0].data().values.note.as_deref(),
        Some("editedword")
    );
    drop(page);
    let page = change(&mut db, AnnotationChange::Remove { id: &id }, &governor).unwrap();
    assert_eq!(page.total, 0);
    drop(page);
    let page = change(&mut db, AnnotationChange::Undo, &governor).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(found(&db, "editedword"), 1);
    drop(page);
    // A new edit branches history and must not replay the old deletion.
    drop(
        change(
            &mut db,
            AnnotationChange::Review {
                ids: std::slice::from_ref(&id),
                state: AnnotationReviewState::Resolved,
            },
            &governor,
        )
        .unwrap(),
    );
    assert!(change(&mut db, AnnotationChange::Redo, &governor).is_err());
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(
        db.change_annotation(
            "original",
            AnnotationChange::Undo,
            &AnnotationQuery::default(),
            IndexAccess::Enabled,
            &governor,
            &cancelled
        )
        .is_err()
    );
    let page = change(&mut db, AnnotationChange::Undo, &governor).unwrap();
    assert_eq!(
        page.entries()[0].data().values.review_state,
        AnnotationReviewState::Accepted
    );
    drop(page);
    let before_external = revision(&db, &governor);
    db.execute_batch("UPDATE annotations SET note='externalword'")
        .unwrap();
    assert!(matches!(
        change(
            &mut db,
            AnnotationChange::Edit {
                id: &id,
                expected: &before_external,
                kind: AnnotationKind::PageComment,
                color: AnnotationColor::Violet,
                note: Some("staleoverwrite"),
                review: AnnotationReviewState::Pending,
                tags: None
            },
            &governor
        ),
        Err(yamantaka_domain::Failure::StaleAnnotation)
    ));
    assert_eq!(found(&db, "staleoverwrite"), 0);
    assert!(change(&mut db, AnnotationChange::Undo, &governor).is_err());
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("history.sqlite3");
    db.snapshot(&path).unwrap();
    let raw = rusqlite::Connection::open(&path).unwrap();
    let (geometry, blob, note): (String, Vec<u8>, String) = raw
        .query_row(
            "SELECT rects_json,inherited_blob,note FROM annotations",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        geometry,
        "[ {\"left\":0.1,\"top\":0.2,\"right\":0.4,\"bottom\":0.3} ]"
    );
    assert_eq!(blob, [0, 17, 34, 255]);
    assert_eq!(note, "externalword");
    for index in 0..105 {
        let expected = revision(&db, &governor);
        drop(
            change(
                &mut db,
                AnnotationChange::Edit {
                    id: &id,
                    expected: &expected,
                    kind: AnnotationKind::PageComment,
                    color: AnnotationColor::Violet,
                    note: Some(&format!("step{index}")),
                    review: AnnotationReviewState::Pending,
                    tags: None,
                },
                &governor,
            )
            .unwrap(),
        );
    }
    let page = db
        .annotations(
            &AnnotationQuery {
                document_id: Some("original".into()),
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!((page.history.undo, page.history.redo), (100, 0));
    drop(page);
    for _ in 0..99 {
        drop(change(&mut db, AnnotationChange::Undo, &governor).unwrap());
    }
    let page = change(&mut db, AnnotationChange::Undo, &governor).unwrap();
    assert_eq!(
        page.entries()[0].data().values.note.as_deref(),
        Some("step4")
    );
    assert_eq!((page.history.undo, page.history.redo), (0, 100));
    drop(page);
    assert!(change(&mut db, AnnotationChange::Undo, &governor).is_err());
    db.integrity().unwrap();
    drop(draft);
    drop(db);
    assert_eq!(governor.snapshot().usage.total, 0);
}
