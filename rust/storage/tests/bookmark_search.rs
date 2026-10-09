use yamantaka_domain::{Cancellation, bookmarks::*, search::*};
use yamantaka_storage::Database;

fn library() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf','Study',3,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf','Other',2,1,'now','now','now','fb');").unwrap();
    db
}
fn hits(db: &Database, text: &str) -> Vec<SearchResult> {
    db.search(
        &SearchQuery {
            text: text.into(),
            kinds: vec![SearchKind::Bookmark],
            ..Default::default()
        },
        &Cancellation::default(),
    )
    .unwrap()
}
fn change(db: &mut Database, change: BookmarkChange, access: IndexAccess) -> BookmarkPage {
    db.change_bookmark("a", &change, 0, access, &Cancellation::default())
        .unwrap()
}

#[test]
fn enabled_changes_are_immediate_and_do_not_require_pdf_indexing() {
    let mut db = library();
    let page = change(
        &mut db,
        BookmarkChange::Save {
            page: 1,
            label: Some("光学 revisión".into()),
        },
        IndexAccess::Enabled,
    );
    let result = hits(&db, "revisión");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].document_id, "a");
    assert_eq!(result[0].page, Some(1));
    let id = page.entries[0].id.clone();
    change(
        &mut db,
        BookmarkChange::Ensure { page: 1 },
        IndexAccess::Enabled,
    );
    assert_eq!(hits(&db, "revisión").len(), 1);
    change(
        &mut db,
        BookmarkChange::Rename {
            id: id.clone(),
            label: Some("Tomorrow 宇宙".into()),
        },
        IndexAccess::Enabled,
    );
    assert!(hits(&db, "revisión").is_empty());
    assert_eq!(hits(&db, "Tomorrow").len(), 1);
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(
        hits(&db, "Tomorrow").len(),
        1,
        "bulk refresh must retain bookmarks without a PDF index state"
    );
    change(&mut db, BookmarkChange::Remove { id }, IndexAccess::Enabled);
    assert!(hits(&db, "Tomorrow").is_empty());
    assert!(db.index_state("a").unwrap().is_none());
}

#[test]
fn disabled_excluded_and_cancelled_edits_respect_consent() {
    let mut db = library();
    let page = change(
        &mut db,
        BookmarkChange::Save {
            page: 1,
            label: Some("Original".into()),
        },
        IndexAccess::Enabled,
    );
    let id = page.entries[0].id.clone();
    change(
        &mut db,
        BookmarkChange::Rename {
            id: id.clone(),
            label: Some("DisabledSecret".into()),
        },
        IndexAccess::Disabled,
    );
    assert!(hits(&db, "DisabledSecret").is_empty());
    assert_eq!(
        hits(&db, "Original").len(),
        1,
        "disabling indexing does not implicitly erase retained entries"
    );
    db.exclude_from_index("a", true, &Cancellation::default())
        .unwrap();
    change(
        &mut db,
        BookmarkChange::Rename {
            id: id.clone(),
            label: Some("ExcludedSecret".into()),
        },
        IndexAccess::Enabled,
    );
    assert!(hits(&db, "ExcludedSecret").is_empty());
    assert_eq!(db.index_stats().unwrap().rows, 0);
    let before = db.bookmarks("a", 0, &Cancellation::default()).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(
        db.change_bookmark(
            "a",
            &BookmarkChange::Remove { id },
            0,
            IndexAccess::Enabled,
            &cancel
        )
        .is_err()
    );
    assert_eq!(
        db.bookmarks("a", 0, &Cancellation::default()).unwrap(),
        before
    );
    db.exclude_from_index("a", false, &Cancellation::default())
        .unwrap();
    change(
        &mut db,
        BookmarkChange::Ensure { page: 1 },
        IndexAccess::Enabled,
    );
    assert_eq!(hits(&db, "ExcludedSecret").len(), 1);
}

#[test]
fn fts_failure_rolls_back_the_bookmark_write_and_disabled_edits_still_work() {
    let mut db = library();
    let original = change(
        &mut db,
        BookmarkChange::Save {
            page: 0,
            label: Some("Preserved".into()),
        },
        IndexAccess::Disabled,
    );
    let edit = BookmarkChange::Rename {
        id: original.entries[0].id.clone(),
        label: Some("Attempted".into()),
    };
    db.execute_batch("DROP TABLE library_search;").unwrap();
    assert!(
        db.change_bookmark(
            "a",
            &edit,
            0,
            IndexAccess::Enabled,
            &Cancellation::default()
        )
        .is_err()
    );
    assert_eq!(
        db.bookmarks("a", 0, &Cancellation::default()).unwrap(),
        original
    );
    let disabled = change(&mut db, edit, IndexAccess::Disabled);
    assert_eq!(disabled.entries[0].label.as_deref(), Some("Attempted"));
}

#[test]
fn bookmark_refresh_preserves_native_index_sessions_and_other_search_entries() {
    let mut db = library();
    let cancel = Cancellation::default();
    let mut session = db
        .begin_index("a", false, IndexAccess::Enabled, &cancel)
        .unwrap();
    db.index_page(
        &mut session,
        0,
        "PageContent",
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    db.change_bookmark(
        "b",
        &BookmarkChange::Save {
            page: 0,
            label: Some("OtherPassage".into()),
        },
        0,
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    let before = db.index_state("a").unwrap();
    change(
        &mut db,
        BookmarkChange::Save {
            page: 1,
            label: Some("Passage".into()),
        },
        IndexAccess::Enabled,
    );
    assert_eq!(db.index_state("a").unwrap(), before);
    assert_eq!(hits(&db, "OtherPassage").len(), 1);
    let result = db
        .search(
            &SearchQuery {
                text: "PageContent".into(),
                ..Default::default()
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].kind, SearchKind::PageText);
    db.index_page(
        &mut session,
        1,
        "LaterContent",
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    assert_eq!(hits(&db, "Passage").len(), 1);
}
