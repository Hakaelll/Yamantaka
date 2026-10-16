use std::{
    fs,
    path::{Path, PathBuf},
};
use yamantaka_domain::{
    Cancellation, LibraryRepository, NewDocument, PdfEngine,
    front_matter::{FrontMatter, ImportAnalysis},
    metadata_review::{ManualMetadata, MetadataMutation},
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
fn reanalysis_replaces_evidence_and_preserves_manual_metadata_topics_and_reading_data() {
    let worker = engine();
    let governor = worker.memory_governor();
    let source = root().join("build/rust-fixtures/front-matter-missing.pdf");
    let bytes = fs::read(&source).unwrap();
    let analyzed = analyze(&worker, &source);
    let cancel = Cancellation::default();
    let mut db = Database::in_memory_at(11).unwrap();
    let id = db.import(&analyzed).unwrap();
    let original = db.get(&id).unwrap().unwrap();
    let before = db.metadata_review(&id, &cancel).unwrap();
    let refreshed = db
        .reanalyze_metadata(
            &before,
            &original,
            &analyzed,
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        refreshed.suggested_title.as_deref(),
        Some("Geometry of Light")
    );
    assert_ne!(before.revision, refreshed.revision);
    let corrected = db
        .change_metadata(
            &refreshed,
            &MetadataMutation::Save(
                ManualMetadata::new("Manual geometry", "Manual author").unwrap(),
            ),
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    db.execute_batch(&format!("UPDATE documents SET reading_progress=0.5,is_favorite=1 WHERE id='{id}'; UPDATE document_enrichment SET topics_manual=1,title_confidence=0.9 WHERE document_id='{id}';")).unwrap();
    let result = db
        .reanalyze_metadata(
            &corrected,
            &original,
            &analyzed,
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert_eq!(result.title, "Manual geometry");
    assert_eq!(result.author, "Manual author");
    assert!(result.suggested_title.is_none() && result.suggested_author.is_none());
    let document = db.get(&id).unwrap().unwrap();
    assert_eq!(document.reading_progress, 0.5);
    assert!(document.favorite);
    db.execute_batch(&format!("CREATE TEMP TABLE verify_reanalysis_guard(v INTEGER CHECK(v=1)); INSERT INTO verify_reanalysis_guard SELECT count(*) FROM document_enrichment WHERE document_id='{id}' AND topics_manual=1 AND title_confidence IS NULL AND status='completed';")).unwrap();
    assert_eq!(fs::read(source).unwrap(), bytes);
    drop(analyzed);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn old_metadata_or_changed_native_source_cannot_commit_reanalysis() {
    let worker = engine();
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("study.pdf");
    fs::copy(
        root().join("build/rust-fixtures/front-matter-missing.pdf"),
        &source,
    )
    .unwrap();
    let analyzed = analyze(&worker, &source);
    let cancel = Cancellation::default();
    let mut db = Database::in_memory_at(11).unwrap();
    let id = db.import(&analyzed).unwrap();
    let original = db.get(&id).unwrap().unwrap();
    db.execute_batch(&format!(
        "UPDATE documents SET title='Accepted automatic title' WHERE id='{id}'"
    ))
    .unwrap();
    let old = db.metadata_review(&id, &cancel).unwrap();
    assert!(
        db.reanalyze_metadata(&old, &original, &analyzed, IndexAccess::Disabled, &cancel)
            .is_err()
    );
    assert_eq!(db.metadata_review(&id, &cancel).unwrap(), old);
    let fresh = db
        .change_metadata(
            &old,
            &MetadataMutation::Save(ManualMetadata::new("New correction", "New author").unwrap()),
            IndexAccess::Disabled,
            &cancel,
        )
        .unwrap();
    assert!(
        db.reanalyze_metadata(&old, &original, &analyzed, IndexAccess::Disabled, &cancel)
            .is_err()
    );
    let modified = fs::metadata(&source).unwrap().modified().unwrap();
    let mut bytes = fs::read(&source).unwrap();
    bytes[7] = if bytes[7] == b'7' { b'6' } else { b'7' };
    fs::write(&source, &bytes).unwrap();
    fs::File::options()
        .write(true)
        .open(&source)
        .unwrap()
        .set_modified(modified)
        .unwrap();
    assert!(
        db.reanalyze_metadata(&fresh, &original, &analyzed, IndexAccess::Disabled, &cancel)
            .is_err()
    );
    assert_eq!(db.metadata_review(&id, &cancel).unwrap(), fresh);
    assert_eq!(fs::read(&source).unwrap(), bytes);
}

#[test]
fn ignored_or_failed_reanalysis_writes_restore_the_entire_previous_review() {
    let worker = engine();
    let analyzed = analyze(
        &worker,
        &root().join("build/rust-fixtures/front-matter-missing.pdf"),
    );
    let cancel = Cancellation::default();
    for trigger in [
        "CREATE TRIGGER broken BEFORE UPDATE ON document_enrichment BEGIN SELECT RAISE(IGNORE); END;",
        "DROP TABLE library_search;",
        "CREATE TRIGGER broken AFTER UPDATE ON app_metadata BEGIN UPDATE documents SET path='wrong.pdf'; END;",
        "CREATE TRIGGER broken AFTER UPDATE ON app_metadata BEGIN UPDATE documents SET source_title='Wrong source'; END;",
        "CREATE TRIGGER broken BEFORE UPDATE ON app_metadata BEGIN SELECT RAISE(ABORT,'Synthetic reanalysis failure'); END;",
    ] {
        let mut db = Database::in_memory_at(11).unwrap();
        let id = db.import(&analyzed).unwrap();
        let original = db.get(&id).unwrap().unwrap();
        let before = db.metadata_review(&id, &cancel).unwrap();
        db.execute_batch(trigger).unwrap();
        assert!(
            db.reanalyze_metadata(&before, &original, &analyzed, IndexAccess::Enabled, &cancel)
                .is_err()
        );
        assert_eq!(db.metadata_review(&id, &cancel).unwrap(), before);
    }
}
