use yamantaka_domain::{
    Cancellation, Failure,
    annotations::repository::*,
    annotations::*,
    memory::*,
    pdf_geometry::NormalizedPageRect,
    pdf_text::{PdfPageText, TextCharacter},
    reader_preferences::PageRotation,
    search::*,
    text_selection::*,
};
use yamantaka_storage::Database;

fn library() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    documents(&db);
    db
}
fn documents(db: &Database) {
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',1,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf',1,1,'now','now','now','fb');").unwrap();
}
fn query() -> AnnotationQuery {
    AnnotationQuery {
        document_id: Some("a".into()),
        ..Default::default()
    }
}
fn draft(governor: &MemoryGovernor, missing: bool) -> AnnotationDraft {
    let text = "café study";
    let region = NormalizedPageRect::new([0.1, 0.2], [0.2, 0.3]).unwrap();
    let characters = text
        .char_indices()
        .map(|(index, value)| TextCharacter {
            bytes: index as u32..(index + value.len_utf8()) as u32,
            region: (!missing).then_some(region),
        })
        .collect();
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .unwrap();
    let page = PdfPageText::from_reserved(8, text.into(), characters, lease).unwrap();
    let selection = TextSelection::new(
        SelectionPosition::new(8, 1).unwrap(),
        SelectionPosition::new(8, 10).unwrap(),
    );
    AnnotationDraft::from_selection(
        AnnotationSource {
            document_id: "a",
            fingerprint: "fa",
            rotation: PageRotation::Deg90,
        },
        &selection.page(&page).unwrap().unwrap(),
        AnnotationKind::Highlight,
        AnnotationColor::Violet,
        Some("Étude 学习"),
        governor,
    )
    .unwrap()
}
fn create(
    db: &mut Database,
    governor: &MemoryGovernor,
    missing: bool,
    confirm: bool,
    access: IndexAccess,
) -> yamantaka_domain::Result<AnnotationPage> {
    db.change_annotation(
        "a",
        AnnotationChange::Create {
            draft: &draft(governor, missing),
            confirm_page_note: confirm,
            tags: &["Optics".into(), " optics ".into()],
        },
        &query(),
        access,
        governor,
        &Cancellation::default(),
    )
}
fn hits(db: &Database, text: &str) -> Vec<SearchResult> {
    db.search(
        &SearchQuery {
            text: text.into(),
            kinds: vec![SearchKind::Annotation],
            ..Default::default()
        },
        &Cancellation::default(),
    )
    .unwrap()
}

