use super::*;
use yamantaka_storage::Database;

fn begin(lifecycle: &mut ReadingSessionLifecycle, id: &str, now: u64) {
    lifecycle
        .begin(
            &ReadingSessionStart {
                id,
                document: "a",
                started_at: "2026-10-10T12:00:00Z",
                page: 1,
            },
            Duration::from_secs(now),
            true,
        )
        .unwrap();
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
fn finish(lifecycle: &mut ReadingSessionLifecycle, now: u64) {
    lifecycle
        .finish(
            "2026-10-10T12:10:00Z".into(),
            2,
            Some(context()),
            Duration::from_secs(now),
        )
        .unwrap();
}
fn reply(request: &ReadingSessionRequest, result: Result<()>) -> ReadingSessionReply {
    ReadingSessionReply {
        id: request.id.clone(),
        document: request.document.clone(),
        operation: request.operation(),
        result,
        cancel: request.cancel.clone(),
    }
}

#[test]
fn closing_before_start_acknowledgement_preserves_the_old_session_during_next_reading() {
    let mut lifecycle = ReadingSessionLifecycle::default();
    begin(&mut lifecycle, "first", 0);
    let first_start = lifecycle.next_request().unwrap();
    assert!(lifecycle.next_request().is_none());
    assert!(lifecycle.recovery_commands().is_err());
    lifecycle.observe(Duration::from_secs(10), false, false);
    lifecycle.observe(Duration::from_secs(200), true, false);
    finish(&mut lifecycle, 205);
    begin(&mut lifecycle, "second", 205);
    let second_start = lifecycle.next_request().unwrap();
    assert_eq!(second_start.id, "second");
    lifecycle
        .accept(reply(&first_start, Ok(())))
        .unwrap()
        .unwrap();
    let first_finish = lifecycle.next_request().unwrap();
    assert_eq!(first_finish.id, "first");
    let ReadingSessionChange::Finish {
        active_seconds,
        page,
        context: Some(saved),
        ..
    } = &first_finish.change
    else {
        panic!("Expected completion")
    };
    assert_eq!((*active_seconds, *page), (15, 2));
    assert_eq!(saved.scroll_offset, 123.5);
    lifecycle
        .accept(reply(&first_finish, Ok(())))
        .unwrap()
        .unwrap();
    assert!(lifecycle.accept(reply(&first_finish, Ok(()))).is_none());
    lifecycle
        .accept(reply(&second_start, Ok(())))
        .unwrap()
        .unwrap();
    finish(&mut lifecycle, 210);
    let second_finish = lifecycle.next_request().unwrap();
    let ReadingSessionChange::Finish { active_seconds, .. } = second_finish.change else {
        panic!("Expected completion")
    };
    assert_eq!(active_seconds, 5);
}

#[test]
fn admission_failure_and_cancelled_retries_keep_frozen_payload_and_reject_late_replies() {
    let mut lifecycle = ReadingSessionLifecycle::default();
    begin(&mut lifecycle, "first", 0);
    let denied = lifecycle.next_request().unwrap();
    assert!(lifecycle.submission_failed(&denied, Failure::QueueFull));
    finish(&mut lifecycle, 400);
    assert!(
        lifecycle.next_request().is_none(),
        "Failures require an explicit retry"
    );
    assert!(lifecycle.retry("first"));
    let retried = lifecycle.next_request().unwrap();
    assert!(!retried.cancel.same_request(&denied.cancel));
    assert!(lifecycle.accept(reply(&denied, Ok(()))).is_none());
    let mut wrong = reply(&retried, Ok(()));
    wrong.document = "other".into();
    assert!(lifecycle.accept(wrong).is_none());
    let mut wrong = reply(&retried, Ok(()));
    wrong.operation = ReadingSessionOperation::Finish;
    assert!(lifecycle.accept(wrong).is_none());
    lifecycle.accept(reply(&retried, Ok(()))).unwrap().unwrap();
    let cancelled = lifecycle.next_request().unwrap();
    cancelled.cancel.cancel();
    assert!(matches!(
        lifecycle.accept(reply(&cancelled, Err(Failure::Cancelled))),
        Some(Err(Failure::Cancelled))
    ));
    lifecycle.observe(Duration::from_secs(1000), true, true);
    assert!(lifecycle.retry("first"));
    let retried_finish = lifecycle.next_request().unwrap();
    assert!(lifecycle.accept(reply(&cancelled, Ok(()))).is_none());
    let ReadingSessionChange::Finish {
        ended_at,
        active_seconds,
        context: Some(saved),
        ..
    } = retried_finish.change
    else {
        panic!("Expected completion")
    };
    assert_eq!(ended_at, "2026-10-10T12:10:00Z");
    assert_eq!(active_seconds, 30);
    assert_eq!(saved.zoom_level, 1.5);
}

#[test]
fn invalid_completion_preserves_live_activity_and_full_queue_never_discards_sessions() {
    let mut lifecycle = ReadingSessionLifecycle::default();
    begin(&mut lifecycle, "first", 0);
    let mut invalid = context();
    invalid.scroll_offset = f64::NAN;
    assert!(
        lifecycle
            .finish(
                "2026-10-10T12:10:00Z".into(),
                2,
                Some(invalid),
                Duration::from_secs(5)
            )
            .is_err()
    );
    assert!(
        lifecycle
            .finish(
                "2026-10-10T11:00:00Z".into(),
                2,
                None,
                Duration::from_secs(6)
            )
            .is_err()
    );
    assert!(
        lifecycle
            .begin(
                &ReadingSessionStart {
                    id: "other",
                    document: "a",
                    started_at: "2026-10-10T12:00:00Z",
                    page: 0
                },
                Duration::ZERO,
                true
            )
            .is_err()
    );
    finish(&mut lifecycle, 7);
    for i in 1..MAX_UNCOMPLETED_SESSIONS {
        begin(&mut lifecycle, &format!("session-{i}"), 10);
        finish(&mut lifecycle, 11);
    }
    assert!(!lifecycle.has_capacity());
    assert!(matches!(
        lifecycle.begin(
            &ReadingSessionStart {
                id: "overflow",
                document: "a",
                started_at: "2026-10-10T12:00:00Z",
                page: 0
            },
            Duration::ZERO,
            true
        ),
        Err(Failure::QueueFull)
    ));
    let recovered = lifecycle.recovery_commands().unwrap();
    assert_eq!(recovered.len(), MAX_UNCOMPLETED_SESSIONS * 2);
    assert_eq!(recovered[0].id, "first");
    let ReadingSessionChange::Finish { active_seconds, .. } = recovered[1].change else {
        panic!("Expected completion")
    };
    assert_eq!(active_seconds, 7);
}

#[test]
fn sqlite_completion_failure_can_retry_and_recovery_replay_is_idempotent() {
    let mut db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES ('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa'); CREATE TRIGGER fail_context BEFORE INSERT ON app_metadata WHEN NEW.key='reading_session_contexts_v1' BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
    let mut lifecycle = ReadingSessionLifecycle::default();
    begin(&mut lifecycle, "first", 0);
    let start = lifecycle.next_request().unwrap();
    lifecycle.accept(start.execute(&mut db)).unwrap().unwrap();
    finish(&mut lifecycle, 20);
    let finish = lifecycle.next_request().unwrap();
    assert!(lifecycle.accept(finish.execute(&mut db)).unwrap().is_err());
    assert!(lifecycle.next_request().is_none());
    let recovery = lifecycle.recovery_commands().unwrap();
    db.execute_batch("DROP TRIGGER fail_context").unwrap();
    for _ in 0..2 {
        for command in &recovery {
            ReadingSessionRequest {
                id: command.id.clone(),
                document: command.document.clone(),
                change: command.change.clone(),
                cancel: Cancellation::default(),
            }
            .execute(&mut db)
            .result
            .unwrap();
        }
    }
    assert!(lifecycle.retry("first"));
    let finish = lifecycle.next_request().unwrap();
    lifecycle.accept(finish.execute(&mut db)).unwrap().unwrap();
    assert!(lifecycle.recovery_commands().unwrap().is_empty());
}
