use rusqlite::Connection;
use yamantaka_domain::{Cancellation, Failure, reading_sessions::*};
use yamantaka_storage::Database;

fn seed(db: &Database) {
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf',3,1,'now','now','now','fb');").unwrap();
}
fn context() -> ReaderSessionContext {
    ReaderSessionContext {
        zoom_level: 1.5,
        page_layout: "continuous".into(),
        reading_mode: "document".into(),
        reading_theme: "ivory".into(),
        scroll_offset: 123.5,
        panel_section: Some("annotations".into()),
    }
}
fn start() -> ReadingSessionStart<'static> {
    ReadingSessionStart {
        id: "synthetic-session",
        document: "a",
        started_at: "2026-10-10T12:00:00Z",
        page: 1,
    }
}
fn finish(context: Option<&ReaderSessionContext>) -> ReadingSessionFinish<'_> {
    ReadingSessionFinish {
        id: "synthetic-session",
        document: "a",
        ended_at: "2026-10-10T12:10:00Z",
        page: 2,
        active_seconds: 99,
        context,
    }
}

#[test]
fn starts_and_finishes_are_retryable_and_preserve_legacy_context_extensions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let cancel = Cancellation::default();
    db.start_reading_session(&start(), &cancel).unwrap();
    db.start_reading_session(&start(), &cancel).unwrap();
    db.execute_batch(r#"INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1','{"older":{"future":"kept"},"synthetic-session":{"futureField":{"enabled":true},"zoomLevel":1}}','original');"#).unwrap();
    let c = context();
    let command = finish(Some(&c));
    db.finish_reading_session(&command, &cancel).unwrap();
    db.finish_reading_session(&command, &cancel).unwrap();
    let read = Connection::open(&path).unwrap();
    let session:(i64,String,u32,u32,i64)=read.query_row("SELECT (SELECT COUNT(*) FROM reading_sessions),ended_at,start_page,end_page,active_seconds FROM reading_sessions",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(session, (1, "2026-10-10T12:10:00.000000Z".into(), 1, 2, 99));
    let raw: String = read
        .query_row(
            "SELECT value FROM app_metadata WHERE key='reading_session_contexts_v1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(value["older"]["future"], "kept");
    assert_eq!(value["synthetic-session"]["futureField"]["enabled"], true);
    assert_eq!(value["synthetic-session"]["zoomLevel"], 1.5);
    assert_eq!(value["synthetic-session"]["scrollOffset"], 123.5);
    assert_eq!(value["synthetic-session"]["panelSection"], "annotations");
    let mut collision = start();
    collision.document = "b";
    assert!(matches!(
        db.start_reading_session(&collision, &cancel),
        Err(Failure::Invalid(_))
    ));
    let mut changed = finish(Some(&c));
    changed.active_seconds = 100;
    assert!(matches!(
        db.finish_reading_session(&changed, &cancel),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(
        read.query_row("SELECT active_seconds FROM reading_sessions", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        99
    );
}

#[test]
fn malformed_contexts_and_late_metadata_failures_never_partially_finish_a_session() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let cancel = Cancellation::default();
    db.start_reading_session(&start(), &cancel).unwrap();
    let c = context();
    let command = finish(Some(&c));
    db.execute_batch("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1','malformed','original');").unwrap();
    assert!(matches!(
        db.finish_reading_session(&command, &cancel),
        Err(Failure::Invalid(_))
    ));
    let read = Connection::open(&path).unwrap();
    assert!(
        read.query_row("SELECT ended_at FROM reading_sessions", [], |r| r
            .get::<_, Option<String>>(0))
            .unwrap()
            .is_none()
    );
    assert_eq!(
        read.query_row("SELECT value FROM app_metadata", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "malformed"
    );
    db.execute_batch("DELETE FROM app_metadata; CREATE TRIGGER reject_session_context BEFORE INSERT ON app_metadata WHEN NEW.key='reading_session_contexts_v1' BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
    assert!(matches!(
        db.finish_reading_session(&command, &cancel),
        Err(Failure::Storage(_))
    ));
    assert!(
        read.query_row("SELECT ended_at FROM reading_sessions", [], |r| r
            .get::<_, Option<String>>(0))
            .unwrap()
            .is_none()
    );
    db.execute_batch("DROP TRIGGER reject_session_context;")
        .unwrap();
    db.finish_reading_session(&command, &cancel).unwrap();
    assert!(
        read.query_row("SELECT ended_at FROM reading_sessions", [], |r| r
            .get::<_, Option<String>>(0))
            .unwrap()
            .is_some()
    );
}

#[test]
fn validation_cancellation_and_duration_limits_preserve_session_identity() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.start_reading_session(&start(), &cancel),
        Err(Failure::Cancelled)
    ));
    let cancel = Cancellation::default();
    db.start_reading_session(&start(), &cancel).unwrap();
    let mut c = context();
    c.zoom_level = f64::NAN;
    assert!(matches!(
        db.finish_reading_session(&finish(Some(&c)), &cancel),
        Err(Failure::Invalid(_))
    ));
    let mut command = finish(None);
    command.ended_at = "2026-10-09T12:00:00Z";
    assert!(matches!(
        db.finish_reading_session(&command, &cancel),
        Err(Failure::Invalid(_))
    ));
    command = finish(None);
    command.document = "b";
    assert!(matches!(
        db.finish_reading_session(&command, &cancel),
        Err(Failure::Invalid(_))
    ));
    command = finish(None);
    command.active_seconds = u64::MAX;
    db.finish_reading_session(&command, &cancel).unwrap();
    assert_eq!(
        Connection::open(path)
            .unwrap()
            .query_row("SELECT active_seconds FROM reading_sessions", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        86400
    );
}

#[test]
fn session_contract_keeps_each_historical_schema_compatible_after_migration() {
    for version in 1..=11 {
        let mut db = Database::in_memory_at(version).unwrap();
        seed(&db);
        db.execute_batch("INSERT INTO reading_sessions(id,document_id,started_at,ended_at,start_page,end_page,active_seconds) VALUES('legacy','a','2020-01-01T00:00:00Z','2020-01-01T00:01:00Z',0,1,40);").unwrap();
        db.migrate_current().unwrap();
        let cancel = Cancellation::default();
        db.start_reading_session(&start(), &cancel).unwrap();
        db.finish_reading_session(&finish(Some(&context())), &cancel)
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("verified.sqlite3");
        db.snapshot(&path).unwrap();
        let legacy: (String,String,String,u32,u32,i64) = Connection::open(path).unwrap().query_row("SELECT document_id,started_at,ended_at,start_page,end_page,active_seconds FROM reading_sessions WHERE id='legacy'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).unwrap();
        assert_eq!(
            legacy,
            (
                "a".into(),
                "2020-01-01T00:00:00Z".into(),
                "2020-01-01T00:01:00Z".into(),
                0,
                1,
                40
            )
        );
    }
}
