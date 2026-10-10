use rusqlite::Connection;
use serde_json::{Value, json};
use yamantaka_domain::{
    Cancellation, Failure,
    memory::{GlobalMemoryLimits, MemoryGovernor},
    reading_sessions::{ReadingSessionRepository, history::ReadingHistoryQuery},
};
use yamantaka_storage::Database;

fn seed(db: &Database) {
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,title_override,source_title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,is_missing) VALUES('a','study Ω.pdf','a','source-a.pdf','Original','Manual Ω','Source',3,1,'now','now','now','fa',0),('b','missing.pdf','b','source-b.pdf',NULL,NULL,'Source B',3,1,'now','now','now','fb',1); WITH RECURSIVE n(x) AS(VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180) INSERT INTO reading_sessions(id,document_id,started_at,ended_at,start_page,end_page,active_seconds) SELECT printf('s-%03d',x),CASE WHEN x%2=0 THEN 'a' ELSE 'b' END,'2020-01-01T00:00:00Z','2020-01-01T00:10:00Z',0,2,99 FROM n; INSERT INTO reading_sessions(id,document_id,started_at,start_page,end_page,active_seconds) VALUES('live','a','2026-01-01T00:00:00Z',0,0,0);").unwrap();
}
fn budget(bytes: usize) -> MemoryGovernor {
    MemoryGovernor::new(GlobalMemoryLimits {
        total: bytes,
        text: bytes,
        ..Default::default()
    })
}

#[test]
fn completed_history_is_paged_filtered_and_uses_canonical_metadata() {
    let db = Database::in_memory_at(11).unwrap();
    seed(&db);
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let page = db
        .reading_history(&ReadingHistoryQuery::default(), &governor, &cancel)
        .unwrap();
    assert_eq!(
        (page.total(), page.offset(), page.entries().len()),
        (181, 0, 80)
    );
    let first = &page.entries()[0];
    assert_eq!(first.id, "s-180");
    assert_eq!(first.title, "Manual Ω");
    assert_eq!(first.path, "study Ω.pdf");
    assert_eq!(
        (first.start_page, first.end_page, first.active_seconds),
        (0, 2, 99)
    );
    assert!(first.context_json.is_none());
    assert!(governor.usage().total > 0);
    drop(page);
    assert_eq!(governor.usage().total, 0);
    let page = db
        .reading_history(
            &ReadingHistoryQuery {
                document: None,
                offset: 160,
            },
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(page.entries().len(), 21);
    assert_eq!(page.entries().last().unwrap().id, "s-000");
    drop(page);
    let page = db
        .reading_history(
            &ReadingHistoryQuery {
                document: Some("b".into()),
                offset: 80,
            },
            &governor,
            &cancel,
        )
        .unwrap();
    assert_eq!((page.total(), page.entries().len()), (90, 10));
    assert!(
        page.entries()
            .iter()
            .all(|r| r.document == "b" && r.missing && r.title == "Source B")
    );
    drop(page);
    let page = db
        .reading_history(
            &ReadingHistoryQuery {
                document: None,
                offset: usize::MAX,
            },
            &governor,
            &cancel,
        )
        .unwrap();
    assert!(page.entries().is_empty());
    assert_eq!(page.offset(), i64::MAX as usize);
    drop(page);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn contexts_preserve_extensions_and_unrelated_large_objects_are_not_copied() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(&path).unwrap();
    let unusual = "legacy\"quoted\\session";
    read.execute("UPDATE reading_sessions SET id=?1,started_at='2025-01-01T00:00:00Z',ended_at='2025-01-01T00:01:00Z' WHERE id='s-180'", [unusual]).unwrap();
    let context = json!({"zoomLevel":1.5,"future":{"bool":true,"array":[null,"Ω"]}});
    let mut map = json!({"unrelated":{"huge":"x".repeat(400000)},"s-179":true});
    map.as_object_mut()
        .unwrap()
        .insert(unusual.into(), context.clone());
    read.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1',?1,'original')", [map.to_string()]).unwrap();
    let governor = budget(64000);
    let page = db
        .reading_history(
            &ReadingHistoryQuery::default(),
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(page.entries()[0].id, unusual);
    assert_eq!(
        serde_json::from_str::<Value>(page.entries()[0].context_json.as_ref().unwrap()).unwrap(),
        context
    );
    assert!(page.entries()[1].context_json.is_none());
    assert!(governor.usage().total < 64000);
    drop(page);
    assert_eq!(governor.usage().total, 0);
    let unchanged: String = read
        .query_row(
            "SELECT value FROM app_metadata WHERE key='reading_session_contexts_v1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(serde_json::from_str::<Value>(&unchanged).unwrap(), map);
    map.as_object_mut()
        .unwrap()
        .insert(unusual.into(), json!({"future":"x".repeat(400000)}));
    read.execute(
        "UPDATE app_metadata SET value=?1 WHERE key='reading_session_contexts_v1'",
        [map.to_string()],
    )
    .unwrap();
    assert!(matches!(
        db.reading_history(
            &ReadingHistoryQuery::default(),
            &governor,
            &Cancellation::default()
        ),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn cancellation_and_late_budget_denial_release_all_history_memory() {
    let db = Database::in_memory_at(11).unwrap();
    seed(&db);
    let governor = budget(64000);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.reading_history(&ReadingHistoryQuery::default(), &governor, &cancel),
        Err(Failure::Cancelled)
    ));
    assert_eq!(governor.usage().total, 0);
    db.execute_batch(
        "UPDATE documents SET title_override=CAST(zeroblob(200000) AS TEXT) WHERE id='b'",
    )
    .unwrap();
    assert!(matches!(
        db.reading_history(
            &ReadingHistoryQuery::default(),
            &governor,
            &Cancellation::default()
        ),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(governor.usage().total, 0);
    db.execute_batch("UPDATE documents SET title_override=CAST(X'FF' AS TEXT) WHERE id='b'")
        .unwrap();
    assert!(matches!(
        db.reading_history(
            &ReadingHistoryQuery::default(),
            &governor,
            &Cancellation::default()
        ),
        Err(Failure::Storage(_))
    ));
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn malformed_context_duplicate_keys_and_bad_timestamps_fail_without_partial_pages() {
    let db = Database::in_memory_at(11).unwrap();
    seed(&db);
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    db.execute_batch("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1','[broken','original')").unwrap();
    assert!(matches!(
        db.reading_history(&ReadingHistoryQuery::default(), &governor, &cancel),
        Err(Failure::Invalid(_))
    ));
    db.execute_batch("UPDATE app_metadata SET value='{\"s-180\":{},\"s-180\":{\"future\":true}}' WHERE key='reading_session_contexts_v1'").unwrap();
    assert!(matches!(
        db.reading_history(&ReadingHistoryQuery::default(), &governor, &cancel),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(governor.usage().total, 0);
    db.execute_batch("UPDATE app_metadata SET value='{}' WHERE key='reading_session_contexts_v1'; UPDATE reading_sessions SET ended_at='invalid' WHERE id='s-179'").unwrap();
    assert!(matches!(
        db.reading_history(&ReadingHistoryQuery::default(), &governor, &cancel),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(governor.usage().total, 0);
}
