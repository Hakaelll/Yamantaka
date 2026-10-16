use rusqlite::Connection;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use yamantaka_application::runtime::{Action, Runtime, Update};
use yamantaka_domain::{
    Cancellation, Failure, LibraryQuery, LibraryRepository, NewDocument, PdfEngine,
    front_matter::ImportAnalysis, pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn worker() -> PdfWorker {
    PdfWorker::start(
        &root().join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap()
}
fn source(name: &str) -> PathBuf {
    root().join(format!("build/rust-fixtures/{name}.pdf"))
}

#[test]
fn indexing_cannot_close_a_reader_with_a_legacy_temporary_looking_catalogue_id() {
    use yamantaka_application::search_index::{IndexCommand, IndexPhase, SearchIndexController};
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("library.sqlite3");
    let mut database = Database::open(&path).unwrap();
    let engine = worker();
    let governor = engine.memory_governor();
    let reader_path = source("textual");
    let index_path = source("front-matter-missing");
    let original_reader = fs::read(&reader_path).unwrap();
    let original_index = fs::read(&index_path).unwrap();
    let reader = engine
        .open(
            "index-1-a".into(),
            reader_path.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    engine.set_visible_documents(&["index-1-a".into()]).unwrap();
    let db = Connection::open(&path).unwrap();
    for (id, path, pages) in [
        ("a", &index_path, 2),
        ("index-1-a", &reader_path, reader.pages.len()),
    ] {
        db.execute("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES(?1,?2,?2,?1,?3,1,'now','now','now',?1)", rusqlite::params![id, path.to_str().unwrap(), u32::try_from(pages).unwrap()]).unwrap();
    }
    let mut controller = SearchIndexController::default();
    controller
        .command(IndexCommand::Enable(true), &mut database)
        .unwrap();
    controller
        .command(IndexCommand::Start { rebuild: false }, &mut database)
        .unwrap();
    let due = Instant::now() + Duration::from_secs(20);
    while controller.progress().phase != IndexPhase::Completed {
        assert!(Instant::now() < due, "Indexing did not finish");
        controller.step(&mut database, &engine).unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(controller.progress().failed_documents, 0);
    assert_eq!(controller.progress().completed_documents, 2);
    assert!(
        engine
            .text("index-1-a".into(), 0, Cancellation::default())
            .unwrap()
            .contains("Yamantaka Rust")
    );
    assert_eq!(fs::read(reader_path).unwrap(), original_reader);
    assert_eq!(fs::read(index_path).unwrap(), original_index);
    drop(reader);
    drop(engine);
    assert_eq!(governor.usage().total, 0);
}

fn wait(runtime: &Runtime) -> Update {
    let due = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(
            Instant::now() < due,
            "Import did not produce a terminal event"
        );
        match runtime.updates.recv_timeout(Duration::from_millis(100)) {
            Ok(update) => return update,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("Import actor stopped: {error}"),
        }
    }
}

#[test]
fn real_import_actor_saves_proposals_author_evidence_and_matching_source_in_one_catalogue() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("library.sqlite3");
    let database = Database::open(&path).unwrap();
    let engine = worker();
    let governor = engine.memory_governor();
    let runtime = Runtime::start(database, engine, || {}).unwrap();
    let paths: Vec<_> = [
        "front-matter-missing",
        "front-matter-conflict",
        "front-matter-logo",
    ]
    .into_iter()
    .map(source)
    .collect();
    let originals: Vec<_> = paths.iter().map(|path| fs::read(path).unwrap()).collect();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Import(paths.clone(), cancel.clone()))
        .unwrap();
    assert!(matches!(wait(&runtime),Update::Imported(3,token) if token.same_request(&cancel)));
    assert_eq!(runtime.import_revision(), 3);
    runtime
        .submit(Action::Query(
            LibraryQuery::default(),
            Cancellation::default(),
        ))
        .unwrap();
    let Update::Library(documents, count, _) = wait(&runtime) else {
        panic!("Expected catalogue query")
    };
    assert_eq!(count, 3);
    assert_eq!(documents.len(), 3);
    let db = Connection::open(&path).unwrap();
    let missing = documents
        .iter()
        .find(|document| document.file_name == "front-matter-missing.pdf")
        .unwrap();
    assert_eq!(missing.display_title(), "front-matter-missing.pdf");
    assert_eq!(missing.author.as_deref(), Some("Ada Lovelace"));
    let stored:(String,Option<String>,Option<String>,String,String)=db.query_row("SELECT e.status,e.suggested_title,e.suggested_author,d.source_title,d.source_author FROM document_enrichment e JOIN documents d ON d.id=e.document_id WHERE d.id=?",[&missing.id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?))).unwrap();
    assert_eq!(
        stored,
        (
            "review".into(),
            Some("Geometry of Light".into()),
            None,
            "Untitled".into(),
            "Unknown".into()
        )
    );
    let encoded: String = db
        .query_row(
            "SELECT value FROM app_metadata WHERE key=?",
            [format!("rust.frontMatterEvidence.v1:{}", missing.id)],
            |row| row.get(0),
        )
        .unwrap();
    let evidence: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(evidence["title"]["kind"], "review");
    assert_eq!(evidence["title"]["candidate"]["page"], 0);
    assert_eq!(evidence["author"]["kind"], "autofill");
    let conflict = documents
        .iter()
        .find(|document| document.file_name == "front-matter-conflict.pdf")
        .unwrap();
    assert_eq!(conflict.title.as_deref(), Some("Old project title"));
    assert_eq!(conflict.author.as_deref(), Some("Old Person"));
    let proposals: (String, String) = db
        .query_row(
            "SELECT suggested_title,suggested_author FROM document_enrichment WHERE document_id=?",
            [&conflict.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        proposals,
        ("Geometry of Light".into(), "Ada Lovelace".into())
    );
    let matched = documents
        .iter()
        .find(|document| document.file_name == "front-matter-logo.pdf")
        .unwrap();
    assert_eq!(matched.title.as_deref(), Some("Geometry of Light"));
    let status: String = db
        .query_row(
            "SELECT status FROM document_enrichment WHERE document_id=?",
            [&matched.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "completed");
    for (path, original) in paths.iter().zip(originals) {
        assert_eq!(fs::read(path).unwrap(), original);
    }
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn evidence_write_failure_rolls_back_the_document_and_never_advances_catalogue_revision() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("library.sqlite3");
    let database = Database::open(&path).unwrap();
    database.execute_batch("CREATE TRIGGER fail_evidence BEFORE INSERT ON app_metadata WHEN NEW.key LIKE 'rust.frontMatterEvidence.v1:%' BEGIN SELECT RAISE(FAIL,'synthetic evidence failure'); END;").unwrap();
    let engine = worker();
    let governor = engine.memory_governor();
    let runtime = Runtime::start(database, engine, || {}).unwrap();
    runtime
        .submit(Action::Import(
            vec![source("front-matter-missing")],
            Cancellation::default(),
        ))
        .unwrap();
    assert!(matches!(
        wait(&runtime),
        Update::Failed(Failure::Storage(_), _)
    ));
    assert_eq!(runtime.import_revision(), 0);
    let db = Connection::open(path).unwrap();
    for table in ["documents", "document_enrichment", "app_metadata"] {
        let count: i64 = db
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn ignored_or_mutated_import_records_are_detected_before_the_atomic_commit() {
    for trigger in [
        "CREATE TRIGGER corrupt AFTER INSERT ON document_enrichment BEGIN UPDATE document_enrichment SET suggested_title='wrong'; END;",
        "CREATE TRIGGER ignore_analysis BEFORE INSERT ON document_enrichment BEGIN SELECT RAISE(IGNORE); END;",
        "CREATE TRIGGER corrupt_evidence AFTER INSERT ON app_metadata BEGIN UPDATE app_metadata SET value='{}' WHERE key=NEW.key; END;",
        "CREATE TRIGGER corrupt_analysis_late AFTER INSERT ON app_metadata BEGIN UPDATE document_enrichment SET suggested_title='wrong'; END;",
        "CREATE TRIGGER corrupt_path AFTER INSERT ON documents BEGIN UPDATE documents SET path='wrong.pdf' WHERE id=NEW.id; END;",
    ] {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("library.sqlite3");
        let database = Database::open(&path).unwrap();
        database.execute_batch(trigger).unwrap();
        let engine = worker();
        let governor = engine.memory_governor();
        let runtime = Runtime::start(database, engine, || {}).unwrap();
        runtime
            .submit(Action::Import(
                vec![source("front-matter-missing")],
                Cancellation::default(),
            ))
            .unwrap();
        assert!(
            matches!(wait(&runtime), Update::Failed(Failure::Storage(_), _)),
            "Changed or ignored import was committed"
        );
        assert_eq!(runtime.import_revision(), 0);
        let db = Connection::open(path).unwrap();
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        drop(runtime);
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn bounded_analysis_failure_keeps_a_valid_basic_import_and_a_redacted_retry_state() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("library.sqlite3");
    let engine = worker();
    let governor = engine.memory_governor();
    let runtime = Runtime::start(Database::open(&path).unwrap(), engine, || {}).unwrap();
    runtime
        .submit(Action::Import(
            vec![source("front-matter-work-limit")],
            Cancellation::default(),
        ))
        .unwrap();
    assert!(matches!(wait(&runtime), Update::Imported(1, _)));
    let db = Connection::open(path).unwrap();
    let stored:(String,String,String,String)=db.query_row("SELECT d.title,d.author,e.status,e.error_code FROM documents d JOIN document_enrichment e ON e.document_id=d.id",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).unwrap();
    assert_eq!(
        stored,
        (
            "Native work limit".into(),
            "Known Institute".into(),
            "failed".into(),
            "front_matter_work_limit".into()
        )
    );
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn native_source_snapshot_rejects_changed_header_even_with_preserved_size_and_time() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("source.pdf");
    fs::copy(source("front-matter-missing"), &path).unwrap();
    let engine = worker();
    let info = engine
        .open("source".into(), path.clone(), None, Cancellation::default())
        .unwrap();
    let revision = engine
        .try_source_revision("source".into(), Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    let mut bytes = fs::read(&path).unwrap();
    bytes[7] = if bytes[7] == b'7' { b'6' } else { b'7' };
    fs::write(&path, &bytes).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(revision.modified))
        .unwrap();
    assert_eq!(fs::metadata(&path).unwrap().len(), revision.size);
    assert_eq!(
        engine
            .try_source_revision("source".into(), Cancellation::default())
            .unwrap()
            .wait()
            .unwrap(),
        revision
    );
    let document = NewDocument {
        path: path.clone(),
        info,
        source: revision,
        analysis: ImportAnalysis::NotRequested,
    };
    let mut database = Database::in_memory_at(11).unwrap();
    assert!(matches!(
        database.import(&document),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(database.count(&LibraryQuery::default()).unwrap(), 0);
    assert_eq!(fs::read(path).unwrap(), bytes);
}
