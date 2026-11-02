use super::*;
use std::{fs, path::Path, time::Duration};
use yamantaka_domain::{
    LibraryRepository, NewDocument, PdfEngine, front_matter::ImportAnalysis,
    pdf_ticket::AsyncPdfEngine,
};
use yamantaka_storage::{Database, identity::full_sha256};
fn hash(path: &Path, governor: &MemoryGovernor) -> HashedFile {
    let mut job =
        ImportHashJob::start(path.into(), governor.clone(), Cancellation::default()).unwrap();
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(5)).unwrap();
    drop(select);
    let proof = job.finish().unwrap();
    drop(job);
    proof
}
pub(crate) fn verify(root: &Path) {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("Imported study 日本語.pdf");
    fs::copy(root.join("build/rust-fixtures/textual.pdf"), &path).unwrap();
    let path = path.canonicalize().unwrap();
    let original = fs::read(&path).unwrap();
    let worker = yamantaka_pdf::PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        Default::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    let proof = hash(&path, &governor);
    assert_eq!(
        fs::write(&path, b"attempted replacement")
            .unwrap_err()
            .raw_os_error(),
        Some(32)
    );
    let info = worker
        .open(
            "hash-proof".into(),
            path.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let source = worker
        .try_source_revision("hash-proof".into(), Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    worker.close("hash-proof".into()).unwrap();
    let reference = proof.reference(&source).unwrap();
    let mut different = source.clone();
    different.size += 1;
    assert!(matches!(
        proof.reference(&different),
        Err(Failure::Invalid(_))
    ));
    let document = NewDocument {
        path: path.clone(),
        info,
        source,
        analysis: ImportAnalysis::NotRequested,
    };
    let database = temp.path().join("import.sqlite3");
    let mut db = Database::open(&database).unwrap();
    let cancel = Cancellation::default();
    let mut bad_reference = reference.clone();
    bad_reference.source.size += 1;
    assert!(matches!(
        db.import(&document, &bad_reference, &cancel, None),
        Err(Failure::Invalid(_))
    ));
    // This deferred failure occurs at main commit, after the auxiliary digest write.
    db.execute_batch("CREATE TABLE commit_fault(document_id TEXT REFERENCES documents(id) DEFERRABLE INITIALLY DEFERRED); CREATE TRIGGER reject_commit AFTER INSERT ON documents BEGIN INSERT INTO commit_fault VALUES('absent-document'); END;").unwrap();
    assert!(matches!(
        db.import(&document, &reference, &cancel, None),
        Err(Failure::Storage(_))
    ));
    assert_eq!(db.count(&Default::default()).unwrap(), 0);
    let mut cache_path = database.clone().into_os_string();
    cache_path.push(".file-hashes.sqlite3");
    let cache = rusqlite::Connection::open(PathBuf::from(cache_path)).unwrap();
    assert_eq!(
        cache
            .query_row("SELECT count(*) FROM file_hashes", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    drop(cache);
    db.execute_batch("DROP TRIGGER reject_commit; DROP TABLE commit_fault;")
        .unwrap();
    let job = db
        .queue_import(
            &yamantaka_domain::import_recovery::ImportSource::Files(vec![path.clone()])
                .absolute()
                .unwrap(),
        )
        .unwrap()
        .unwrap();
    let id = db
        .import(&document, &reference, &cancel, Some(&job))
        .unwrap();
    let committed = db.import_checkpoint(&job, &path).unwrap().unwrap();
    assert_eq!(committed.document, id);
    assert_eq!(
        committed.origin,
        yamantaka_domain::import_recovery::CheckpointOrigin::Imported
    );
    assert!(
        db.import_checkpoint("another-job", &path)
            .unwrap()
            .is_none()
    );
    db.remove_import(&job).unwrap();
    assert_eq!(
        db.file_snapshot(&id).unwrap().digest,
        Some(reference.digest)
    );
    db.execute_batch(&format!("UPDATE documents SET title_override='My retained title',is_favorite=1,last_page_index=1 WHERE id='{id}'")).unwrap();
    assert_eq!(db.import(&document, &reference, &cancel, None).unwrap(), id);
    assert_eq!(db.registered_import_path(&path).unwrap(), Some(id.clone()));
    let plain = path
        .to_string_lossy()
        .strip_prefix(r"\\?\")
        .unwrap()
        .to_owned();
    assert_eq!(
        db.registered_import_path(Path::new(&plain)).unwrap(),
        Some(id.clone())
    );
    use std::os::windows::ffi::OsStringExt;
    let invalid_path = temp.path().join(std::ffi::OsString::from_wide(&[0xd800]));
    assert!(matches!(
        db.registered_import_path(&invalid_path),
        Err(Failure::Invalid(_))
    ));
    let saved = db.get(&id).unwrap().unwrap();
    assert_eq!(saved.display_title(), "My retained title");
    assert!(saved.favorite);
    assert_eq!(saved.last_page_index, 1);
    let reopened = Database::open(&database).unwrap();
    assert_eq!(
        reopened.file_snapshot(&id).unwrap().digest,
        Some(reference.digest)
    );
    drop(reopened);
    let actual = full_sha256(&path, &cancel).unwrap();
    assert_eq!(
        reference
            .digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        actual
    );
    // Refuse an unknown auxiliary version without committing a new document.
    let future = temp.path().join("future.sqlite3");
    let mut future_cache = future.clone().into_os_string();
    future_cache.push(".file-hashes.sqlite3");
    let future_cache = PathBuf::from(future_cache);
    let sql = rusqlite::Connection::open(&future_cache).unwrap();
    sql.pragma_update(None, "user_version", 99).unwrap();
    drop(sql);
    let before = full_sha256(&future_cache, &cancel).unwrap();
    let mut future = Database::open(&future).unwrap();
    assert!(matches!(
        future.import(&document, &reference, &cancel, None),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(future.count(&Default::default()).unwrap(), 0);
    assert_eq!(full_sha256(&future_cache, &cancel).unwrap(), before);
    assert_eq!(fs::read(&path).unwrap(), original);
    drop(proof);
    drop(document);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
    // The guard is temporary; ordinary file operations work after ownership ends.
    assert!(fs::OpenOptions::new().write(true).open(&path).is_ok());
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        ImportHashJob::start(path, governor.clone(), cancelled),
        Err(Failure::Cancelled)
    ));
}
