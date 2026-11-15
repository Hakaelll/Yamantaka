use yamantaka_domain::{
    Cancellation, LibraryRepository,
    annotations::{repository::*, study::StudyGeometry, *},
    memory::{GlobalMemoryLimits, MemoryGovernor},
    reader_preferences::PageRotation,
    search::IndexAccess,
};
use yamantaka_storage::{Database, backup};

#[test]
fn book_summary_and_review_schedule_survive_v9_restore() {
    use yamantaka_domain::study_book::{BookStudy, Card, Operation, Section, Source};
    let cancel = Cancellation::default();
    let fixture = serde_json::from_str(include_str!("fixtures/populated-backup-v9.json")).unwrap();
    let backup::RestorePreview {
        mut database,
        preferences,
        ..
    } = backup::preview_payload(fixture, &cancel).unwrap();
    let source = Source {
        annotation: "annotation".into(),
        page: 2,
        point: [0.4, 0.2],
    };
    let book = BookStudy {
        sections: vec![Section {
            title: "Optics".into(),
            text: "Read the figure".into(),
            source: Some(source.clone()),
        }],
        cards: vec![Card {
            question: "What changes?".into(),
            answer: "The angle".into(),
            source: Some(source),
            due: Some(1_800_000_000),
            interval: 2,
        }],
        ..BookStudy::default()
    };
    database
        .study_book("original", Operation::Save(book.clone()))
        .unwrap();
    assert!(
        database
            .study_book("original", Operation::Save(book))
            .is_err()
    );
    let directory = tempfile::tempdir().unwrap();
    let exported = directory.path().join("summary.md");
    database
        .study_book("original", Operation::Export(exported.clone()))
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(exported).unwrap(),
        "## Optics\n\nRead the figure\n\n> PDF · p. 3\n\n"
    );
    let path = directory.path().join("study.json");
    database
        .export_backup(&path, &preferences, &cancel)
        .unwrap();
    let mut restored = backup::preview(&path, &cancel).unwrap().database;
    let loaded = restored.study_book("original", Operation::Load).unwrap();
    assert_eq!(loaded.sections[0].title, "Optics");
    assert_eq!(loaded.cards[0].due, Some(1_800_000_000));
    assert_eq!(loaded.cards[0].interval, 2);
    assert_eq!(loaded.cards[0].source.as_ref().unwrap().point, [0.4, 0.2]);
}

#[test]
fn study_geometry_round_trips_new_shapes_and_preserves_legacy_width() {
    use yamantaka_domain::annotations::study::StrokeWeight;
    let legacy = r#"{"version":1,"kind":"ink","points":[[0.1,0.2],[0.8,0.9]]}"#;
    let old = StudyGeometry::decode(AnnotationKind::Ink, legacy).unwrap();
    assert_eq!(old.weight(), StrokeWeight::Medium);
    for kind in [
        AnnotationKind::Ellipse,
        AnnotationKind::Line,
        AnnotationKind::Crop,
    ] {
        let points = vec![[0.12, 0.34], [0.56, 0.78]];
        let geometry = StudyGeometry::new(kind, points.clone())
            .unwrap()
            .with_weight(StrokeWeight::Thick);
        let encoded = serde_json::to_string(&geometry).unwrap();
        let restored = StudyGeometry::decode(kind, &encoded).unwrap();
        assert_eq!(restored.points(), &points);
        assert_eq!(restored.weight(), StrokeWeight::Thick);
        assert!(StudyGeometry::decode(AnnotationKind::Arrow, &encoded).is_err());
    }
}

