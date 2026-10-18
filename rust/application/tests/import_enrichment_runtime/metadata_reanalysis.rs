use super::*;
use yamantaka_application::metadata_review::{MetadataChange, MetadataRequest};
use yamantaka_domain::metadata_review::{ManualMetadata, MetadataMutation, MetadataReview};

fn read(runtime: &Runtime, id: &str) -> MetadataReview {
    runtime
        .submit(Action::Metadata(Box::new(MetadataRequest {
            document: id.into(),
            change: MetadataChange::Read,
            cancel: Cancellation::default(),
        })))
        .unwrap();
    let Update::Metadata(reply) = wait(runtime) else {
        panic!("Expected metadata");
    };
    reply.result.unwrap()
}
fn imported(runtime: &Runtime, path: PathBuf) -> yamantaka_domain::Document {
    runtime
        .submit(Action::Import(vec![path], Cancellation::default()))
        .unwrap();
    assert!(matches!(wait(runtime), Update::Imported(1, _)));
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    let Update::Library(mut documents, 1, _) = wait(runtime) else {
        panic!("Expected imported document");
    };
    documents.remove(0)
}

#[test]
fn evidence_preview_rejects_a_changed_physical_pdf_even_when_the_catalogue_is_unchanged() {
    use yamantaka_application::cover_source::CoverSource;
    use yamantaka_domain::metadata_review::MetadataField;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("source.pdf");
    let original = fs::read(source("front-matter-evidence-later")).unwrap();
    fs::write(&path, &original).unwrap();
    let worker = worker();
    let governor = worker.memory_governor();
    let runtime = Runtime::start(Database::in_memory_at(11).unwrap(), worker, || {}).unwrap();
    let document = imported(&runtime, path.clone());
    let review = read(&runtime, &document.id);
    assert_eq!(review.title_evidence.as_ref().unwrap().page, 1);
    let reference = CoverSource::metadata_evidence(&document, &review, MetadataField::Title);
    for generation in [1, 2] {
        if generation == 2 {
            let mut changed = original.clone();
            changed.extend_from_slice(b"\n% Synthetic source replacement\n");
            fs::write(&path, changed).unwrap();
        }
        runtime
            .submit(Action::PreviewCover(
                reference.clone(),
                1,
                generation,
                Cancellation::default(),
            ))
            .unwrap();
        let Update::Cover(id, result, _) = wait(&runtime) else {
            panic!("Expected preview result")
        };
        assert_eq!(id, document.id);
        if generation == 1 {
            let image = result.unwrap();
            assert_eq!(image.key.page, 1);
            assert_eq!(image.key.generation, 1);
        } else {
            assert!(matches!(result, Err(Failure::Invalid(_))));
        }
    }
    assert_eq!(read(&runtime, &document.id), review);
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(
        fs::read(source("front-matter-evidence-later")).unwrap(),
        original
    );
}

