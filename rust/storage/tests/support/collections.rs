use yamantaka_domain::{Cancellation, LibraryQuery, LibraryRepository, collections::*};
use yamantaka_storage::Database;

fn page(db: &Database) -> CollectionPage {
    db.collections(
        &CollectionQuery {
            document: Some("original".into()),
            ..Default::default()
        },
        &Cancellation::default(),
    )
    .unwrap()
}
pub fn verify(db: &mut Database) {
    let cancel = Cancellation::default();
    let before = db.get("original").unwrap().unwrap();
    let id = db
        .change_collection(
            &CollectionChange::Create {
                name: "  Estudio 文献  ".into(),
                icon: "study".into(),
            },
            &cancel,
        )
        .unwrap();
    let collection = page(db).entries.remove(0);
    assert_eq!(
        (
            &collection.id,
            collection.name.as_str(),
            collection.icon.as_str(),
            collection.document_count,
            collection.contains_document
        ),
        (&id, "Estudio 文献", "study", 0, false)
    );
    assert!(
        db.change_collection(
            &CollectionChange::Create {
                name: "estudio 文献".into(),
                icon: "folder".into()
            },
            &cancel
        )
        .is_err()
    );
    for invalid in [String::new(), "x".repeat(61), "😀".repeat(31)] {
        assert!(
            db.change_collection(
                &CollectionChange::Create {
                    name: invalid,
                    icon: "folder".into()
                },
                &cancel
            )
            .is_err()
        );
    }
    let assign = |included| CollectionChange::SetMembership {
        expected: (&collection).into(),
        document: "original".into(),
        included,
    };
    db.change_collection(&assign(true), &cancel).unwrap();
    db.change_collection(&assign(true), &cancel).unwrap();
    let assigned = page(db).entries.remove(0);
    assert_eq!(
        (assigned.document_count, assigned.contains_document),
        (1, true)
    );
    let filter = LibraryQuery {
        collection_id: Some(id.clone()),
        ..Default::default()
    };
    assert_eq!(db.query(&filter).unwrap()[0].id, "original");
    assert_eq!(db.count(&filter).unwrap(), 1);
    db.change_collection(&assign(false), &cancel).unwrap();
    assert_eq!(db.count(&filter).unwrap(), 0);
    db.change_collection(&assign(true), &cancel).unwrap();
    let snapshot = CollectionRevision::from(&assigned);
    db.change_collection(
        &CollectionChange::Rename {
            expected: snapshot.clone(),
            name: "Study 100%_\\".into(),
        },
        &cancel,
    )
    .unwrap();
    assert!(
        db.change_collection(&CollectionChange::Delete { expected: snapshot }, &cancel)
            .is_err()
    );
    let renamed = page(db).entries.remove(0);
    assert_ne!(renamed.updated_at, assigned.updated_at);
    assert_eq!(renamed.document_count, 1);
    let literal = db
        .collections(
            &CollectionQuery {
                text: "%_\\".into(),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(
        (literal.total, literal.entries[0].id.as_str()),
        (1, id.as_str())
    );

    db.execute_batch("CREATE TRIGGER reject_collection_delete BEFORE DELETE ON collections BEGIN SELECT RAISE(ABORT,'synthetic rollback'); END;").unwrap();
    assert!(
        db.change_collection(
            &CollectionChange::Delete {
                expected: (&renamed).into()
            },
            &cancel
        )
        .is_err()
    );
    assert_eq!(db.count(&filter).unwrap(), 1);
    db.execute_batch("DROP TRIGGER reject_collection_delete")
        .unwrap();
    let stop = Cancellation::default();
    stop.cancel();
    assert!(
        db.change_collection(
            &CollectionChange::Delete {
                expected: (&renamed).into()
            },
            &stop
        )
        .is_err()
    );
    assert_eq!(db.count(&filter).unwrap(), 1);
    assert!(
        db.change_collection(
            &CollectionChange::SetMembership {
                expected: (&renamed).into(),
                document: "missing".into(),
                included: true
            },
            &cancel
        )
        .is_err()
    );

    for index in 0..81 {
        db.change_collection(
            &CollectionChange::Create {
                name: format!("Reference {index:03}"),
                icon: "reference".into(),
            },
            &cancel,
        )
        .unwrap();
    }
    let first = db
        .collections(&CollectionQuery::default(), &cancel)
        .unwrap();
    let second = db
        .collections(
            &CollectionQuery {
                offset: 80,
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(
        (first.total, first.entries.len(), second.entries.len()),
        (82, 80, 2)
    );
    assert!(
        first
            .entries
            .iter()
            .all(|entry| second.entries.iter().all(|other| entry.id != other.id))
    );
    let late = second.entries.last().unwrap();
    let lookup = db
        .collections(
            &CollectionQuery {
                id: Some(late.id.clone()),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(
        (
            lookup.total,
            lookup.entries.len(),
            lookup.entries[0].id.as_str()
        ),
        (1, 1, late.id.as_str())
    );
    let absent = db
        .collections(
            &CollectionQuery {
                id: Some("no-such-collection".into()),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(absent.total, 0);
    assert!(absent.entries.is_empty());
    db.change_collection(
        &CollectionChange::Pin {
            expected: late.into(),
            pinned: true,
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(page(db).entries[0].id, late.id);
    db.change_collection(
        &CollectionChange::Delete {
            expected: (&renamed).into(),
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(db.count(&filter).unwrap(), 0);
    assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 1);
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
            before.id,
            before.title,
            before.path,
            before.last_page_index,
            before.reading_progress
        )
    );
    db.integrity().unwrap();
}
