use super::tests::{documents, folder_result};
use crate::runtime::{Action, Runtime, Update};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use yamantaka_domain::{
    Cancellation, Failure, LibraryQuery, LibraryRepository, import_recovery::ImportSource,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::{Database, identity::full_sha256};
fn runtime(root: &Path, database: &Path) -> Runtime {
    Runtime::start(
        Database::open(database).unwrap(),
        PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap(),
        || {},
    )
    .unwrap()
}
fn jobs(runtime: &Runtime) -> Vec<yamantaka_domain::import_recovery::PendingImport> {
    runtime.submit(Action::PendingImports).unwrap();
    loop {
        if let Update::PendingImports(result) = runtime
            .updates
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
        {
            return result.unwrap();
        }
    }
}
pub(super) fn verify(root: &Path, paths: &[PathBuf]) {
    let temp = tempfile::tempdir().unwrap();
    let database = temp.path().join("library.sqlite3");
    let source = ImportSource::Files(paths.to_vec()).absolute().unwrap();
    let first_runtime = runtime(root, &database);
    first_runtime
        .submit(Action::Import(
            vec![paths[0].clone()],
            Cancellation::default(),
        ))
        .unwrap();
    assert_eq!(folder_result(&first_runtime).unwrap(), 1);
    assert!(jobs(&first_runtime).is_empty());
    drop(first_runtime);
    let mut db = Database::open(&database).unwrap();
    let first = db.query(&LibraryQuery::default()).unwrap().remove(0);
    assert!(db.file_snapshot(&first.id).unwrap().digest.is_some());

    let job = db.queue_import(&source).unwrap().unwrap();
    db.checkpoint_import(&job, &paths[0], &first.id).unwrap();
    assert_eq!(db.pending_imports().unwrap()[0].completed, 1);
    assert_eq!(db.schema_version().unwrap(), 11);
    drop(db);
    // Reopen the actual files. Merely opening the runtime must not resume work.
    let resumed = runtime(root, &database);
    let pending = jobs(&resumed);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, job);
    assert_eq!(pending[0].completed, 1);
    resumed
        .submit(Action::Query(
            LibraryQuery::default(),
            Cancellation::default(),
        ))
        .unwrap();
    loop {
        if let Update::Library(rows, count, _) = resumed
            .updates
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
        {
            assert_eq!(count, 1);
            assert_eq!(rows[0].id, first.id);
            break;
        }
    }
    resumed
        .submit(Action::ResumeImport(job, Cancellation::default()))
        .unwrap();
    assert_eq!(folder_result(&resumed).unwrap(), 3);
    assert_eq!(resumed.import_revision(), 2);
    let ids = documents(&resumed);
    assert!(ids.contains(&first.id));
    assert!(jobs(&resumed).is_empty());
    drop(resumed);
    // Simulate interruption after a main-db commit but before the recovery checkpoint.
    let mut db = Database::open(&database).unwrap();
    db.execute_batch(&format!("UPDATE documents SET title_override='Keep my title',is_favorite=1,last_page_index=1 WHERE id='{}'",first.id)).unwrap();
    let gap = db
        .queue_import(
            &ImportSource::Files(vec![paths[0].clone()])
                .absolute()
                .unwrap(),
        )
        .unwrap()
        .unwrap();
    drop(db);
    let replay = runtime(root, &database);
    replay
        .submit(Action::ResumeImport(gap, Cancellation::default()))
        .unwrap();
    assert_eq!(folder_result(&replay).unwrap(), 1);
    assert_eq!(documents(&replay), ids);
    assert!(jobs(&replay).is_empty());
    drop(replay);
    let mut db = Database::open(&database).unwrap();
    let saved = db.get(&first.id).unwrap().unwrap();
    assert_eq!(saved.display_title(), "Keep my title");
    assert!(saved.favorite);
    assert_eq!(saved.last_page_index, 1);
    let discard = db.queue_import(&source).unwrap().unwrap();
    db.checkpoint_import(&discard, &paths[0], &first.id)
        .unwrap();
    drop(db);
    let discard_runtime = runtime(root, &database);
    discard_runtime
        .submit(Action::DiscardImport(discard))
        .unwrap();
    loop {
        if let Update::PendingImports(result) = discard_runtime
            .updates
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
        {
            assert!(result.unwrap().is_empty());
            break;
        }
    }
    assert_eq!(documents(&discard_runtime), ids);
    drop(discard_runtime);
    // A close barrier persists an intent even if native admission never happened.
    let waiting = runtime(root, &database);
    waiting
        .submit(Action::ImportFolder(
            temp.path().join("not-created-yet"),
            Cancellation::default(),
        ))
        .unwrap();
    waiting.flush_import_recovery().unwrap();
    drop(waiting);
    let db = Database::open(&database).unwrap();
    assert_eq!(db.pending_imports().unwrap().len(), 1);
    drop(db);
    // Unknown recovery versions must not modify the sidecar or prevent opening v11.
    let mut sidecar = database.as_os_str().to_owned();
    sidecar.push(".imports.sqlite3");
    let sidecar = PathBuf::from(sidecar);
    {
        let connection = rusqlite::Connection::open(&sidecar).unwrap();
        connection.pragma_update(None, "user_version", 2).unwrap();
    }
    let before = full_sha256(&sidecar, &Cancellation::default()).unwrap();
    let db = Database::open(&database).unwrap();
    assert_eq!(db.schema_version().unwrap(), 11);
    assert!(matches!(db.pending_imports(), Err(Failure::Invalid(_))));
    assert_eq!(
        full_sha256(&sidecar, &Cancellation::default()).unwrap(),
        before
    );
    assert_eq!(db.query(&LibraryQuery::default()).unwrap().len(), 3);
}