#[test]
fn real_actor_reanalyzes_while_the_original_reader_stays_open_and_manual_values_win() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("catalogue.sqlite3");
    let worker = worker();
    let governor = worker.memory_governor();
    let runtime = Runtime::start(Database::open(&database).unwrap(), worker, || {}).unwrap();
    let path = source("front-matter-missing");
    let original = fs::read(&path).unwrap();
    let document = imported(&runtime, path.clone());
    let snapshot = read(&runtime, &document.id);
    runtime
        .submit(Action::Metadata(Box::new(MetadataRequest {
            document: document.id.clone(),
            change: MetadataChange::Apply {
                expected: Box::new(snapshot),
                mutation: MetadataMutation::Save(
                    ManualMetadata::new("My geometry", "My author").unwrap(),
                ),
            },
            cancel: Cancellation::default(),
        })))
        .unwrap();
    let Update::Metadata(reply) = wait(&runtime) else {
        panic!("Expected correction");
    };
    let expected = reply.result.unwrap();
    let reader = Cancellation::default();
    runtime
        .submit(Action::Open(document.clone(), None, reader.clone()))
        .unwrap();
    assert!(matches!(wait(&runtime), Update::Opened(..)));
    runtime
        .submit(Action::ConfirmOpen(document.id.clone(), reader))
        .unwrap();
    assert!(matches!(wait(&runtime), Update::Committed(_, _)));
    let capacity_before = runtime.pdf_capacity_revision();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::ReanalyzeMetadata(
            Box::new(expected.clone()),
            cancel.clone(),
        ))
        .unwrap();
    let Update::Metadata(reply) = wait(&runtime) else {
        panic!("Expected reanalysis");
    };
    assert!(reply.cancel.same_request(&cancel) && reply.writing);
    let refreshed = reply.result.unwrap();
    assert!(runtime.pdf_capacity_revision() > capacity_before);
    assert_ne!(refreshed.revision, expected.revision);
    assert_eq!(refreshed.title, "My geometry");
    assert_eq!(refreshed.author, "My author");
    assert!(refreshed.suggested_title.is_none() && refreshed.suggested_author.is_none());
    runtime
        .submit(Action::Text(
            document.id.clone(),
            0,
            Cancellation::default(),
        ))
        .unwrap();
    assert!(
        matches!(wait(&runtime),Update::Text(id,0,text,_) if id==document.id && text.contains("Geometry of Light"))
    );
    assert_eq!(runtime.import_revision(), 1);
    assert_eq!(fs::read(path).unwrap(), original);
    let db = Connection::open(&database).unwrap();
    let counts: (i64, i64) = db
        .query_row(
            "SELECT (SELECT count(*) FROM documents),(SELECT count(*) FROM document_enrichment)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(counts, (1, 1));
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn bounded_reanalysis_failure_and_pre_cancelled_requests_keep_the_previous_review() {
    let worker = worker();
    let governor = worker.memory_governor();
    let runtime = Runtime::start(Database::in_memory_at(11).unwrap(), worker, || {}).unwrap();
    let document = imported(&runtime, source("front-matter-work-limit"));
    let expected = read(&runtime, &document.id);
    for stopped in [false, true] {
        let cancel = Cancellation::default();
        if stopped {
            cancel.cancel();
        }
        runtime
            .submit(Action::ReanalyzeMetadata(
                Box::new(expected.clone()),
                cancel.clone(),
            ))
            .unwrap();
        let Update::Metadata(reply) = wait(&runtime) else {
            panic!("Expected terminal analysis result");
        };
        assert!(reply.cancel.same_request(&cancel));
        if stopped {
            assert!(matches!(reply.result, Err(Failure::Cancelled)));
        } else {
            assert!(reply.result.is_err());
        }
        assert_eq!(read(&runtime, &document.id), expected);
    }
    assert_eq!(runtime.import_revision(), 1);
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn reanalysis_preserves_accepted_automatic_values_and_proposes_conflicting_front_matter() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("catalogue.sqlite3");
    let runtime = Runtime::start(Database::open(&database).unwrap(), worker(), || {}).unwrap();
    let document = imported(&runtime, source("front-matter-missing"));
    let db = Connection::open(&database).unwrap();
    db.execute("UPDATE documents SET title='Accepted legacy title',author='Accepted legacy author' WHERE id=?",[&document.id]).unwrap();
    let expected = read(&runtime, &document.id);
    runtime
        .submit(Action::ReanalyzeMetadata(
            Box::new(expected),
            Cancellation::default(),
        ))
        .unwrap();
    let Update::Metadata(reply) = wait(&runtime) else {
        panic!("Expected analyzed metadata");
    };
    let result = reply.result.unwrap();
    assert_eq!(result.title, "Accepted legacy title");
    assert_eq!(result.author, "Accepted legacy author");
    assert_eq!(result.suggested_title.as_deref(), Some("Geometry of Light"));
    assert_eq!(result.suggested_author.as_deref(), Some("Ada Lovelace"));
    assert!(result.title_override.is_none() && result.author_override.is_none());
}
