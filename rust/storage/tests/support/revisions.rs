use yamantaka_domain::{Cancellation, LibraryRepository, revisions::*};
use yamantaka_storage::Database;
fn groups(db: &Database, document: &str) -> RevisionPage {
    db.revision_groups(
        &RevisionQuery {
            document: document.into(),
            offset: 0,
        },
        &Cancellation::default(),
    )
    .unwrap()
}
fn members(db: &Database, group: &str) -> RevisionMemberPage {
    db.revision_members(
        &RevisionMemberQuery {
            group: group.into(),
            offset: 0,
        },
        &Cancellation::default(),
    )
    .unwrap()
}
pub fn verify(seed: &str) {
    let mut db = Database::in_memory_at(11).unwrap();
    db.execute_batch(seed).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT 'target','target.pdf','target.pdf','target.pdf','Other edition',page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint FROM documents WHERE id='original'; WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<79) INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT printf('extra-%02d',x),printf('extra-%02d.pdf',x),printf('extra-%02d.pdf',x),'extra.pdf','Edition',page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint FROM n JOIN documents ON documents.id='original'; UPDATE documents SET title_override='Manual edition' WHERE id='target';").unwrap();
    let cancel = Cancellation::default();
    let create = RevisionChange::Create {
        name: "  Editions 文献  ".into(),
        members: vec!["original".into(), "target".into(), "original".into()],
        current: Some("original".into()),
    };
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(db.change_revision(&create, &cancelled).is_err());
    assert!(
        db.change_revision(
            &RevisionChange::Create {
                name: " ".into(),
                members: vec!["original".into(), "target".into()],
                current: None
            },
            &cancel
        )
        .is_err()
    );
    assert!(
        db.change_revision(
            &RevisionChange::Create {
                name: "Missing".into(),
                members: vec!["original".into(), "missing".into()],
                current: None
            },
            &cancel
        )
        .is_err()
    );
    assert!(
        db.change_revision(
            &RevisionChange::Create {
                name: "Not a member".into(),
                members: vec!["original".into(), "target".into()],
                current: Some("extra-00".into())
            },
            &cancel
        )
        .is_err()
    );
    db.execute_batch("CREATE TEMP TRIGGER reject_second_member BEFORE INSERT ON document_revision_members WHEN NEW.document_id='target' BEGIN SELECT RAISE(ABORT,'synthetic revision failure'); END;").unwrap();
    assert!(db.change_revision(&create, &cancel).is_err());
    assert_eq!(groups(&db, "original").total, 0);
    db.execute_batch("DROP TRIGGER reject_second_member")
        .unwrap();
    let id = db.change_revision(&create, &cancel).unwrap();
    let original = groups(&db, "original").entries.remove(0);
    assert_eq!(original.name, "Editions 文献");
    assert_eq!(original.member_count, 2);
    assert_eq!(original.current.as_deref(), Some("original"));
    assert_eq!(groups(&db, "target").entries[0].id, id);
    let list = members(&db, &id);
    assert_eq!(list.total, 2);
    assert!(
        list.entries
            .iter()
            .any(|member| member.document == "target" && member.title == "Manual edition")
    );
    assert!(
        db.change_revision(
            &RevisionChange::SetCurrent {
                expected: (&original).into(),
                document: "extra-00".into()
            },
            &cancel
        )
        .is_err()
    );
    db.change_revision(
        &RevisionChange::SetCurrent {
            expected: (&original).into(),
            document: "target".into(),
        },
        &cancel,
    )
    .unwrap();
    let current = groups(&db, "original").entries.remove(0);
    assert_eq!(current.current_title.as_deref(), Some("Manual edition"));
    assert_eq!(current.created_at, original.created_at);
    assert_ne!(current.updated_at, original.updated_at);
    assert!(
        db.change_revision(
            &RevisionChange::SetCurrent {
                expected: (&original).into(),
                document: "original".into()
            },
            &cancel
        )
        .is_err()
    );
    db.execute_batch("CREATE TEMP TRIGGER reject_current BEFORE UPDATE ON document_revision_groups BEGIN SELECT RAISE(ABORT,'synthetic current revision failure'); END;").unwrap();
    assert!(
        db.change_revision(
            &RevisionChange::SetCurrent {
                expected: (&current).into(),
                document: "original".into()
            },
            &cancel
        )
        .is_err()
    );
    assert_eq!(
        groups(&db, "original").entries[0].current.as_deref(),
        Some("target")
    );
    db.execute_batch("DROP TRIGGER reject_current; DELETE FROM documents WHERE id='target'")
        .unwrap();
    assert!(
        db.change_revision(
            &RevisionChange::SetCurrent {
                expected: (&current).into(),
                document: "original".into()
            },
            &cancel
        )
        .is_err()
    );
    assert_eq!(members(&db, &id).total, 1);
    assert_eq!(groups(&db, "original").entries[0].current, None);
    let many: Vec<String> = (0..80).map(|index| format!("extra-{index:02}")).collect();
    let large = db
        .change_revision(
            &RevisionChange::Create {
                name: "Large historical group".into(),
                members: many.clone(),
                current: Some("extra-79".into()),
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(members(&db, &large).entries.len(), 80);
    let tail = db
        .revision_members(
            &RevisionMemberQuery {
                group: large,
                offset: 80,
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(tail.total, 80);
    assert!(tail.entries.is_empty());
    let mut oversized = many;
    oversized.push("original".into());
    assert!(
        db.change_revision(
            &RevisionChange::Create {
                name: "Too many".into(),
                members: oversized,
                current: None
            },
            &cancel
        )
        .is_err()
    );
    db.integrity().unwrap();
}
