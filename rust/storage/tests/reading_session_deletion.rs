use rusqlite::Connection;
use serde_json::{Value, json};
use yamantaka_domain::{Cancellation, Failure, reading_sessions::ReadingSessionRepository};
use yamantaka_storage::Database;

fn seed(db: &Database) {
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',2,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf',2,1,'now','now','now','fb'); INSERT INTO reading_sessions(id,document_id,started_at,ended_at,start_page,end_page,active_seconds) VALUES('finished','a','2026-01-01T00:00:00Z','2026-01-01T00:01:00Z',0,1,10),('other','b','2026-01-01T00:00:00Z','2026-01-01T00:01:00Z',0,1,10); INSERT INTO reading_sessions(id,document_id,started_at,start_page,end_page,active_seconds) VALUES('live','a','2026-01-01T00:00:00Z',0,0,0);").unwrap();
}
fn metadata(read: &Connection) -> (String, String) {
    read.query_row(
        "SELECT value,updated_at FROM app_metadata WHERE key='reading_session_contexts_v1'",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .unwrap()
}
fn rows(read: &Connection) -> i64 {
    read.query_row("SELECT count(*) FROM reading_sessions", [], |r| r.get(0))
        .unwrap()
}
#[test]
fn deletion_is_scoped_idempotent_and_preserves_other_context_types() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(&path).unwrap();
    let id = "legacy\"quoted\\Ω";
    read.execute(
        "UPDATE reading_sessions SET id=?1 WHERE id='finished'",
        [id],
    )
    .unwrap();
    let original = json!({id:{"zoomLevel":1.25,"future":true},"other":{"kept":true},"live":{"kept":true},"array":[1,true,null],"false":false,"true":true,"null":null,"number":1.25,"text":"Ω quoted \"value\""});
    read.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1',?1,'original')",[original.to_string()]).unwrap();
    let before = metadata(&read);
    assert!(
        !db.delete_reading_session(id, "b", &Cancellation::default())
            .unwrap()
    );
    assert!(
        !db.delete_reading_session("live", "a", &Cancellation::default())
            .unwrap()
    );
    assert_eq!(metadata(&read), before);
    assert!(
        db.delete_reading_session(id, "a", &Cancellation::default())
            .unwrap()
    );
    assert_eq!(rows(&read), 2);
    let mut expected = original;
    expected.as_object_mut().unwrap().remove(id);
    assert_eq!(
        serde_json::from_str::<Value>(&metadata(&read).0).unwrap(),
        expected
    );
    let after = metadata(&read);
    assert!(
        !db.delete_reading_session(id, "a", &Cancellation::default())
            .unwrap()
    );
    assert_eq!(metadata(&read), after);
}
#[test]
fn malformed_context_and_late_delete_failure_roll_back_both_records() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(&path).unwrap();
    read.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1','broken','original')",[]).unwrap();
    assert!(matches!(
        db.delete_reading_session("finished", "a", &Cancellation::default()),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(metadata(&read), ("broken".into(), "original".into()));
    assert_eq!(rows(&read), 3);
    for action in ["RAISE(ABORT,'late failure')", "RAISE(IGNORE)"] {
        read.execute("UPDATE app_metadata SET value='{\"finished\":{\"kept\":true},\"other\":false}',updated_at='original'",[]).unwrap();
        read.execute_batch(&format!("CREATE TRIGGER fail_delete BEFORE DELETE ON reading_sessions BEGIN SELECT {action}; END;")).unwrap();
        let before = metadata(&read);
        assert!(
            db.delete_reading_session("finished", "a", &Cancellation::default())
                .is_err()
        );
        assert_eq!(rows(&read), 3);
        assert_eq!(metadata(&read), before);
        read.execute_batch("DROP TRIGGER fail_delete;").unwrap();
    }
}
#[test]
fn cancellation_and_missing_context_are_safe() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(&path).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.delete_reading_session("finished", "a", &cancel),
        Err(Failure::Cancelled)
    ));
    assert_eq!(rows(&read), 3);
    assert!(
        db.delete_reading_session("finished", "a", &Cancellation::default())
            .unwrap()
    );
    assert_eq!(rows(&read), 2);
    assert_eq!(
        read.query_row(
            "SELECT count(*) FROM app_metadata WHERE key='reading_session_contexts_v1'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}
#[test]
fn retention_also_rolls_back_when_a_delete_is_ignored() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(&path).unwrap();
    read.execute_batch("WITH RECURSIVE n(x) AS(VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<29) INSERT INTO reading_sessions(id,document_id,started_at,ended_at,start_page,end_page,active_seconds) SELECT printf('s-%02d',x),'a','2020-01-01T00:00:00Z','2020-01-01T00:01:00Z',0,1,10 FROM n; INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1','{\"s-00\":{\"kept\":true}}','original'); CREATE TRIGGER ignore_delete BEFORE DELETE ON reading_sessions WHEN OLD.id='s-00' BEGIN SELECT RAISE(IGNORE); END;").unwrap();
    let before = metadata(&read);
    let count = rows(&read);
    assert!(matches!(
        db.trim_reading_history(25, &Cancellation::default()),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(rows(&read), count);
    assert_eq!(metadata(&read), before);
}