#[test]
fn pending_review_loads_one_note_and_can_undo_resolution() {
    let cancel = Cancellation::default();
    let governor = MemoryGovernor::new(GlobalMemoryLimits::default());
    let fixture = serde_json::from_str(include_str!("fixtures/populated-backup-v9.json")).unwrap();
    let mut database = backup::preview_payload(fixture, &cancel).unwrap().database;
    database
        .execute_batch(
            "UPDATE annotations SET review_state='pending' WHERE id='annotation';
         INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,
           document_fingerprint,status,created_at,updated_at,review_state)
         VALUES('second','original',3,'pageComment','violet','Follow-up','[]',0,
           'legacy','attached','2026-11-08','2026-11-08','pending');",
        )
        .unwrap();
    let query = AnnotationQuery {
        document_id: Some("original".into()),
        review: Some(AnnotationReviewState::Pending),
        limit: Some(std::num::NonZeroUsize::MIN),
        ..Default::default()
    };
    let first = database.annotations(&query, &governor, &cancel).unwrap();
    assert_eq!(first.total, 2);
    assert_eq!(first.entries().len(), 1);
    assert_eq!(first.entries()[0].data().id, "annotation");
    let next = database
        .change_annotation(
            "original",
            AnnotationChange::Review {
                ids: &["annotation".into()],
                state: AnnotationReviewState::Resolved,
            },
            &query,
            IndexAccess::Disabled,
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(next.total, 1);
    assert_eq!(next.entries()[0].data().id, "second");
    let restored = database
        .change_annotation(
            "original",
            AnnotationChange::Undo,
            &query,
            IndexAccess::Disabled,
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(restored.total, 2);
    assert_eq!(restored.entries()[0].data().id, "annotation");
}

#[test]
fn drawing_survives_history_and_v9_backup() {
    let cancel = Cancellation::default();
    let governor = MemoryGovernor::new(GlobalMemoryLimits::default());
    let fixture = serde_json::from_str(include_str!("fixtures/populated-backup-v9.json")).unwrap();
    let backup::RestorePreview {
        mut database,
        preferences,
        ..
    } = backup::preview_payload(fixture, &cancel).unwrap();
    let points = [[0.1, 0.2], [0.7, 0.8]];
    let draft = AnnotationDraft::drawing(
        AnnotationSource {
            document_id: "original",
            fingerprint: "legacy",
            rotation: PageRotation::Deg90,
        },
        2,
        AnnotationColor::Blue,
        StudyGeometry::new(AnnotationKind::Arrow, points.to_vec()).unwrap(),
        &governor,
    )
    .unwrap();
    let query = AnnotationQuery {
        document_id: Some("original".into()),
        kind: Some(AnnotationKind::Arrow),
        ..Default::default()
    };
    let change = |database: &mut Database, change| {
        database
            .change_annotation(
                "original",
                change,
                &query,
                IndexAccess::Disabled,
                &governor,
                &cancel,
            )
            .unwrap()
    };
    let created = change(
        &mut database,
        AnnotationChange::Create {
            draft: &draft,
            confirm_page_note: false,
            tags: &[],
        },
    );
    let id = created.entries()[0].data().id.clone();
    assert!(
        database
            .change_annotation(
                "original",
                AnnotationChange::EraseMark { id: "annotation" },
                &query,
                IndexAccess::Disabled,
                &governor,
                &cancel
            )
            .is_err()
    );
    assert_eq!(
        change(&mut database, AnnotationChange::EraseMark { id: &id }).total,
        0
    );
    assert_eq!(
        change(&mut database, AnnotationChange::Undo).entries()[0]
            .data()
            .id,
        id
    );
    assert_eq!(change(&mut database, AnnotationChange::Redo).total, 0);
    assert_eq!(
        change(&mut database, AnnotationChange::Undo).entries()[0]
            .data()
            .id,
        id
    );
    assert_eq!(change(&mut database, AnnotationChange::Undo).total, 0);
    assert_eq!(
        change(&mut database, AnnotationChange::Redo).entries()[0]
            .data()
            .id,
        id
    );
    let latest = database.annotations(&query, &governor, &cancel).unwrap();
    let revision = latest.entries()[0].revision();
    let moved = StudyGeometry::new(AnnotationKind::Arrow, points.to_vec())
        .unwrap()
        .with_bounds([[0.2, 0.3], [0.8, 0.9]])
        .unwrap()
        .with_weight(yamantaka_domain::annotations::study::StrokeWeight::Thick);
    change(
        &mut database,
        AnnotationChange::Edit {
            id: &id,
            expected: revision,
            drawing: Some(&moved),
            kind: AnnotationKind::Arrow,
            color: AnnotationColor::Blue,
            note: None,
            review: AnnotationReviewState::Pending,
            tags: None,
        },
    );
    change(&mut database, AnnotationChange::Undo);
    change(&mut database, AnnotationChange::Redo);
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("study.json");
    database
        .export_backup(&file, &preferences, &cancel)
        .unwrap();
    let restored = backup::preview(&file, &cancel).unwrap();
    assert_eq!(restored.backup_version, 9);
    let marks = restored
        .database
        .annotation_marks("original", 2, &governor, &cancel)
        .unwrap();
    let mark = marks.marks().iter().find(|mark| mark.id() == id).unwrap();
    assert_eq!(mark.drawing(), Some(&moved));
    assert_eq!(mark.kind(), AnnotationKind::Arrow);
    assert!(!mark.has_note());
    assert!(
        marks
            .marks()
            .iter()
            .any(|mark| mark.kind() == AnnotationKind::Highlight && mark.has_note())
    );
    let exact = restored
        .database
        .annotations(
            &AnnotationQuery {
                document_id: Some("original".into()),
                annotation_id: Some(id.clone()),
                ..Default::default()
            },
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(exact.total, 1);
    assert_eq!(exact.entries()[0].data().id, id);
    assert!(StudyGeometry::new(AnnotationKind::Ink, vec![[0.1, 0.2], [f64::NAN, 0.1]]).is_err());
    assert!(StudyGeometry::new(AnnotationKind::Rectangle, vec![[0.1, 0.2], [0.1, 0.8]]).is_err());
}
