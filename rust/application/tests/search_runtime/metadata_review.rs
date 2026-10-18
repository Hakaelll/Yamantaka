use super::*;
use yamantaka_application::metadata_review::{MetadataChange, MetadataRequest};
use yamantaka_domain::metadata_review::{ManualMetadata, MetadataMutation};

#[test]
fn evidence_checks_reply_with_their_own_token_without_waiting_for_a_native_open() {
    use yamantaka_application::metadata_source::MetadataSourceRequest;
    use yamantaka_domain::metadata_review::MetadataField;
    let (runtime, native, _notified) = setup();
    runtime
        .submit(Action::Open(reader(), None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, held) = next(&native) else {
        panic!("Expected held native open")
    };
    runtime
        .submit(Action::Metadata(Box::new(MetadataRequest {
            document: "a".into(),
            change: MetadataChange::Read,
            cancel: Cancellation::default(),
        })))
        .unwrap();
    let Update::Metadata(reply) = update(&runtime) else {
        panic!("Expected metadata snapshot")
    };
    let expected = reply.result.unwrap();
    for cancelled in [false, true] {
        let token = Cancellation::default();
        if cancelled {
            token.cancel();
        }
        runtime
            .submit(Action::MetadataSource(Box::new(MetadataSourceRequest {
                expected: Box::new(expected.clone()),
                field: MetadataField::Title,
                cancel: token.clone(),
            })))
            .unwrap();
        let Update::MetadataSource(reply) = update(&runtime) else {
            panic!("Expected scoped source reply")
        };
        assert_eq!(reply.document, "a");
        assert!(reply.cancel.same_request(&token));
        assert!(reply.result.is_err());
        if cancelled {
            assert!(matches!(
                reply.result,
                Err(yamantaka_domain::Failure::Cancelled)
            ));
        }
    }
    assert!(native.try_recv().is_err());
    held.send(Ok(info())).unwrap();
    assert!(matches!(update(&runtime), Update::Opened(..)));
}

#[test]
fn metadata_review_and_corrections_remain_scoped_while_native_open_is_held() {
    let (runtime, native, _notified) = setup();
    runtime
        .submit(Action::Open(reader(), None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, held) = next(&native) else {
        panic!("Expected held PDF open");
    };
    let read = Cancellation::default();
    runtime
        .submit(Action::Metadata(Box::new(MetadataRequest {
            document: "a".into(),
            change: MetadataChange::Read,
            cancel: read.clone(),
        })))
        .unwrap();
    let Update::Metadata(reply) = update(&runtime) else {
        panic!("Expected metadata review");
    };
    assert!(reply.cancel.same_request(&read));
    assert!(!reply.writing);
    let snapshot = reply.result.unwrap();
    assert_eq!(snapshot.title, "Astronomy");
    let save = Cancellation::default();
    runtime
        .submit(Action::Metadata(Box::new(MetadataRequest {
            document: "a".into(),
            change: MetadataChange::Apply {
                expected: Box::new(snapshot.clone()),
                mutation: MetadataMutation::Save(
                    ManualMetadata::new("Étude 学习", "Ada García").unwrap(),
                ),
            },
            cancel: save.clone(),
        })))
        .unwrap();
    let Update::Metadata(reply) = update(&runtime) else {
        panic!("Expected saved correction");
    };
    assert!(reply.cancel.same_request(&save));
    assert!(reply.writing);
    let saved = reply.result.unwrap();
    assert_eq!(saved.title, "Étude 学习");
    assert_eq!(saved.author, "Ada García");
    runtime
        .submit(Action::Metadata(Box::new(MetadataRequest {
            document: "different".into(),
            change: MetadataChange::Apply {
                expected: Box::new(saved.clone()),
                mutation: MetadataMutation::ResetOverrides,
            },
            cancel: Cancellation::default(),
        })))
        .unwrap();
    assert!(matches!(update(&runtime),Update::Metadata(reply) if reply.result.is_err()));
    let cancelled = Cancellation::default();
    cancelled.cancel();
    runtime
        .submit(Action::Metadata(Box::new(MetadataRequest {
            document: "a".into(),
            change: MetadataChange::Apply {
                expected: Box::new(saved),
                mutation: MetadataMutation::ResetOverrides,
            },
            cancel: cancelled.clone(),
        })))
        .unwrap();
    assert!(
        matches!(update(&runtime),Update::Metadata(reply) if reply.cancel.same_request(&cancelled) && matches!(reply.result,Err(yamantaka_domain::Failure::Cancelled)))
    );
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    let Update::Library(documents, 1, _) = update(&runtime) else {
        panic!("Expected unchanged catalogue");
    };
    assert_eq!(documents[0].display_title(), "Étude 学习");
    assert!(native.try_recv().is_err());
    held.send(Ok(info())).unwrap();
    assert!(matches!(update(&runtime), Update::Opened(..)));
}
