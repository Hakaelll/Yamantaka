use yamantaka_domain::{
    Cancellation, LibraryRepository,
    relations::*,
    search::{IndexAccess, SearchKind, SearchQuery, SearchRepository},
};
use yamantaka_storage::Database;
fn page(db: &Database, document: &str) -> RelationPage {
    db.relations(
        &RelationQuery {
            document: document.into(),
            offset: 0,
        },
        &Cancellation::default(),
    )
    .unwrap()
}
fn hits(db: &Database) -> usize {
    db.search(
        &SearchQuery {
            text: "Destination".into(),
            kinds: vec![SearchKind::Relation],
            document_id: Some("original".into()),
            ..Default::default()
        },
        &Cancellation::default(),
    )
    .unwrap()
    .len()
}
pub fn verify(seed: &str) {
    let mut db = Database::in_memory_at(11).unwrap();
    db.execute_batch(seed).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT 'target','target.pdf','target.pdf','target.pdf','Destination',page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint FROM documents WHERE id='original'").unwrap();
    let cancel = Cancellation::default();
    assert!(
        db.change_relation(
            &RelationChange::Create {
                source: "original".into(),
                target: "original".into(),
                kind: RelationKind::Related,
                source_page: None,
                target_page: None,
            },
            IndexAccess::Disabled,
            &cancel
        )
        .is_err()
    );
    let change = RelationChange::Create {
        source: "original".into(),
        target: "target".into(),
        kind: RelationKind::Replaces,
        source_page: None,
        target_page: None,
    };
    let id = db
        .change_relation(&change, IndexAccess::Disabled, &cancel)
        .unwrap();
    assert_eq!(hits(&db), 0);
    db.execute_batch("UPDATE document_relations SET source_anchor_kind='annotation',source_anchor_id='historical-note',target_anchor_kind='page',target_anchor_id='2' WHERE source_document_id='original'").unwrap();
    assert_eq!(
        db.change_relation(&change, IndexAccess::Enabled, &cancel)
            .unwrap(),
        id
    );
    assert_eq!(page(&db, "original").total, 1);
    assert_eq!(hits(&db), 1);
    let incoming = page(&db, "target").entries.remove(0);
    assert!(incoming.backlink);
    assert_eq!(incoming.other_document, "original");
    assert_eq!(
        incoming.relation.source_anchor_id.as_deref(),
        Some("historical-note")
    );
    assert_eq!(incoming.relation.target_anchor_id.as_deref(), Some("2"));
    let remove = RelationChange::Remove {
        document: "target".into(),
        expected: incoming.relation.clone(),
    };
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(
        db.change_relation(&remove, IndexAccess::Enabled, &cancelled)
            .is_err()
    );
    assert_eq!(page(&db, "target").total, 1);
    db.execute_batch("CREATE TEMP TRIGGER reject_relation_delete BEFORE DELETE ON document_relations BEGIN SELECT RAISE(ABORT,'synthetic relation failure'); END;").unwrap();
    assert!(
        db.change_relation(&remove, IndexAccess::Enabled, &cancel)
            .is_err()
    );
    assert_eq!(page(&db, "target").total, 1);
    assert_eq!(hits(&db), 1);
    db.execute_batch("DROP TRIGGER reject_relation_delete")
        .unwrap();
    let mut stale = incoming.relation.clone();
    stale.target_anchor_id = Some("3".into());
    assert!(
        db.change_relation(
            &RelationChange::Remove {
                document: "target".into(),
                expected: stale
            },
            IndexAccess::Enabled,
            &cancel
        )
        .is_err()
    );
    db.change_relation(&remove, IndexAccess::Enabled, &cancel)
        .unwrap();
    assert_eq!(page(&db, "original").total, 0);
    assert_eq!(page(&db, "target").total, 0);
    assert_eq!(hits(&db), 0);
    db.exclude_from_index("target", true, &cancel).unwrap();
    db.change_relation(&change, IndexAccess::Enabled, &cancel)
        .unwrap();
    assert_eq!(hits(&db), 0);
    assert_eq!(page(&db, "original").total, 1);
    verify_locations(&mut db);
    db.integrity().unwrap();
}

fn verify_locations(db: &mut Database) {
    let cancel = Cancellation::default();
    let existing = page(db, "original").entries.remove(0).relation;
    db.change_relation(
        &RelationChange::Remove {
            document: "original".into(),
            expected: existing,
        },
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    let change = RelationChange::Create {
        source: "original".into(),
        target: "target".into(),
        kind: RelationKind::Related,
        source_page: Some(1),
        target_page: Some(4),
    };
    let id = db
        .change_relation(&change, IndexAccess::Enabled, &cancel)
        .unwrap();
    assert_eq!(
        page(db, "original").entries[0].destination,
        RelationDestination::Page(4)
    );
    assert_eq!(
        page(db, "target").entries[0].destination,
        RelationDestination::Page(1)
    );
    assert_eq!(
        db.change_relation(&change, IndexAccess::Enabled, &cancel)
            .unwrap(),
        id
    );
    for target_page in [Some(2), Some(7), Some(u32::MAX)] {
        assert!(
            db.change_relation(
                &RelationChange::Create {
                    source: "original".into(),
                    target: "target".into(),
                    kind: RelationKind::Related,
                    source_page: Some(1),
                    target_page
                },
                IndexAccess::Enabled,
                &cancel
            )
            .is_err()
        );
        assert_eq!(
            page(db, "original").entries[0].destination,
            RelationDestination::Page(4)
        );
    }
    db.execute_batch("INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at) VALUES('relation-bookmark','target',3,'Chapter','now','now'); UPDATE document_relations SET target_anchor_kind='bookmark',target_anchor_id='relation-bookmark';").unwrap();
    assert_eq!(
        page(db, "original").entries[0].destination,
        RelationDestination::Page(3)
    );
    db.execute_batch("UPDATE bookmarks SET document_id='original' WHERE id='relation-bookmark'")
        .unwrap();
    assert_eq!(
        page(db, "original").entries[0].destination,
        RelationDestination::Unavailable
    );
    db.execute_batch("INSERT INTO annotations(id,document_id,page_index,type,color,document_fingerprint,created_at,updated_at) VALUES('relation-note','target',5,'pageComment','violet','legacy','now','now'); UPDATE document_relations SET target_anchor_kind='annotation',target_anchor_id='relation-note';").unwrap();
    assert_eq!(
        page(db, "original").entries[0].destination,
        RelationDestination::Page(5)
    );
    db.execute_batch("UPDATE annotations SET status='orphaned' WHERE id='relation-note'")
        .unwrap();
    assert_eq!(
        page(db, "original").entries[0].destination,
        RelationDestination::Unavailable
    );
    for sql in [
        "UPDATE document_relations SET target_anchor_kind='future',target_anchor_id='kept'",
        "UPDATE document_relations SET target_anchor_kind='page',target_anchor_id='-1'",
        "UPDATE document_relations SET target_anchor_id='7'",
        "UPDATE document_relations SET target_anchor_id=NULL",
    ] {
        db.execute_batch(sql).unwrap();
        assert_eq!(
            page(db, "original").entries[0].destination,
            RelationDestination::Unavailable
        );
    }
    db.execute_batch("UPDATE document_relations SET target_anchor_kind=NULL,target_anchor_id=NULL")
        .unwrap();
    assert_eq!(
        page(db, "original").entries[0].destination,
        RelationDestination::Document
    );
}
