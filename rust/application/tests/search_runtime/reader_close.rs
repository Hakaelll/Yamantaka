use super::*;
use yamantaka_application::reader_close::ReaderCloseRequest;
use yamantaka_domain::Failure;

fn close(runtime: &Runtime, document: &str, cancel: Cancellation) {
    runtime
        .submit(Action::Close(ReaderCloseRequest {
            document: document.to_owned(),
            cancel,
        }))
        .unwrap();
}

#[test]
fn cancelled_admission_skips_native_work_and_late_cancellation_preserves_the_close_result() {
    let (runtime, native, _) = setup();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    close(&runtime, "reader", cancelled.clone());
    let Update::ReaderClosed(reply) = update(&runtime) else {
        panic!("Expected cancelled close reply");
    };
    assert_eq!(reply.document, "reader");
    assert!(reply.cancel.same_request(&cancelled));
    assert!(matches!(reply.result, Err(Failure::Cancelled)));
    assert!(native.try_recv().is_err());
    let invalid = Cancellation::default();
    close(&runtime, "", invalid.clone());
    let Update::ReaderClosed(reply) = update(&runtime) else {
        panic!("Expected invalid close reply");
    };
    assert!(reply.document.is_empty());
    assert!(reply.cancel.same_request(&invalid));
    assert!(matches!(reply.result, Err(Failure::Invalid(_))));
    assert!(native.try_recv().is_err());
    let id = "legacy\"quoted\\Ω";
    let token = Cancellation::default();
    close(&runtime, id, token.clone());
    let Native::Close(actual, held) = next(&native) else {
        panic!("Expected native close");
    };
    assert_eq!(actual, id);
    runtime.submit(Action::Favorite("a".into(), true)).unwrap();
    committed(&runtime, WriteKind::Favorite);
    token.cancel();
    held.send(Ok(())).unwrap();
    let Update::ReaderClosed(reply) = update(&runtime) else {
        panic!("Expected completed native close");
    };
    assert_eq!(reply.document, id);
    assert!(reply.cancel.same_request(&token));
    assert!(reply.cancel.is_cancelled());
    reply.result.unwrap();
}

#[test]
fn closing_revokes_an_older_open_and_same_document_reopening_waits_for_native_completion() {
    let (runtime, native, _) = setup();
    let original = Cancellation::default();
    runtime
        .submit(Action::Open(reader(), None, original.clone()))
        .unwrap();
    let Native::Open(id, _, opened) = next(&native) else {
        panic!("Expected original open");
    };
    assert_eq!(id, "reader");
    let token = Cancellation::default();
    close(&runtime, "reader", token.clone());
    let Native::Close(id, closed) = next(&native) else {
        panic!("Expected close");
    };
    assert_eq!(id, "reader");
    query_barrier(&runtime);
    assert!(original.is_cancelled());
    let replacement = Cancellation::default();
    runtime
        .submit(Action::Open(reader(), None, replacement.clone()))
        .unwrap();
    query_barrier(&runtime);
    assert!(
        native.try_recv().is_err(),
        "Reopening must wait for the older close"
    );
    opened.send(Ok(info())).unwrap();
    let Update::OpenFailed(document, error, cancel) = update(&runtime) else {
        panic!("Expected revoked old opening");
    };
    assert_eq!(document.id, "reader");
    assert!(matches!(error, Failure::Cancelled));
    assert!(cancel.same_request(&original));
    closed.send(Ok(())).unwrap();
    let Update::ReaderClosed(reply) = update(&runtime) else {
        panic!("Expected scoped closure");
    };
    assert!(reply.cancel.same_request(&token));
    reply.result.unwrap();
    let Native::Open(id, cancel, opened) = next(&native) else {
        panic!("Expected admitted replacement");
    };
    assert_eq!(id, "reader");
    assert!(cancel.same_request(&replacement));
    assert!(!cancel.is_cancelled());
    opened.send(Ok(info())).unwrap();
    let Update::Opened(document, _, cancel) = update(&runtime) else {
        panic!("Expected replacement ownership offer");
    };
    assert_eq!(document.id, "reader");
    assert!(cancel.same_request(&replacement));
    assert!(!cancel.is_cancelled());
    runtime
        .submit(Action::ConfirmOpen(document.id.clone(), cancel))
        .unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
}

#[test]
fn native_failure_and_disconnection_keep_visible_ownership_and_publish_scoped_errors() {
    let pins = Arc::new(Mutex::new(Vec::new()));
    let db = Database::in_memory_at(11).unwrap();
    let (runtime, native, _) = start_database_with_pins(db, pins.clone());
    runtime
        .submit(Action::Open(reader(), None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, opened) = next(&native) else {
        panic!("Expected open");
    };
    opened.send(Ok(info())).unwrap();
    let Update::Opened(document, _, cancel) = update(&runtime) else {
        panic!("Expected ownership offer");
    };
    runtime
        .submit(Action::ConfirmOpen(document.id.clone(), cancel))
        .unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    for outcome in [0, 1, 2] {
        let token = Cancellation::default();
        close(&runtime, "reader", token.clone());
        let Native::Close(_, closed) = next(&native) else {
            panic!("Expected close");
        };
        if outcome == 1 {
            drop(closed);
        } else {
            let error = if outcome == 2 {
                Failure::Cancelled
            } else {
                Failure::Pdf("Synthetic close failure".into())
            };
            closed.send(Err(error)).unwrap();
        }
        let Update::ReaderClosed(reply) = update(&runtime) else {
            panic!("Expected scoped native error");
        };
        assert_eq!(reply.document, "reader");
        assert!(reply.cancel.same_request(&token));
        if outcome == 2 {
            assert!(matches!(reply.result, Err(Failure::Cancelled)));
        } else {
            assert!(matches!(reply.result, Err(Failure::Pdf(_))));
        }
    }
    let mut other = reader();
    other.id = "other-reader".into();
    runtime
        .submit(Action::Open(other, None, Cancellation::default()))
        .unwrap();
    let Native::Open(id, _, opened) = next(&native) else {
        panic!("Expected another reader");
    };
    assert_eq!(id, "other-reader");
    assert_eq!(
        pins.lock().unwrap().last().unwrap(),
        &vec!["reader".to_owned(), "other-reader".to_owned()]
    );
    opened.send(Ok(info())).unwrap();
    let Update::Opened(document, _, cancel) = update(&runtime) else {
        panic!("Expected another ownership offer");
    };
    runtime
        .submit(Action::ConfirmOpen(document.id.clone(), cancel))
        .unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
}