#[test]
fn create_edit_tags_review_delete_and_snapshot_restore_preserve_identity_and_geometry() {
    let mut db = library();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let initial = create(&mut db, &governor, false, false, IndexAccess::Enabled).unwrap();
    let record = initial.entries()[0].data();
    let id = record.id.clone();
    let created = record.created_at.clone();
    assert_eq!(
        record.values.page_index, 8,
        "metadata page count must not veto a canonical PDF page"
    );
    assert_eq!(record.values.selected_text.as_deref(), Some("afé study"));
    assert_eq!(record.values.context_before.as_deref(), Some("c"));
    assert_eq!(record.values.page_rotation, 90);
    assert_eq!(record.tags, vec!["Optics"]);
    let json = serde_json::to_value(&record.values.rects).unwrap();
    assert_eq!(
        json[0],
        serde_json::json!({"left":0.1,"top":0.2,"right":0.2,"bottom":0.3})
    );
    assert_eq!(hits(&db, "学习").len(), 1);
    db.execute_batch("ALTER TABLE annotations ADD COLUMN extension_note TEXT; ALTER TABLE annotations ADD COLUMN extra_integer INTEGER; ALTER TABLE annotations ADD COLUMN extra_real REAL; ALTER TABLE annotations ADD COLUMN extra_blob BLOB; ALTER TABLE annotations ADD COLUMN extra_null BLOB; UPDATE annotations SET origin_document_id='origin',origin_fingerprint='origin-sha',extension_note='preserve',extra_integer=42,extra_real=3.5,extra_blob=X'00FF10'; CREATE TRIGGER require_annotation_extension BEFORE INSERT ON annotations WHEN NEW.extension_note IS NULL BEGIN SELECT RAISE(ABORT,'Extension required'); END;").unwrap();
    let edited = db
        .change_annotation(
            "a",
            AnnotationChange::Edit {
                id: &id,
                kind: AnnotationKind::Underline,
                color: AnnotationColor::Blue,
                note: Some("  Tomorrow 宇宙  "),
                review: AnnotationReviewState::Accepted,
                tags: None,
            },
            &query(),
            IndexAccess::Enabled,
            &governor,
            &cancel,
        )
        .unwrap();
    let data = edited.entries()[0].data();
    assert_eq!(data.id, id);
    assert_eq!(data.created_at, created);
    assert_eq!(data.values.note.as_deref(), Some("Tomorrow 宇宙"));
    assert_eq!(serde_json::to_value(&data.values.rects).unwrap(), json);
    assert_eq!(data.origin_document_id.as_deref(), Some("origin"));
    assert!(hits(&db, "学习").is_empty());
    assert_eq!(hits(&db, "Tomorrow").len(), 1);
    let ids = vec![id.clone()];
    db.change_annotation(
        "a",
        AnnotationChange::AddTag {
            ids: &ids,
            tag: "Review",
        },
        &query(),
        IndexAccess::Enabled,
        &governor,
        &cancel,
    )
    .unwrap();
    let filtered = db
        .annotations(
            &AnnotationQuery {
                kind: Some(AnnotationKind::Underline),
                color: Some(AnnotationColor::Blue),
                review: Some(AnnotationReviewState::Accepted),
                tag: Some(" REVIEW ".into()),
                text: "Tomorrow".into(),
                ..query()
            },
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(filtered.total, 1);
    db.change_annotation(
        "a",
        AnnotationChange::Review {
            ids: &ids,
            state: AnnotationReviewState::Resolved,
        },
        &query(),
        IndexAccess::Disabled,
        &governor,
        &cancel,
    )
    .unwrap();
    let saved = db.annotations(&query(), &governor, &cancel).unwrap();
    db.change_annotation(
        "a",
        AnnotationChange::Remove { id: &id },
        &query(),
        IndexAccess::Enabled,
        &governor,
        &cancel,
    )
    .unwrap();
    assert!(hits(&db, "Tomorrow").is_empty());
    let restored = db
        .change_annotation(
            "a",
            AnnotationChange::Save {
                record: &saved.entries()[0],
            },
            &query(),
            IndexAccess::Enabled,
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(restored.entries()[0].data().created_at, created);
    assert_eq!(
        restored.entries()[0].data().origin_fingerprint.as_deref(),
        Some("origin-sha")
    );
    assert_eq!(restored.entries()[0].data().tags.len(), 2);
    let extras = &restored.entries()[0].data().extensions;
    assert!(extras.iter().any(|(name, value)| name == "extra_integer"
        && matches!(value, AnnotationColumnValue::Integer(42))));
    assert!(extras.iter().any(|(name, value)| name == "extra_real"
        && matches!(value,AnnotationColumnValue::Real(value) if *value==3.5)));
    assert!(extras.iter().any(|(name, value)| name == "extra_blob"
        && matches!(value,AnnotationColumnValue::Blob(value) if value==&[0,255,16])));
    assert!(
        extras
            .iter()
            .any(|(name, value)| name == "extra_null"
                && matches!(value, AnnotationColumnValue::Null))
    );
    assert!(
        restored.entries()[0]
            .data()
            .extensions
            .iter()
            .any(|(name, value)| name == "extension_note"
                && matches!(value,AnnotationColumnValue::Text(value) if value=="preserve"))
    );
    assert_eq!(
        restored.entries()[0].data().updated_at,
        saved.entries()[0].data().updated_at
    );
    drop(initial);
    drop(edited);
    drop(filtered);
    drop(saved);
    drop(restored);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn fallback_cancellation_wrong_document_and_failed_snapshot_do_not_write() {
    let mut db = library();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    assert!(matches!(
        create(&mut db, &governor, true, false, IndexAccess::Enabled),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(
        db.annotations(&query(), &governor, &cancel).unwrap().total,
        0
    );
    let page = create(&mut db, &governor, true, true, IndexAccess::Disabled).unwrap();
    assert_eq!(
        page.entries()[0].data().values.kind,
        AnnotationKind::PageComment
    );
    assert!(page.entries()[0].data().values.rects.is_empty());
    let id = page.entries()[0].data().id.clone();
    assert!(
        db.change_annotation(
            "b",
            AnnotationChange::Remove { id: &id },
            &query(),
            IndexAccess::Enabled,
            &governor,
            &cancel
        )
        .is_err()
    );
    let stopped = Cancellation::default();
    stopped.cancel();
    assert!(matches!(
        db.change_annotation(
            "a",
            AnnotationChange::Remove { id: &id },
            &query(),
            IndexAccess::Enabled,
            &governor,
            &stopped
        ),
        Err(Failure::Cancelled)
    ));
    db.execute_batch("INSERT INTO annotations(id,document_id,page_index,type,color,rects_json,document_fingerprint,created_at,updated_at) VALUES('broken','a',0,'highlight','yellow','invalid','fa','now','now');").unwrap();
    assert!(create(&mut db, &governor, false, false, IndexAccess::Enabled).is_err());
    db.execute_batch("DELETE FROM annotations WHERE id='broken';")
        .unwrap();
    assert_eq!(
        db.annotations(&query(), &governor, &cancel).unwrap().total,
        1
    );
    assert!(hits(&db, "学习").is_empty());
    drop(page);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn search_consent_exclusion_failure_and_active_index_state_are_independent() {
    let mut db = library();
    db.execute_batch("UPDATE documents SET title='Reviewed title',source_title='Raw processor title' WHERE id='a'").unwrap();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    create(&mut db, &governor, false, false, IndexAccess::Disabled).unwrap();
    assert!(hits(&db, "学习").is_empty());
    let session = db
        .begin_index("a", false, IndexAccess::Enabled, &cancel)
        .unwrap();
    create(&mut db, &governor, false, false, IndexAccess::Enabled).unwrap();
    let indexed = hits(&db, "学习");
    assert!(!indexed.is_empty());
    assert!(indexed.iter().all(|hit| hit.title == "Reviewed title"));
    assert_eq!(
        db.index_state("a").unwrap().unwrap().indexed_pages,
        session.cursor
    );
    assert_eq!(
        db.index_state("a").unwrap().unwrap().updated_at,
        session.revision
    );
    db.exclude_from_index("a", true, &cancel).unwrap();
    create(&mut db, &governor, false, false, IndexAccess::Enabled).unwrap();
    assert!(hits(&db, "学习").is_empty());
    db.execute_batch("ALTER TABLE library_search RENAME TO unavailable_search;")
        .unwrap();
    assert!(create(&mut db, &governor, false, false, IndexAccess::Enabled).is_err());
    assert_eq!(
        db.annotations(&query(), &governor, &cancel).unwrap().total,
        3
    );
    create(&mut db, &governor, false, false, IndexAccess::Disabled).unwrap();
    assert_eq!(
        db.annotations(&query(), &governor, &cancel).unwrap().total,
        4
    );
}

#[test]
fn all_historical_schemas_support_annotations_after_migration_and_reopen_exactly() {
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    for version in 1..=11 {
        let mut db = Database::in_memory_at(version).unwrap();
        documents(&db);
        if version >= 2 {
            db.execute_batch("INSERT INTO annotations(id,document_id,page_index,type,color,note,selected_text,rects_json,page_rotation,document_fingerprint,created_at,updated_at) VALUES('legacy','a',0,'textNote','rose','Legacy 学习','Old passage','[{\"left\":0.1,\"top\":0.2,\"right\":0.3,\"bottom\":0.4}]',270,'fa','2020-01-02T03:04:05Z','2020-01-02T03:04:05Z');").unwrap();
        }
        db.migrate_current().unwrap();
        let page = create(&mut db, &governor, false, false, IndexAccess::Disabled).unwrap();
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("annotation-store.sqlite3");
        db.snapshot(&path).unwrap();
        let reopened = Database::open(&path).unwrap();
        let copy = reopened.annotations(&query(), &governor, &cancel).unwrap();
        assert_eq!(copy.entries()[0].data().id, page.entries()[0].data().id);
        assert_eq!(
            copy.entries()[0].data().created_at,
            page.entries()[0].data().created_at
        );
        assert_eq!(
            serde_json::to_value(&copy.entries()[0].data().values).unwrap(),
            serde_json::to_value(&page.entries()[0].data().values).unwrap()
        );
        assert_eq!(copy.entries()[0].data().tags, page.entries()[0].data().tags);
        if version >= 2 {
            let legacy = copy
                .entries()
                .iter()
                .find(|record| record.data().id == "legacy")
                .unwrap()
                .data();
            assert_eq!(legacy.values.kind, AnnotationKind::TextNote);
            assert_eq!(legacy.values.note.as_deref(), Some("Legacy 学习"));
            assert_eq!(legacy.values.page_rotation, 270);
            assert_eq!(legacy.created_at, "2020-01-02T03:04:05Z");
            assert_eq!(legacy.values.review_state, AnnotationReviewState::Pending);
        }
        assert_eq!(reopened.schema_version().unwrap(), 11);
    }
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn paged_filters_literal_patterns_and_budget_failure_are_bounded_and_atomic() {
    let mut db = library();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    db.execute_batch("WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<89) INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) SELECT printf('id-%03d',x),'a',x%3,'pageComment','blue',CASE WHEN x=5 THEN 'literal %_ study' ELSE 'Other' END,'fa','2020','2020' FROM n;").unwrap();
    db.execute_batch("UPDATE annotations SET updated_at='2020-01-01T00:00:00Z' WHERE id='id-000'; UPDATE annotations SET updated_at='2020-02-01T00:00:00Z' WHERE id='id-001'; UPDATE annotations SET updated_at='2020-03-01T00:00:00Z' WHERE id='id-002';").unwrap();
    assert_eq!(
        db.annotations(
            &AnnotationQuery {
                updated_after: Some("2020-02-01T00:00:00Z".into()),
                updated_before: Some("2020-03-01T00:00:00Z".into()),
                ..query()
            },
            &governor,
            &cancel
        )
        .unwrap()
        .total,
        1
    );
    assert_eq!(
        db.annotations(&AnnotationQuery::default(), &governor, &cancel)
            .unwrap()
            .entries()[0]
            .data()
            .id,
        "id-002"
    );
    let first = db.annotations(&query(), &governor, &cancel).unwrap();
    assert_eq!(first.entries().len(), 80);
    assert_eq!(first.total, 90);
    let last = db
        .annotations(
            &AnnotationQuery {
                offset: 80,
                ..query()
            },
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(last.entries().len(), 10);
    assert!(
        db.annotations(
            &AnnotationQuery {
                offset: usize::MAX,
                ..query()
            },
            &governor,
            &cancel
        )
        .unwrap()
        .entries()
        .is_empty()
    );
    assert_eq!(
        db.annotations(
            &AnnotationQuery {
                page: Some(2),
                text: "%_".into(),
                ..query()
            },
            &governor,
            &cancel
        )
        .unwrap()
        .total,
        1
    );
    let blocked = MemoryGovernor::new(GlobalMemoryLimits {
        text: 1,
        ..Default::default()
    });
    assert!(matches!(
        db.change_annotation(
            "a",
            AnnotationChange::Edit {
                id: "id-005",
                kind: AnnotationKind::PageComment,
                color: AnnotationColor::Blue,
                note: Some("Must roll back"),
                review: AnnotationReviewState::Accepted,
                tags: None,
            },
            &query(),
            IndexAccess::Disabled,
            &blocked,
            &cancel
        ),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(
        db.annotations(
            &AnnotationQuery {
                text: "%_".into(),
                ..query()
            },
            &governor,
            &cancel
        )
        .unwrap()
        .total,
        1
    );
    assert_eq!(blocked.usage().text, 0);
    drop(first);
    drop(last);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn malformed_records_and_cross_document_batches_never_normalize_or_partially_commit() {
    let mut db = library();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    db.execute_batch("INSERT INTO annotations(id,document_id,page_index,type,color,document_fingerprint,created_at,updated_at) VALUES('a-note','a',0,'pageComment','yellow','fa','original','original'),('b-note','b',0,'pageComment','yellow','fb','original','original');").unwrap();
    let ids = vec!["a-note".into(), "b-note".into()];
    assert!(
        db.change_annotation(
            "a",
            AnnotationChange::Review {
                ids: &ids,
                state: AnnotationReviewState::Accepted
            },
            &query(),
            IndexAccess::Disabled,
            &governor,
            &cancel
        )
        .is_err()
    );
    let page = db.annotations(&query(), &governor, &cancel).unwrap();
    assert_eq!(page.entries()[0].data().updated_at, "original");
    assert_eq!(
        page.entries()[0].data().values.review_state,
        AnnotationReviewState::Pending
    );
    drop(page);
    for invalid in [
        "[{\"left\":-0.1,\"top\":0.2,\"right\":0.3,\"bottom\":0.4}]",
        "[{\"left\":0.1,\"top\":0.2,\"right\":0.3,\"bottom\":0.4,\"future\":1}]",
    ] {
        db.execute_batch(&format!(
            "UPDATE annotations SET rects_json='{invalid}' WHERE id='a-note';"
        ))
        .unwrap();
        assert!(matches!(
            db.annotations(&query(), &governor, &cancel),
            Err(Failure::Invalid(_))
        ));
        assert_eq!(governor.usage().text, 0);
    }
    db.execute_batch(
        "UPDATE annotations SET rects_json='[]',type='unsupported-value' WHERE id='a-note';",
    )
    .unwrap();
    assert!(matches!(
        db.annotations(&query(), &governor, &cancel),
        Err(Failure::Invalid(_))
    ));
    db.execute_batch("UPDATE annotations SET type='pageComment' WHERE id='a-note';")
        .unwrap();
    assert_eq!(
        db.annotations(&query(), &governor, &cancel).unwrap().total,
        1
    );
}

#[test]
fn version_nine_backup_restores_annotations_and_attachment_review_stays_independent() {
    let mut db = library();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let initial = create(&mut db, &governor, false, false, IndexAccess::Disabled).unwrap();
    let ids = vec![initial.entries()[0].data().id.clone()];
    db.change_annotation(
        "a",
        AnnotationChange::Review {
            ids: &ids,
            state: AnnotationReviewState::Accepted,
        },
        &query(),
        IndexAccess::Disabled,
        &governor,
        &cancel,
    )
    .unwrap();
    let changed = db
        .change_annotation(
            "a",
            AnnotationChange::CheckAttachment {
                fingerprint: "changed",
            },
            &query(),
            IndexAccess::Disabled,
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        changed.entries()[0].data().values.status,
        AttachmentStatus::ReviewRequired
    );
    assert_eq!(
        changed.entries()[0].data().values.review_state,
        AnnotationReviewState::Accepted
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("annotations-v9.json");
    db.export_backup(
        &path,
        &yamantaka_storage::preferences::Preferences::default(),
        &cancel,
    )
    .unwrap();
    let preview = yamantaka_storage::backup::preview(&path, &cancel).unwrap();
    assert_eq!(preview.backup_version, 9);
    let restored = preview
        .database
        .annotations(&query(), &governor, &cancel)
        .unwrap();
    assert_eq!(restored.entries()[0].data().id, ids[0]);
    assert_eq!(
        restored.entries()[0].data().tags,
        changed.entries()[0].data().tags
    );
    assert_eq!(
        serde_json::to_value(&restored.entries()[0].data().values).unwrap(),
        serde_json::to_value(&changed.entries()[0].data().values).unwrap()
    );
    let reattached = db
        .change_annotation(
            "a",
            AnnotationChange::CheckAttachment { fingerprint: "fa" },
            &query(),
            IndexAccess::Disabled,
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        reattached.entries()[0].data().values.status,
        AttachmentStatus::Attached
    );
    assert_eq!(
        reattached.entries()[0].data().values.review_state,
        AnnotationReviewState::Accepted
    );
    drop(initial);
    drop(changed);
    drop(restored);
    drop(reattached);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn editing_tags_replaces_or_clears_links_and_rolls_back_with_the_note_and_search() {
    let mut db = library();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let original = create(&mut db, &governor, false, false, IndexAccess::Enabled).unwrap();
    let data = original.entries()[0].data();
    let id = data.id.clone();
    let geometry = serde_json::to_value(&data.values.rects).unwrap();
    let tags = vec!["Study".to_string(), "étude".to_string()];
    let changed = db
        .change_annotation(
            "a",
            AnnotationChange::Edit {
                id: &id,
                kind: AnnotationKind::Underline,
                color: AnnotationColor::Blue,
                note: Some("New note"),
                review: AnnotationReviewState::Accepted,
                tags: Some(&tags),
            },
            &query(),
            IndexAccess::Enabled,
            &governor,
            &cancel,
        )
        .unwrap();
    let row = changed.entries()[0].data();
    assert_eq!(row.tags, vec!["Study", "étude"]);
    assert_eq!(serde_json::to_value(&row.values.rects).unwrap(), geometry);
    assert_eq!(hits(&db, "étude").len(), 1);
    assert!(hits(&db, "Optics").is_empty());
    db.execute_batch("CREATE TRIGGER reject_new_tag BEFORE INSERT ON annotation_tags WHEN (SELECT name FROM tags WHERE id=NEW.tag_id)='Reject' BEGIN SELECT RAISE(ABORT,'Synthetic rejection'); END;").unwrap();
    let rejected = vec!["Reject".into()];
    assert!(
        db.change_annotation(
            "a",
            AnnotationChange::Edit {
                id: &id,
                kind: AnnotationKind::Highlight,
                color: AnnotationColor::Yellow,
                note: Some("Must roll back"),
                review: AnnotationReviewState::Resolved,
                tags: Some(&rejected)
            },
            &query(),
            IndexAccess::Enabled,
            &governor,
            &cancel
        )
        .is_err()
    );
    let after = db.annotations(&query(), &governor, &cancel).unwrap();
    assert_eq!(after.entries()[0].data().tags, row.tags);
    assert_eq!(
        after.entries()[0].data().values.note.as_deref(),
        Some("New note")
    );
    assert_eq!(hits(&db, "étude").len(), 1);
    assert!(hits(&db, "Reject").is_empty());
    let cleared = db
        .change_annotation(
            "a",
            AnnotationChange::Edit {
                id: &id,
                kind: AnnotationKind::Underline,
                color: AnnotationColor::Blue,
                note: Some("New note"),
                review: AnnotationReviewState::Accepted,
                tags: Some(&[]),
            },
            &query(),
            IndexAccess::Enabled,
            &governor,
            &cancel,
        )
        .unwrap();
    assert!(cleared.entries()[0].data().tags.is_empty());
    assert!(hits(&db, "étude").is_empty());
    drop(original);
    drop(changed);
    drop(after);
    drop(cleared);
    assert_eq!(governor.usage().text, 0);
}
