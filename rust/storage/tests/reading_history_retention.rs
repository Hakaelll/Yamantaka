use rusqlite::Connection;
use serde_json::{Value, json};
use yamantaka_domain::{Cancellation, Failure, reading_sessions::ReadingSessionRepository};
use yamantaka_storage::{Database, preferences::Preferences};

fn seed(db: &Database, count: usize) {
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa');").unwrap();
    db.execute_batch(&format!("WITH RECURSIVE n(x) AS(VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<{}) INSERT INTO reading_sessions(id,document_id,started_at,ended_at,start_page,end_page,active_seconds) SELECT printf('s-%04d',x),'a',printf('2020-01-01T%02d:%02d:%02dZ',x/3600,(x/60)%60,x%60),'2021-01-01T00:00:00Z',0,1,10 FROM n; INSERT INTO reading_sessions(id,document_id,started_at,start_page,end_page,active_seconds) VALUES('active','a','2019-01-01T00:00:00Z',0,0,0);",count-1)).unwrap();
}
fn count(read: &Connection) -> i64 {
    read.query_row("SELECT count(*) FROM reading_sessions", [], |r| r.get(0))
        .unwrap()
}
fn contexts(read: &Connection) -> String {
    read.query_row(
        "SELECT value FROM app_metadata WHERE key='reading_session_contexts_v1'",
        [],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn pruning_is_atomic_and_preserves_open_sessions_and_unknown_json_types() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db, 35);
    let read = Connection::open(&path).unwrap();
    let unusual_id = "legacy\"quoted\\session";
    read.execute(
        "UPDATE reading_sessions SET id=?1 WHERE id='s-0000'",
        [unusual_id],
    )
    .unwrap();
    let mut metadata = json!({"active":{"future":true},"unknownArray":[true,null,"Unicode Ω"],"unknownBoolean":true,"unknownFalse":false,"unknownNull":null,"unknownText":"quoted \"value\"","unknownNumber":1.25});
    for i in 0..35 {
        let id = if i == 0 {
            unusual_id.into()
        } else {
            format!("s-{i:04}")
        };
        metadata
            .as_object_mut()
            .unwrap()
            .insert(id, json!({"future":{"bool":false,"array":[1,"Ω"]}}));
    }
    read.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1',?1,'original')", [metadata.to_string()]).unwrap();
    assert_eq!(
        db.trim_reading_history(0, &Cancellation::default())
            .unwrap(),
        10
    );
    assert_eq!(count(&read), 26);
    assert_eq!(
        read.query_row(
            "SELECT count(*) FROM reading_sessions WHERE id='active'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    for i in 0..10 {
        let id = if i == 0 {
            unusual_id.into()
        } else {
            format!("s-{i:04}")
        };
        metadata.as_object_mut().unwrap().remove(&id);
    }
    assert_eq!(
        serde_json::from_str::<Value>(&contexts(&read)).unwrap(),
        metadata
    );
    let after = contexts(&read);
    assert_eq!(
        db.trim_reading_history(25, &Cancellation::default())
            .unwrap(),
        0
    );
    assert_eq!(contexts(&read), after);
}

#[test]
fn corruption_and_a_late_delete_failure_preserve_rows_and_metadata_for_recovery() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db, 35);
    let read = Connection::open(&path).unwrap();
    read.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1','[broken','original')", []).unwrap();
    assert!(matches!(
        db.trim_reading_history(25, &Cancellation::default()),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(count(&read), 36);
    assert_eq!(contexts(&read), "[broken");
    let original = " { \"s-0000\" : { \"future\" : true }, \"active\" : {} } ";
    read.execute(
        "UPDATE app_metadata SET value=?1 WHERE key='reading_session_contexts_v1'",
        [original],
    )
    .unwrap();
    db.execute_batch("CREATE TRIGGER fail_delete BEFORE DELETE ON reading_sessions BEGIN SELECT RAISE(ABORT,'synthetic late failure'); END;").unwrap();
    assert!(
        db.trim_reading_history(25, &Cancellation::default())
            .is_err()
    );
    assert_eq!(count(&read), 36);
    assert_eq!(contexts(&read), original);
    assert_eq!(
        read.query_row(
            "SELECT updated_at FROM app_metadata WHERE key='reading_session_contexts_v1'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "original"
    );
    db.execute_batch("DROP TRIGGER fail_delete").unwrap();
    assert_eq!(
        db.trim_reading_history(25, &Cancellation::default())
            .unwrap(),
        10
    );
    assert_eq!(
        serde_json::from_str::<Value>(&contexts(&read)).unwrap(),
        json!({"active":{}})
    );
}

#[test]
fn cancellation_and_upper_retention_bound_preserve_unfinished_rows() {
    let mut db = Database::in_memory_at(11).unwrap();
    seed(&db, 1005);
    let token = Cancellation::default();
    token.cancel();
    assert!(matches!(
        db.trim_reading_history(25, &token),
        Err(Failure::Cancelled)
    ));
    assert_eq!(
        db.trim_reading_history(usize::MAX, &Cancellation::default())
            .unwrap(),
        5
    );
    assert_eq!(
        db.trim_reading_history(1000, &Cancellation::default())
            .unwrap(),
        0
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("snapshot.sqlite3");
    db.snapshot(&path).unwrap();
    assert_eq!(count(&Connection::open(path).unwrap()), 1001);
}

#[test]
fn imported_retention_limits_are_clamped_without_rewriting_preferences() {
    let mut preferences = Preferences::default();
    assert_eq!(preferences.reading_history_limit(), 250);
    for (value, expected) in [
        (json!(-1), 25),
        (json!(50), 50),
        (json!(u64::MAX), 1000),
        (json!("future"), 250),
    ] {
        preferences
            .legacy
            .insert("readingHistoryLimit".into(), value.clone());
        assert_eq!(preferences.reading_history_limit(), expected);
        assert_eq!(preferences.legacy["readingHistoryLimit"], value);
        assert!(preferences.rust.is_empty());
    }
    preferences
        .rust
        .insert("readingHistoryLimit".into(), json!(75));
    assert_eq!(preferences.reading_history_limit(), 75);
}
