#[path = "support/import.rs"]
mod verified_import;
use std::{
    fs,
    path::{Path, PathBuf},
};
use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, NewDocument, PdfEngine,
    front_matter::{FrontMatter, ImportAnalysis},
    metadata_review::{ManualMetadata, MetadataField, MetadataMutation},
    pdf_text::TextGeometryPurpose,
    pdf_ticket::AsyncPdfEngine,
    search::IndexAccess,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn engine() -> PdfWorker {
    PdfWorker::start(
        &root().join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap()
}
fn analyze(worker: &PdfWorker, path: &Path) -> NewDocument {
    let path = path.canonicalize().unwrap();
    let cancel = Cancellation::default();
    let info = worker
        .open("analysis".into(), path.clone(), None, cancel.clone())
        .unwrap();
    let source = worker
        .try_source_revision("analysis".into(), cancel.clone())
        .unwrap()
        .wait()
        .unwrap();
    let mut scan = FrontMatter::new(
        path.file_name().unwrap().to_str().unwrap(),
        info.title.as_deref(),
        info.author.as_deref(),
        &worker.memory_governor(),
    )
    .unwrap();
    for page in 0..info.pages.len().min(6) {
        let text = worker
            .try_text_geometry(
                "analysis".into(),
                page as u32,
                TextGeometryPurpose::FrontMatter,
                cancel.clone(),
            )
            .unwrap()
            .wait()
            .unwrap();
        scan.observe(&text, info.pages[page].coordinates(), &cancel)
            .unwrap();
    }
    let analysis = ImportAnalysis::Complete(Box::new(scan.finish(&cancel).unwrap()));
    worker.close("analysis".into()).unwrap();
    NewDocument {
        path,
        info,
        source,
        analysis,
    }
}

#[test]
fn native_page_evidence_opens_the_later_page_without_changing_library_decisions() {
    let worker = engine();
    let governor = worker.memory_governor();
    let path = root().join("build/rust-fixtures/front-matter-evidence-later.pdf");
    let original = fs::read(&path).unwrap();
    let analyzed = analyze(&worker, &path);
    let mut db = Database::in_memory_at(11).unwrap();
    let id = verified_import::save(&mut db, &analyzed).unwrap();
    let cancel = Cancellation::default();
    let review = db.metadata_review(&id, &cancel).unwrap();
    assert_eq!(
        (
            review.title_evidence.as_ref().unwrap().page,
            review.author_evidence.as_ref().unwrap().page
        ),
        (1, 1)
    );
    assert_eq!(
        review.title_evidence.as_ref().unwrap().value,
        "Geometry of Light"
    );
    let before = db.get(&id).unwrap().unwrap();
    for field in [MetadataField::Title, MetadataField::Author] {
        let source = db.metadata_source(&review, field, &cancel).unwrap();
        assert_eq!(source.page, 1);
        assert_eq!(source.document.id, id);
        assert_eq!(source.document.path, review.file.path);
    }
    let after = db.metadata_review(&id, &cancel).unwrap();
    assert_eq!(after, review);
    let document = db.get(&id).unwrap().unwrap();
    assert_eq!(
        (document.title, document.author, document.last_page_index),
        (before.title, before.author, before.last_page_index)
    );
    let manual = db
        .change_metadata(
            &review,
            &MetadataMutation::Save(ManualMetadata::new("My title", "My author").unwrap()),
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert!(manual.title_evidence.is_none() && manual.author_evidence.is_none());
    assert!(matches!(
        db.metadata_source(&review, MetadataField::Title, &cancel),
        Err(Failure::Invalid(_))
    ));
    assert!(matches!(
        db.metadata_source(&manual, MetadataField::Title, &cancel),
        Err(Failure::Invalid(_))
    ));
    drop(analyzed);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(fs::read(path).unwrap(), original);
}

#[test]
fn stale_catalogue_frames_cancelled_requests_and_changed_files_cannot_open_evidence() {
    let worker = engine();
    let governor = worker.memory_governor();
    let fixture = root().join("build/rust-fixtures/front-matter-evidence-later.pdf");
    let original = fs::read(&fixture).unwrap();
    for change in [
        "UPDATE documents SET path='different.pdf'",
        "UPDATE documents SET fingerprint='different'",
        "UPDATE documents SET page_count=1",
        "UPDATE documents SET file_modified_at='later'",
        "UPDATE documents SET is_missing=1",
        "UPDATE document_enrichment SET suggested_title='Different title'",
    ] {
        let analyzed = analyze(&worker, &fixture);
        let db = Database::in_memory_at(11).unwrap();
        let mut db = db;
        let id = verified_import::save(&mut db, &analyzed).unwrap();
        let review = db.metadata_review(&id, &Cancellation::default()).unwrap();
        db.execute_batch(change).unwrap();
        assert!(
            matches!(
                db.metadata_source(&review, MetadataField::Title, &Cancellation::default()),
                Err(Failure::Invalid(_))
            ),
            "{change}"
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("changed.pdf");
    fs::write(&copy, &original).unwrap();
    let analyzed = analyze(&worker, &copy);
    let mut db = Database::in_memory_at(11).unwrap();
    let id = verified_import::save(&mut db, &analyzed).unwrap();
    let review = db.metadata_review(&id, &Cancellation::default()).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.metadata_source(&review, MetadataField::Title, &cancel),
        Err(Failure::Cancelled)
    ));
    let mut changed = original.clone();
    changed.extend_from_slice(b"\n% Synthetic changed source\n");
    fs::write(&copy, &changed).unwrap();
    assert!(matches!(
        db.metadata_source(&review, MetadataField::Title, &Cancellation::default()),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(
        db.metadata_review(&id, &Cancellation::default()).unwrap(),
        review
    );
    drop(analyzed);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(fs::read(fixture).unwrap(), original);
    assert_eq!(fs::read(copy).unwrap(), changed);
}
