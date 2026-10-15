use yamantaka_domain::{Cancellation, Failure, search::*};
use yamantaka_storage::Database;

fn library() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch(r#"
        INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,is_missing,is_modified)
        VALUES ('a','a.pdf','a.pdf','a.pdf','Raw title',3,1,'now','now','2026-01-01','fa',0,0),
               ('b','b.pdf','b.pdf','b.pdf','Otro libro',1,1,'now','now','2026-01-02','fb',1,1);
        UPDATE documents SET title_override='Óptica y señales',author_override='Ada García' WHERE id='a';
        INSERT INTO document_notes VALUES('a','Cuaderno personal de análisis','now');
    "#).unwrap();
    db
}
fn query(text: &str) -> SearchQuery {
    SearchQuery {
        text: text.into(),
        ..SearchQuery::default()
    }
}
fn begin(db: &mut Database, id: &str) -> IndexSession {
    db.begin_index(id, false, IndexAccess::Enabled, &Cancellation::default())
        .unwrap()
}
fn page(db: &mut Database, s: &mut IndexSession, p: u32, text: &str) {
    db.index_page(s, p, text, IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
}

#[test]
fn current_metadata_precedes_raw_sources_and_changes_invalidate_the_metadata_index() {
    let mut db = library();
    db.execute_batch("UPDATE documents SET title='Validated optics',author='Reviewed institute',source_title='Raw processor title',source_author='Raw application',title_override=NULL,author_override=NULL,page_count=1 WHERE id='a'").unwrap();
    let mut index = begin(&mut db, "a");
    page(&mut db, &mut index, 0, "Synthetic page body");
    let cancel = Cancellation::default();
    db.finish_index(&mut index, IndexAccess::Enabled, &cancel)
        .unwrap();
    let document = yamantaka_domain::LibraryRepository::get(&db, "a")
        .unwrap()
        .unwrap();
    let hits = db.search(&query("Validated"), &cancel).unwrap();
    assert!(!hits.is_empty());
    assert!(hits.iter().all(|hit| hit.title == document.display_title()));
    assert!(!db.search(&query("institute"), &cancel).unwrap().is_empty());
    assert!(db.search(&query("processor"), &cancel).unwrap().is_empty());
    assert!(
        db.search(&query("application"), &cancel)
            .unwrap()
            .is_empty()
    );
    assert!(
        !db.index_candidates(None, 80, IndexAccess::Enabled, &cancel)
            .unwrap()
            .contains(&"a".into())
    );
    db.execute_batch("UPDATE documents SET title='Updated optics' WHERE id='a'")
        .unwrap();
    assert!(
        db.index_candidates(None, 80, IndexAccess::Enabled, &cancel)
            .unwrap()
            .contains(&"a".into())
    );
    db.execute_batch("UPDATE documents SET title_override='Manual optics',author_override='Manual institute' WHERE id='a'").unwrap();
    let mut index = begin(&mut db, "a");
    page(&mut db, &mut index, 0, "Synthetic page body");
    db.finish_index(&mut index, IndexAccess::Enabled, &cancel)
        .unwrap();
    let hits = db.search(&query("Manual"), &cancel).unwrap();
    assert!(!hits.is_empty());
    assert!(hits.iter().all(|hit| hit.title == "Manual optics"));
}

#[test]
fn real_unicode_fts_filters_previews_and_relevance() {
    let mut db = library();
    let mut a = begin(&mut db, "a");
    page(
        &mut db,
        &mut a,
        0,
        "La óptica describe señales luminosas. 東京 光学.",
    );
    page(&mut db, &mut a, 1, "Una señal distante");
    page(&mut db, &mut a, 2, "");
    db.finish_index(&mut a, IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    let mut b = begin(&mut db, "b");
    page(&mut db, &mut b, 0, "Óptica antigua");
    let hits = db
        .search(&query("optica"), &Cancellation::default())
        .unwrap();
    assert!(hits.len() >= 3);
    assert!(hits.windows(2).all(|pair| pair[0].rank <= pair[1].rank));
    assert!(hits.iter().all(|h| h.rank.is_finite()));
    assert!(
        hits.iter()
            .any(|h| h.kind == SearchKind::PageText && h.snippet.contains('‹'))
    );
    assert!(hits.iter().any(|h| h.missing && h.stale));
    let q = SearchQuery {
        kinds: vec![SearchKind::PageText],
        missing: Some(false),
        stale: Some(false),
        document_id: Some("a".into()),
        ..query("optica luminosas")
    };
    let hits = db.search(&q, &Cancellation::default()).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].page, Some(0));
    assert_eq!(
        db.search(&query("東京"), &Cancellation::default())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        db.search(&query("Garcia analisis"), &Cancellation::default())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(db.schema_version().unwrap(), 11);
}

#[test]
fn quoted_prefixes_never_execute_user_fts_or_sql_syntax() {
    let mut db = library();
    let mut a = begin(&mut db, "a");
    page(&mut db, &mut a, 0, "alpha beta");
    assert_eq!(
        fts_expression(" alpha \"beta\" "),
        "\"alpha\"* AND \"beta\"*"
    );
    assert_eq!(
        fts_expression("a b c d e f g h ignored"),
        "\"a\"* AND \"b\"* AND \"c\"* AND \"d\"* AND \"e\"* AND \"f\"* AND \"g\"* AND \"h\"*"
    );
    for input in [
        "\"",
        "alpha OR beta",
        "alpha NEAR(beta)",
        "body:alpha",
        "alpha'); DROP TABLE documents; --",
        "* ( ) : ^",
    ] {
        assert!(
            db.search(&query(input), &Cancellation::default())
                .unwrap()
                .is_empty(),
            "{input}"
        );
    }
    assert_eq!(
        db.search(&query("al be"), &Cancellation::default())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(db.schema_version().unwrap(), 11);
}

#[test]
fn cancellation_resume_empty_pages_replacement_and_late_jobs() {
    let mut db = library();
    let mut a = begin(&mut db, "a");
    page(&mut db, &mut a, 0, "first old");
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.index_page(&mut a, 1, "must not appear", IndexAccess::Enabled, &cancel),
        Err(Failure::Cancelled)
    ));
    assert_eq!(db.index_state("a").unwrap().unwrap().indexed_pages, 1);
    let old = a.clone();
    let mut resumed = db
        .begin_index("a", true, IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(resumed.cursor, 1);
    let mut late = old;
    assert!(
        db.index_page(
            &mut late,
            1,
            "late",
            IndexAccess::Enabled,
            &Cancellation::default()
        )
        .is_err()
    );
    page(&mut db, &mut resumed, 0, "first replaced");
    assert_eq!(resumed.cursor, 1);
    page(&mut db, &mut resumed, 1, "");
    page(&mut db, &mut resumed, 2, "last");
    db.finish_index(&mut resumed, IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(
        db.search(&query("first"), &Cancellation::default())
            .unwrap()
            .len(),
        1
    );
    assert!(
        db.search(&query("old"), &Cancellation::default())
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        db.index_state("a").unwrap().unwrap().status,
        IndexStatus::Indexed
    );
    assert_eq!(db.index_stats().unwrap().rows, 3); // metadata + two nonempty pages
}

#[test]
fn fingerprint_changes_reset_and_reject_obsolete_writes() {
    let mut db = library();
    let mut a = begin(&mut db, "a");
    page(&mut db, &mut a, 0, "obsolete");
    db.execute_batch("UPDATE documents SET fingerprint='new-identity' WHERE id='a'")
        .unwrap();
    assert!(
        db.index_page(
            &mut a,
            1,
            "stale",
            IndexAccess::Enabled,
            &Cancellation::default()
        )
        .is_err()
    );
    let mut new = db
        .begin_index("a", true, IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(new.cursor, 0);
    assert!(
        db.search(&query("obsolete"), &Cancellation::default())
            .unwrap()
            .is_empty()
    );
    assert!(
        db.finish_index(&mut new, IndexAccess::Enabled, &Cancellation::default())
            .is_err()
    );
    assert!(
        db.index_page(
            &mut new,
            2,
            "out of order",
            IndexAccess::Enabled,
            &Cancellation::default()
        )
        .is_err()
    );
    for p in 0..3 {
        page(&mut db, &mut new, p, "");
    }
    db.finish_index(&mut new, IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(db.index_stats().unwrap().not_indexable, 1);
}

#[test]
fn opt_in_and_persistent_exclusion_gate_all_indexing_and_rebuild() {
    let mut db = library();
    assert!(
        db.begin_index("a", false, IndexAccess::Disabled, &Cancellation::default())
            .is_err()
    );
    assert_eq!(db.index_stats().unwrap().rows, 0);
    let mut a = begin(&mut db, "a");
    page(&mut db, &mut a, 0, "private");
    assert!(
        db.index_page(
            &mut a,
            1,
            "disabled",
            IndexAccess::Disabled,
            &Cancellation::default()
        )
        .is_err()
    );
    db.exclude_from_index("a", true, &Cancellation::default())
        .unwrap();
    assert!(
        db.search(&query("private"), &Cancellation::default())
            .unwrap()
            .is_empty()
    );
    assert!(
        db.begin_index("a", true, IndexAccess::Enabled, &Cancellation::default())
            .is_err()
    );
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    db.clear_index(&Cancellation::default()).unwrap();
    assert_eq!(db.index_stats().unwrap().excluded, 1);
    assert!(
        db.begin_index("a", false, IndexAccess::Enabled, &Cancellation::default())
            .is_err()
    );
    db.exclude_from_index("a", false, &Cancellation::default())
        .unwrap();
    assert_eq!(begin(&mut db, "a").cursor, 0);
}

#[test]
fn auxiliary_entries_refresh_transactionally_without_duplicates_or_excluded_text() {
    let mut db = library();
    begin(&mut db, "a");
    begin(&mut db, "b");
    db.execute_batch(r#"
        INSERT INTO tags VALUES('t','Astronomía','astronomia','blue','now','now');
        INSERT INTO document_tags VALUES('a','t'),('b','t');
        INSERT INTO annotations(id,document_id,page_index,type,color,note,selected_text,document_fingerprint,created_at,updated_at) VALUES('an','a',1,'note','blue','Astronomía anotada','Órbita','fa','now','now');
        INSERT INTO annotation_tags VALUES('an','t');
        INSERT INTO bookmarks VALUES('bm','a',1,'Astronomía repasar','now','now');
        INSERT INTO collections(id,name,normalized_name,created_at,updated_at) VALUES('c','Astronomía colección','astronomia coleccion','now','now');
        INSERT INTO collection_documents VALUES('c','a','now');
        INSERT INTO document_relations(id,source_document_id,target_document_id,relation_type,created_at) VALUES('r','a','b','sourceOf','now');
    "#).unwrap();
    db.exclude_from_index("b", true, &Cancellation::default())
        .unwrap();
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    let before = db.index_stats().unwrap().rows;
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(db.index_stats().unwrap().rows, before);
    for kind in [
        SearchKind::Annotation,
        SearchKind::Bookmark,
        SearchKind::Collection,
        SearchKind::Tag,
    ] {
        let q = SearchQuery {
            kinds: vec![kind],
            ..query("astronomia")
        };
        let hits = db.search(&q, &Cancellation::default()).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document_id, "a");
    }
    assert_eq!(
        db.search(
            &SearchQuery {
                kinds: vec![SearchKind::Relation],
                ..query("sourceOf")
            },
            &Cancellation::default()
        )
        .unwrap()
        .len(),
        0
    );
    db.exclude_from_index("b", false, &Cancellation::default())
        .unwrap();
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(
        db.search(
            &SearchQuery {
                kinds: vec![SearchKind::Relation],
                ..query("sourceOf")
            },
            &Cancellation::default()
        )
        .unwrap()
        .len(),
        1
    );
    db.execute_batch("UPDATE documents SET title_override='New manual title' WHERE id='a'")
        .unwrap();
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert!(
        db.search(
            &SearchQuery {
                document_id: Some("a".into()),
                ..query("astronomia")
            },
            &Cancellation::default()
        )
        .unwrap()
        .iter()
        .all(|r| r.title == "New manual title")
    );
}

#[test]
fn bounded_pagination_order_and_removed_documents() {
    let mut db = library();
    let mut a = begin(&mut db, "a");
    page(&mut db, &mut a, 0, "needle");
    db.execute_batch("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT 'a',-1,'tag','Óptica y señales','needle','' FROM json_each(json('[1,2,3,4,5]'))").unwrap();
    let all = db
        .search(&query("needle"), &Cancellation::default())
        .unwrap();
    let q = SearchQuery {
        limit: 2,
        offset: 2,
        ..query("needle")
    };
    assert_eq!(db.search(&q, &Cancellation::default()).unwrap(), all[2..4]);
    assert_eq!(
        db.search(
            &SearchQuery {
                limit: 0,
                ..query("needle")
            },
            &Cancellation::default()
        )
        .unwrap()
        .len(),
        1
    );
    assert!(
        db.search(
            &SearchQuery {
                offset: usize::MAX,
                ..query("needle")
            },
            &Cancellation::default()
        )
        .unwrap()
        .is_empty()
    );
    db.execute_batch("DELETE FROM documents WHERE id='a'")
        .unwrap();
    assert!(
        db.search(&query("needle"), &Cancellation::default())
            .unwrap()
            .is_empty()
    );
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(db.index_stats().unwrap().rows, 0);
}

#[test]
fn failures_are_redacted_codes_and_cancelled_changes_leave_state_intact() {
    let mut db = library();
    let mut a = begin(&mut db, "a");
    assert!(
        db.fail_index(&mut a, "C:\\private\\secret.pdf", &Cancellation::default())
            .is_err()
    );
    db.fail_index(&mut a, "passwordRequired", &Cancellation::default())
        .unwrap();
    assert_eq!(
        db.index_state("a").unwrap().unwrap().error_code.as_deref(),
        Some("passwordRequired")
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(db.exclude_from_index("a", true, &cancel).is_err());
    assert!(db.clear_index(&cancel).is_err());
    assert_eq!(
        db.index_state("a").unwrap().unwrap().status,
        IndexStatus::Failed
    );
    assert!(db.search(&query("optica"), &cancel).is_err());
}

#[test]
fn cursor_write_failure_rolls_back_page_and_resume_survives_process_restart() {
    let mut db = library();
    let mut a = begin(&mut db, "a");
    page(&mut db, &mut a, 0, "durable first");
    db.execute_batch("CREATE TRIGGER injected_failure BEFORE UPDATE OF indexed_pages ON search_index_state WHEN NEW.indexed_pages=2 BEGIN SELECT RAISE(ABORT,'injected write failure'); END;").unwrap();
    let revision = a.revision.clone();
    assert!(
        db.index_page(
            &mut a,
            1,
            "must rollback",
            IndexAccess::Enabled,
            &Cancellation::default()
        )
        .is_err()
    );
    assert_eq!(a.revision, revision);
    assert_eq!(db.index_state("a").unwrap().unwrap().indexed_pages, 1);
    assert!(
        db.search(&query("rollback"), &Cancellation::default())
            .unwrap()
            .is_empty()
    );
    db.execute_batch("DROP TRIGGER injected_failure").unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    db.snapshot(&path).unwrap();
    drop(db);
    let mut db = Database::open(&path).unwrap();
    let mut restored = db
        .begin_index("a", true, IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert_eq!(restored.cursor, 1);
    page(&mut db, &mut restored, 1, "");
    page(&mut db, &mut restored, 2, "durable last");
    db.finish_index(
        &mut restored,
        IndexAccess::Enabled,
        &Cancellation::default(),
    )
    .unwrap();
    assert_eq!(
        db.search(&query("durable"), &Cancellation::default())
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn incremental_candidates_skip_missing_excluded_and_completed_until_metadata_changes() {
    let mut db = library();
    assert_eq!(
        db.index_candidates(
            None,
            usize::MAX,
            IndexAccess::Enabled,
            &Cancellation::default()
        )
        .unwrap(),
        vec!["a"]
    );
    let mut a = begin(&mut db, "a");
    for p in 0..3 {
        page(&mut db, &mut a, p, "");
    }
    db.finish_index(&mut a, IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert!(!db.needs_index("a").unwrap());
    assert!(
        db.index_candidates(None, 80, IndexAccess::Enabled, &Cancellation::default())
            .unwrap()
            .is_empty()
    );
    db.execute_batch("UPDATE document_notes SET note='Changed manual note' WHERE document_id='a'")
        .unwrap();
    assert!(db.needs_index("a").unwrap());
    assert_eq!(
        db.index_candidates(None, 80, IndexAccess::Enabled, &Cancellation::default())
            .unwrap(),
        vec!["a"]
    );
    assert!(
        db.index_candidates(
            Some("a"),
            80,
            IndexAccess::Enabled,
            &Cancellation::default()
        )
        .unwrap()
        .is_empty()
    );
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    assert!(!db.needs_index("a").unwrap());
    db.exclude_from_index("a", true, &Cancellation::default())
        .unwrap();
    assert!(!db.needs_index("a").unwrap());
    assert!(
        db.index_candidates(None, 80, IndexAccess::Disabled, &Cancellation::default())
            .is_err()
    );
}
