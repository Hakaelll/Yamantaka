use yamantaka_domain::{
    Cancellation, LibraryQuery, LibraryRepository,
    search::{IndexAccess, SearchKind, SearchQuery, SearchRepository},
    tags::*,
};
use yamantaka_storage::Database;
fn tag(db: &Database, id: &str) -> Tag {
    db.tags(
        &TagQuery {
            id: Some(id.into()),
            document: Some("d".into()),
            ..Default::default()
        },
        &Cancellation::default(),
    )
    .unwrap()
    .entries
    .remove(0)
}
fn results(db: &Database, text: &str, kind: SearchKind) -> usize {
    db.search(
        &SearchQuery {
            text: text.into(),
            document_id: Some("d".into()),
            kinds: vec![kind],
            ..Default::default()
        },
        &Cancellation::default(),
    )
    .unwrap()
    .len()
}
pub fn verify(db: &mut Database) {
    let cancel = Cancellation::default();
    let original = db.get("original").unwrap().unwrap();
    db.execute_batch("INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('legacy-shared','Shared','shared','violet','2026-10-19','2026-10-19'); INSERT INTO annotation_tags(annotation_id,tag_id) VALUES('smart-a','legacy-shared');").unwrap();
    let shared = db
        .change_tag(
            &TagChange::Ensure {
                name: " SHARED ".into(),
            },
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert_eq!(shared, "legacy-shared");
    let row = tag(db, &shared);
    assert_eq!(
        (
            row.name.as_str(),
            row.color.as_str(),
            row.document_count,
            row.annotation_count,
            row.contains_document
        ),
        ("Shared", "violet", 0, 1, false)
    );
    for name in [String::new(), "x".repeat(61), "😀".repeat(31)] {
        assert!(
            db.change_tag(&TagChange::Ensure { name }, IndexAccess::Disabled, &cancel)
                .is_err()
        );
    }
    let local = db
        .change_tag(
            &TagChange::Ensure {
                name: "Local 100%_\\".into(),
            },
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        db.change_tag(
            &TagChange::Ensure {
                name: "local 100%_\\".into()
            },
            IndexAccess::Disabled,
            &cancel
        )
        .unwrap(),
        local
    );
    let literal = db
        .tags(
            &TagQuery {
                text: "%_\\".into(),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(
        (literal.total, literal.entries[0].id.as_str()),
        (1, local.as_str())
    );
    let membership = |id: &str, included| TagChange::SetMembership {
        tag: id.into(),
        document: "d".into(),
        included,
    };
    db.change_tag(&membership(&shared, true), IndexAccess::Disabled, &cancel)
        .unwrap();
    db.change_tag(&membership(&shared, true), IndexAccess::Disabled, &cancel)
        .unwrap();
    assert_eq!(tag(db, &shared).document_count, 1);
    assert_eq!(db.index_stats().unwrap().rows, 0);
    let filtered = LibraryQuery {
        tag_ids: vec![shared.clone()],
        ..Default::default()
    };
    assert_eq!(db.count(&filtered).unwrap(), 1);
    assert_eq!(db.query(&filtered).unwrap()[0].id, "d");

    let mut session = db
        .begin_index("d", false, IndexAccess::Enabled, &cancel)
        .unwrap();
    for page in 0..session.page_count {
        db.index_page(
            &mut session,
            page,
            "Synthetic indexed passage",
            IndexAccess::Enabled,
            &cancel,
        )
        .unwrap();
    }
    db.finish_index(&mut session, IndexAccess::Enabled, &cancel)
        .unwrap();
    db.refresh_search_entries(IndexAccess::Enabled, &cancel)
        .unwrap();
    let indexed = db.index_state("d").unwrap();
    assert_eq!(results(db, "Shared", SearchKind::PageText), 3);
    assert_eq!(results(db, "Shared", SearchKind::Annotation), 1);
    db.change_tag(&membership(&local, true), IndexAccess::Enabled, &cancel)
        .unwrap();
    assert_eq!(results(db, "Local", SearchKind::PageText), 3);
    assert_eq!(results(db, "Local", SearchKind::Tag), 1);
    assert_eq!(db.index_state("d").unwrap(), indexed);
    db.change_tag(&membership(&shared, false), IndexAccess::Disabled, &cancel)
        .unwrap();
    assert_eq!(results(db, "Shared", SearchKind::PageText), 0);
    assert_eq!(results(db, "Shared", SearchKind::Tag), 0);
    assert_eq!(results(db, "Shared", SearchKind::Annotation), 1);
    let row = tag(db, &shared);
    assert_eq!(
        (
            row.document_count,
            row.annotation_count,
            row.contains_document
        ),
        (0, 1, false)
    );
    assert_eq!(db.count(&filtered).unwrap(), 0);
    assert_eq!(db.index_state("d").unwrap(), indexed);
    let stop = Cancellation::default();
    stop.cancel();
    assert!(
        db.change_tag(&membership(&local, false), IndexAccess::Enabled, &stop)
            .is_err()
    );
    assert!(tag(db, &local).contains_document);
    assert!(
        db.change_tag(
            &membership("no-such-tag", true),
            IndexAccess::Enabled,
            &cancel
        )
        .is_err()
    );
    assert!(
        db.change_tag(
            &TagChange::SetMembership {
                tag: local.clone(),
                document: "no-such-document".into(),
                included: true
            },
            IndexAccess::Enabled,
            &cancel
        )
        .is_err()
    );

    db.exclude_from_index("d", true, &cancel).unwrap();
    let excluded = db
        .change_tag(
            &TagChange::Ensure {
                name: "Excluded".into(),
            },
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    db.change_tag(&membership(&excluded, true), IndexAccess::Enabled, &cancel)
        .unwrap();
    assert_eq!(results(db, "Excluded", SearchKind::Tag), 0);
    assert!(tag(db, &excluded).contains_document);
    for i in 0..81 {
        db.change_tag(
            &TagChange::Ensure {
                name: format!("Tag {i:03}"),
            },
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    }
    let first = db.tags(&TagQuery::default(), &cancel).unwrap();
    let second = db
        .tags(
            &TagQuery {
                offset: 80,
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(
        (first.total, first.entries.len(), second.entries.len()),
        (84, 80, 4)
    );
    assert!(
        first
            .entries
            .iter()
            .all(|left| second.entries.iter().all(|right| left.id != right.id))
    );
    let last = second.entries.last().unwrap();
    assert_eq!(tag(db, &last.id).id, last.id);
    let after = db.get("original").unwrap().unwrap();
    assert_eq!(
        (
            after.id,
            after.title,
            after.path,
            after.last_page_index,
            after.reading_progress
        ),
        (
            original.id,
            original.title,
            original.path,
            original.last_page_index,
            original.reading_progress
        )
    );
    db.integrity().unwrap();
    db.exclude_from_index("d", false, &cancel).unwrap();
    db.execute_batch("DROP TABLE library_search").unwrap();
    assert!(
        db.change_tag(&membership(&local, false), IndexAccess::Enabled, &cancel)
            .is_err()
    );
    assert!(
        tag(db, &local).contains_document,
        "A failed search-cache update must roll back the tag change"
    );
}
