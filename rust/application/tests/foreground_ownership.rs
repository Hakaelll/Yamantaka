use std::{path::PathBuf, time::Duration};
use yamantaka_application::runtime::{Action, Runtime, Update, WriteKind};
use yamantaka_domain::{Cancellation, Failure, LibraryQuery, LibraryRepository, PdfEngine};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;

#[test]
fn password_failure_keeps_previous_reader_pinned_against_real_background_eviction() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = root
        .join("build/rust-fixtures/textual.pdf")
        .canonicalize()
        .unwrap();
    let protected = root
        .join("build/rust-fixtures/protected.pdf")
        .canonicalize()
        .unwrap();
    let originals = [
        std::fs::read(&text).unwrap(),
        std::fs::read(&protected).unwrap(),
    ];
    let db = Database::in_memory_at(11).unwrap();
    for (id, path) in [("reader", &text), ("protected", &protected)] {
        let path = path.to_string_lossy().replace('\'', "''");
        db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES ('{id}','{path}','{id}','synthetic.pdf',2,1,'now','now','now','{id}');")).unwrap();
    }
    let documents = db.query(&LibraryQuery::default()).unwrap();
    let engine = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let runtime = Runtime::start(db, engine.clone(), || {}).unwrap();
    let reader = documents
        .iter()
        .find(|doc| doc.id == "reader")
        .unwrap()
        .clone();
    runtime
        .submit(Action::Open(reader, None, Cancellation::default()))
        .unwrap();
    let Update::Opened(document, _, token) = update(&runtime) else {
        panic!("reader should open")
    };
    runtime
        .submit(Action::ConfirmOpen(document.id, token))
        .unwrap();
    assert!(matches!(
        update(&runtime),
        Update::Committed(_, WriteKind::VisibleDocuments)
    ));
    let document = documents
        .iter()
        .find(|doc| doc.id == "protected")
        .unwrap()
        .clone();
    runtime
        .submit(Action::Open(document, None, Cancellation::default()))
        .unwrap();
    assert!(matches!(
        update(&runtime),
        Update::OpenFailed(_, Failure::PasswordRequired, _)
    ));
    // These native background opens deliberately pressure both hot slots. They
    // share the production engine with the actor, just as its index controller does.
    engine
        .open(
            "background-one".into(),
            text.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    engine
        .open(
            "background-two".into(),
            text.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    assert!(
        !engine
            .text("reader".into(), 0, Cancellation::default())
            .unwrap()
            .is_empty()
    );
    assert!(
        engine
            .text("background-one".into(), 0, Cancellation::default())
            .is_err()
    );
    drop(runtime);
    drop(engine);
    assert_eq!(std::fs::read(text).unwrap(), originals[0]);
    assert_eq!(std::fs::read(protected).unwrap(), originals[1]);
}

fn update(runtime: &Runtime) -> Update {
    runtime
        .updates
        .recv_timeout(Duration::from_secs(5))
        .expect("actor terminal response")
}
