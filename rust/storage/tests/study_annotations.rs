use yamantaka_domain::{
    Cancellation,
    annotations::{repository::*, study::StudyGeometry, *},
    memory::{GlobalMemoryLimits, MemoryGovernor},
    reader_preferences::PageRotation,
    search::IndexAccess,
};
use yamantaka_storage::{Database, backup};

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
    assert_eq!(mark.drawing().unwrap().points(), &points);
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
