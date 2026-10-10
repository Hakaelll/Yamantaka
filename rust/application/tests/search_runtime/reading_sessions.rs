use super::*;
use yamantaka_application::reading_sessions::lifecycle::ReadingSessionLifecycle;
use yamantaka_application::reading_sessions::*;
use yamantaka_domain::{Failure, reading_sessions::ReaderSessionContext};

fn request(change: ReadingSessionChange, cancel: Cancellation) -> Action {
    Action::ReadingSession(Box::new(ReadingSessionRequest {
        id: "synthetic-session".into(),
        document: "a".into(),
        change,
        cancel,
    }))
}

#[test]
fn actor_retention_replies_keep_cancellation_identity_and_do_not_require_pdf_work() {
    let (runtime, native, _) = setup();
    let token = Cancellation::default();
    runtime
        .submit(Action::TrimReadingHistory(25, token.clone()))
        .unwrap();
    let Update::ReadingHistoryTrimmed(result, reply) = update(&runtime) else {
        panic!("Expected retention reply")
    };
    assert_eq!(result.unwrap(), 0);
    assert!(reply.same_request(&token));
    token.cancel();
    runtime
        .submit(Action::TrimReadingHistory(25, token.clone()))
        .unwrap();
    let Update::ReadingHistoryTrimmed(result, reply) = update(&runtime) else {
        panic!("Expected cancelled retention reply")
    };
    assert!(matches!(result, Err(Failure::Cancelled)));
    assert!(reply.same_request(&token));
    assert!(native.try_recv().is_err());
}

