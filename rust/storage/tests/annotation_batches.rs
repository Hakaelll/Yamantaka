use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Failure,
    annotations::{batch::AnnotationDraftBatch, repository::*, *},
    memory::*,
    pdf_geometry::NormalizedPageRect,
    pdf_text::{PdfPageText, TextCharacter},
    reader_preferences::PageRotation,
    search::*,
    text_selection::*,
};
use yamantaka_storage::Database;

fn page(governor: &MemoryGovernor, index: u32, missing: bool) -> Arc<PdfPageText> {
    let text = if index == 0 { "café" } else { "study 学习" };
    let region = NormalizedPageRect::new([0.1, 0.2], [0.3, 0.4]).unwrap();
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .unwrap();
    let characters = text
        .char_indices()
        .map(|(index, c)| TextCharacter {
            bytes: index as u32..(index + c.len_utf8()) as u32,
            region: (!missing).then_some(region),
        })
        .collect();
    Arc::new(PdfPageText::from_reserved(index, text.into(), characters, lease).unwrap())
}
fn source() -> AnnotationSource<'static> {
    AnnotationSource {
        document_id: "a",
        fingerprint: "fa",
        rotation: PageRotation::Deg90,
    }
}
fn selection(end: usize) -> TextSelection {
    TextSelection::new(
        SelectionPosition::new(0, 1).unwrap(),
        SelectionPosition::new(1, end).unwrap(),
    )
}
fn batch(governor: &MemoryGovernor, missing: bool) -> AnnotationDraftBatch {
    AnnotationDraftBatch::from_selection(
        source(),
        selection(8),
        &[page(governor, 0, false), page(governor, 1, missing)],
        AnnotationKind::Highlight,
        AnnotationColor::Violet,
        Some("Review study"),
        governor,
    )
    .unwrap()
}
fn database() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',2,1,'now','now','now','fa');").unwrap();
    db
}
fn query() -> AnnotationQuery {
    AnnotationQuery {
        document_id: Some("a".into()),
        ..Default::default()
    }
}
#[test]
fn ordered_unicode_batch_preserves_each_page_and_explicit_partial_geometry() {
    let governor = MemoryGovernor::default();
    let pages = [page(&governor, 0, false), page(&governor, 1, true)];
    let baseline = governor.usage().text;
    let batch = AnnotationDraftBatch::from_selection(
        source(),
        selection(8),
        &pages,
        AnnotationKind::Highlight,
        AnnotationColor::Violet,
        Some("Review study"),
        &governor,
    )
    .unwrap();
    assert_eq!(batch.drafts().len(), 2);
    assert!(batch.requires_page_note_confirmation());
    assert_eq!(
        batch.drafts()[0].values().selected_text.as_deref(),
        Some("afé")
    );
    assert_eq!(
        batch.drafts()[1].values().selected_text.as_deref(),
        Some("study 学习")
    );
    assert_eq!(batch.drafts()[0].values().page_index, 0);
    assert_eq!(batch.drafts()[1].values().page_index, 1);
    assert_eq!(batch.drafts()[0].values().kind, AnnotationKind::Highlight);
    assert_eq!(batch.drafts()[1].values().kind, AnnotationKind::PageComment);
    assert!(batch.drafts()[1].values().rects.is_empty());
    assert_eq!(batch.drafts()[1].values().page_rotation, 90);
    drop(batch);
    assert_eq!(governor.usage().text, baseline);
    for bad in [
        &pages[..1],
        &[pages[1].clone(), pages[0].clone()][..],
        &[pages[0].clone(), pages[0].clone()][..],
    ] {
        assert!(
            AnnotationDraftBatch::from_selection(
                source(),
                selection(8),
                bad,
                AnnotationKind::Highlight,
                AnnotationColor::Violet,
                None,
                &governor
            )
            .is_err()
        );
    }
    assert!(
        AnnotationDraftBatch::from_selection(
            source(),
            selection(9),
            &pages,
            AnnotationKind::Highlight,
            AnnotationColor::Violet,
            None,
            &governor
        )
        .is_err()
    );
    assert_eq!(governor.usage().text, baseline);
    let boundary = AnnotationDraftBatch::from_selection(
        source(),
        selection(0),
        &pages,
        AnnotationKind::Highlight,
        AnnotationColor::Violet,
        None,
        &governor,
    )
    .unwrap();
    assert_eq!(boundary.drafts().len(), 1);
    drop(boundary);
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        text: 1,
        ..Default::default()
    });
    assert!(matches!(
        AnnotationDraftBatch::from_selection(
            source(),
            selection(8),
            &pages,
            AnnotationKind::Highlight,
            AnnotationColor::Violet,
            None,
            &denied
        ),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(denied.usage().text, 0);
    drop(pages);
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn batch_creation_is_atomic_for_late_insert_failure_fallback_and_fts() {
    let governor = MemoryGovernor::default();
    let mut db = database();
    let batch = batch(&governor, true);
    db.execute_batch("CREATE TRIGGER reject_second_page BEFORE INSERT ON annotations WHEN NEW.page_index=1 BEGIN SELECT RAISE(ABORT,'second-page fixture failure'); END;").unwrap();
    let create = |db: &mut Database, confirm| {
        db.change_annotation(
            "a",
            AnnotationChange::CreateBatch {
                batch: &batch,
                confirm_page_note: confirm,
                tags: &["Study".into()],
            },
            &query(),
            IndexAccess::Enabled,
            &governor,
            &Cancellation::default(),
        )
    };
    assert!(create(&mut db, false).is_err());
    assert!(create(&mut db, true).is_err());
    db.execute_batch("CREATE TEMP TABLE batch_assertion(value INTEGER CHECK(value=0)); INSERT INTO batch_assertion SELECT count(*) FROM annotations; INSERT INTO batch_assertion SELECT count(*) FROM tags; INSERT INTO batch_assertion SELECT count(*) FROM annotation_tags; INSERT INTO batch_assertion SELECT count(*) FROM library_search; DROP TABLE batch_assertion;").unwrap();
    db.execute_batch("DROP TRIGGER reject_second_page;")
        .unwrap();
    let saved = create(&mut db, true).unwrap();
    assert_eq!(saved.total, 2);
    assert_eq!(
        saved.entries()[0].data().created_at,
        saved.entries()[1].data().created_at
    );
    for record in saved.entries() {
        assert_eq!(record.data().tags, ["Study"]);
        assert_eq!(record.data().values.note.as_deref(), Some("Review study"));
    }
    let hits = db
        .search(
            &SearchQuery {
                text: "Review".into(),
                kinds: vec![SearchKind::Annotation],
                ..Default::default()
            },
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(hits.len(), 2);
    drop(saved);
    drop(batch);
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn cancelled_or_response_budget_denied_batches_leave_no_partial_notes() {
    let governor = MemoryGovernor::default();
    let mut db = database();
    let batch = batch(&governor, false);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.change_annotation(
            "a",
            AnnotationChange::CreateBatch {
                batch: &batch,
                confirm_page_note: false,
                tags: &[]
            },
            &query(),
            IndexAccess::Disabled,
            &governor,
            &cancel
        ),
        Err(Failure::Cancelled)
    ));
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        text: 6000,
        ..Default::default()
    });
    assert!(matches!(
        db.change_annotation(
            "a",
            AnnotationChange::CreateBatch {
                batch: &batch,
                confirm_page_note: false,
                tags: &[]
            },
            &query(),
            IndexAccess::Enabled,
            &denied,
            &Cancellation::default()
        ),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(
        db.annotations(&query(), &governor, &Cancellation::default())
            .unwrap()
            .total,
        0
    );
    assert_eq!(denied.usage().text, 0);
    drop(batch);
    assert_eq!(governor.usage().text, 0);
}