#[test]
fn session_recovery_barrier_works_without_draining_saturated_events_or_releasing_pdf() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let db = Database::open(&path).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES ('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa');").unwrap();
    db.execute_batch("WITH RECURSIVE n(x) AS(VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<299) INSERT INTO reading_sessions(id,document_id,started_at,ended_at,start_page,end_page,active_seconds) SELECT printf('old-%03d',x),'a','2020-01-01T00:00:00Z','2020-01-01T00:01:00Z',0,0,5 FROM n; INSERT INTO reading_sessions(id,document_id,started_at,start_page,end_page,active_seconds) VALUES('retention-live','a','2019-01-01T00:00:00Z',0,0,0); INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_session_contexts_v1','{\"old-000\":{},\"retention-live\":{\"future\":true}}','original');").unwrap();
    let (runtime, native, _) = start_database(db);
    runtime
        .submit(Action::GetDocument("a".into(), Cancellation::default()))
        .unwrap();
    let Update::Document(_, Some(document), _) = update(&runtime) else {
        panic!("Expected document")
    };
    runtime
        .submit(Action::Open(document, None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, held) = next(&native) else {
        panic!("Expected held PDF")
    };
    for _ in 0..127 {
        runtime.submit(Action::Favorite("a".into(), true)).unwrap();
    }
    assert!(matches!(
        runtime.submit(Action::Favorite("a".into(), true)),
        Err(Failure::QueueFull)
    ));
    let mut lifecycle = ReadingSessionLifecycle::default();
    lifecycle
        .begin(
            &yamantaka_domain::reading_sessions::ReadingSessionStart {
                id: "recovery-session",
                document: "a",
                started_at: "2026-10-10T12:00:00Z",
                page: 1,
            },
            Duration::ZERO,
            true,
        )
        .unwrap();
    lifecycle
        .finish(
            "2026-10-10T12:10:00Z".into(),
            2,
            None,
            Duration::from_secs(45),
        )
        .unwrap();
    let (finished, observed) = bounded(1);
    let thread = std::thread::spawn(move || {
        for _ in 0..2 {
            runtime
                .persist_reading_sessions(lifecycle.recovery_commands().unwrap(), 25)
                .unwrap();
        }
        finished.send(runtime).unwrap();
    });
    let runtime = observed
        .recv_timeout(Duration::from_secs(5))
        .expect("Persistence must not await UI draining or PDF completion");
    thread.join().unwrap();
    let read = rusqlite::Connection::open(&path).unwrap();
    let row: (i64, i64, i64, i64) = read
        .query_row(
            "SELECT count(*),start_page,end_page,active_seconds FROM reading_sessions WHERE id='recovery-session'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(row, (1, 1, 2, 30));
    assert_eq!(
        read.query_row("SELECT count(*) FROM reading_sessions", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        26
    );
    let contexts: String = read
        .query_row(
            "SELECT value FROM app_metadata WHERE key='reading_session_contexts_v1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&contexts).unwrap(),
        serde_json::json!({"retention-live":{"future":true}})
    );
    assert!(native.try_recv().is_err());
    held.send(Err(Failure::Cancelled)).unwrap();
    drop(runtime);
}

#[test]
fn session_writes_and_failures_are_scoped_while_a_native_open_is_held() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let db = Database::open(&path).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES ('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa');").unwrap();
    let (runtime, native, _) = start_database(db);
    runtime
        .submit(Action::GetDocument("a".into(), Cancellation::default()))
        .unwrap();
    let Update::Document(_, Some(document), _) = update(&runtime) else {
        panic!("Expected document")
    };
    runtime
        .submit(Action::Open(document, None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, held) = next(&native) else {
        panic!("Expected held open")
    };
    let read = rusqlite::Connection::open(&path).unwrap();
    for cancel_first in [true, false, false] {
        let token = Cancellation::default();
        if cancel_first {
            token.cancel();
        }
        runtime
            .submit(request(
                ReadingSessionChange::Start {
                    started_at: "2026-10-10T12:00:00Z".into(),
                    page: 1,
                },
                token.clone(),
            ))
            .unwrap();
        let Update::ReadingSession(reply) = update(&runtime) else {
            panic!("Expected scoped start")
        };
        assert_eq!(reply.id, "synthetic-session");
        assert_eq!(reply.document, "a");
        assert_eq!(reply.operation, ReadingSessionOperation::Start);
        assert!(reply.cancel.same_request(&token));
        if cancel_first {
            assert!(matches!(reply.result, Err(Failure::Cancelled)));
        } else {
            reply.result.unwrap();
        }
        let count: i64 = read
            .query_row("SELECT count(*) FROM reading_sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, i64::from(!cancel_first));
    }
    for seconds in [99, 99, 100] {
        let token = Cancellation::default();
        runtime
            .submit(request(
                ReadingSessionChange::Finish {
                    ended_at: "2026-10-10T12:10:00Z".into(),
                    page: 2,
                    active_seconds: seconds,
                    context: Some(ReaderSessionContext {
                        zoom_level: 1.5,
                        page_layout: "continuous".into(),
                        reading_mode: "document".into(),
                        reading_theme: "ivory".into(),
                        scroll_offset: 123.5,
                        panel_section: Some("annotations".into()),
                    }),
                },
                token.clone(),
            ))
            .unwrap();
        let Update::ReadingSession(reply) = update(&runtime) else {
            panic!("Expected scoped finish")
        };
        assert_eq!(reply.id, "synthetic-session");
        assert_eq!(reply.document, "a");
        assert_eq!(reply.operation, ReadingSessionOperation::Finish);
        assert!(reply.cancel.same_request(&token));
        if seconds == 100 {
            assert!(matches!(reply.result, Err(Failure::Invalid(_))));
        } else {
            reply.result.unwrap();
        }
    }
    let row: (i64, i64, i64) = read
        .query_row(
            "SELECT start_page,end_page,active_seconds FROM reading_sessions",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(row, (1, 2, 99));
    let context: String = read
        .query_row(
            "SELECT value FROM app_metadata WHERE key='reading_session_contexts_v1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let context: serde_json::Value = serde_json::from_str(&context).unwrap();
    assert_eq!(context["synthetic-session"]["scrollOffset"], 123.5);
    assert_eq!(context["synthetic-session"]["panelSection"], "annotations");
    assert!(
        native.try_recv().is_err(),
        "Session commands must not submit PDF work"
    );
    held.send(Err(Failure::Cancelled)).unwrap();
}
