use super::*;
use std::{
    path::Path,
    time::{Duration, Instant},
};
use yamantaka_domain::{LibraryQuery, PdfEngine};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;
fn manual(db: &mut Database, id: &str, page: u32, stamp: &str) {
    db.execute_batch(&format!("INSERT INTO document_covers(document_id,source_page_index,small_thumbnail_path,large_thumbnail_path,generated_at,source_modified_at,confidence,selection_mode,algorithm_version,crop_mode,page_aspect_ratio) VALUES('{id}',{page},'','','{stamp}','2026-10-20',1,'manual',1,'contain',0.75) ON CONFLICT(document_id) DO UPDATE SET source_page_index={page},generated_at='{stamp}',selection_mode='manual'")).unwrap();
}
fn request(ids: &[&str]) -> RegenerationRequest {
    RegenerationRequest {
        ids: ids.iter().map(|id| (*id).into()).collect(),
        cancel: Cancellation::default(),
    }
}
fn advance_until(
    job: &mut RegenerationJob,
    db: &mut Database,
    engine: &PdfWorker,
    capacity: u64,
    done: impl Fn(&RegenerationSnapshot) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            Instant::now() < deadline,
            "Native cover regeneration did not reach the expected state"
        );
        if done(&job.snapshot(false)) {
            return;
        }
        if job.ready(capacity) {
            let finished = job.advance(db, engine, capacity);
            if done(&job.snapshot(finished)) {
                return;
            }
            assert!(!finished, "Cover batch finished before the expected state");
        } else {
            let mut select = Select::new();
            job.register(&mut select);
            select.ready_timeout(Duration::from_secs(5)).unwrap();
        }
    }
}
pub(crate) fn verify() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join("build/rust-fixtures/textual.pdf");
    let original = std::fs::read(&source).unwrap();
    let path = source.to_str().unwrap().replace('\'', "''");
    let mut db = Database::in_memory_at(11).unwrap();
    for id in ["a", "b", "c"] {
        db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,title_override,reading_progress,last_page_index) VALUES('{id}','{path}','{id}','fixture.pdf',2,1,'2026-10-20','2026-10-20','2026-10-20','synthetic','My saved title',0.5,1)")).unwrap();
    }
    manual(&mut db, "a", 1, "before");
    manual(&mut db, "b", 0, "before");
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    worker
        .open(
            "reader-a".into(),
            root.join("build/rust-fixtures/automatic-cover.pdf"),
            None,
            Cancellation::default(),
        )
        .unwrap();
    worker
        .open(
            "reader-b".into(),
            root.join("build/rust-fixtures/front-matter-logo.pdf"),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let protected = vec!["reader-a".to_owned(), "reader-b".to_owned()];
    worker.set_visible_documents(&protected).unwrap();
    let mut job = RegenerationJob::start(request(&["b", "c", "missing"]), &db).unwrap();
    advance_until(&mut job, &mut db, &worker, 0, |state| {
        state.waiting_for_capacity
    });
    assert!(!job.ready(0));
    for id in ["reader-a", "reader-b"] {
        worker
            .try_text(id.into(), 0, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap();
    }

    assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 3);
    manual(&mut db, "b", 1, "newer while waiting");
    let newer = db.cover_snapshot("b").unwrap();
    worker.set_visible_documents(&[]).unwrap();
    advance_until(&mut job, &mut db, &worker, 1, |state| state.finished);
    let report = job.reply().result.unwrap();
    assert_eq!(report.succeeded, ["c"]);
    assert_eq!(report.failed, ["missing", "b"]);
    assert!(report.remaining.is_empty());
    assert_eq!(db.cover_snapshot("b").unwrap(), newer);
    assert_eq!(db.cover_source_page("c").unwrap(), Some(0));
    drop(job);
    let mut late = RegenerationJob::start(request(&["a"]), &db).unwrap();
    assert!(!late.advance(&mut db, &worker, 1));
    manual(&mut db, "a", 0, "newer during detection");
    let newest = db.cover_snapshot("a").unwrap();
    advance_until(&mut late, &mut db, &worker, 1, |state| state.finished);
    assert_eq!(late.reply().result.unwrap().failed, ["a"]);
    assert_eq!(db.cover_snapshot("a").unwrap(), newest);
    drop(late);
    let input = request(&["a", "b", "a"]);
    let cancel = input.cancel.clone();
    let mut partial = RegenerationJob::start(input, &db).unwrap();
    advance_until(&mut partial, &mut db, &worker, 1, |state| {
        state.report.succeeded.len() == 1
    });
    cancel.cancel();
    assert!(partial.advance(&mut db, &worker, 1));
    let report = partial.reply().result.unwrap();
    assert_eq!(report.succeeded, ["a"]);
    assert_eq!(report.remaining, ["b"]);
    assert!(report.failed.is_empty());
    assert_eq!(db.cover_snapshot("b").unwrap(), newer);
    drop(partial);
    let input = request(&["a", "b"]);
    let cancel = input.cancel.clone();
    let mut stopped = RegenerationJob::start(input, &db).unwrap();
    cancel.cancel();
    assert!(stopped.advance(&mut db, &worker, 1));
    assert_eq!(stopped.reply().result.unwrap().remaining, ["a", "b"]);
    drop(stopped);
    assert_eq!(
        db.get("a").unwrap().unwrap().display_title(),
        "My saved title"
    );
    assert_eq!(db.get("a").unwrap().unwrap().reading_progress, 0.5);
    // Exercise actor admission, readiness, scoped completion and SQL while parked.
    worker
        .open(
            "reader-a".into(),
            root.join("build/rust-fixtures/automatic-cover.pdf"),
            None,
            Cancellation::default(),
        )
        .unwrap();
    worker
        .open(
            "reader-b".into(),
            root.join("build/rust-fixtures/front-matter-logo.pdf"),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let runtime = crate::runtime::Runtime::start(db, worker.clone(), || {}).unwrap();
    runtime
        .submit(crate::runtime::Action::VisibleDocuments(protected))
        .unwrap();
    let input = request(&["a", "c"]);
    let scope = input.cancel.clone();
    runtime
        .submit(crate::runtime::Action::RegenerateCovers(Box::new(input)))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !runtime
        .cover_regeneration_progress()
        .is_some_and(|state| state.cancel.same_request(&scope) && state.waiting_for_capacity)
    {
        assert!(Instant::now() < deadline);
        let _ = runtime.updates.recv_timeout(Duration::from_millis(50));
    }
    let query = Cancellation::default();
    runtime
        .submit(crate::runtime::Action::Query(
            LibraryQuery::default(),
            query.clone(),
        ))
        .unwrap();
    loop {
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
        {
            crate::runtime::Update::Library(_, count, cancel) if cancel.same_request(&query) => {
                assert_eq!(count, 3);
                break;
            }
            _ => {}
        }
    }
    scope.cancel();
    runtime.wake();
    loop {
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
        {
            crate::runtime::Update::SelectionOrganized(reply)
                if reply.cancel.same_request(&scope) =>
            {
                let report = reply.result.unwrap();
                assert!(report.succeeded.is_empty());
                assert_eq!(report.remaining, ["a", "c"]);
                break;
            }
            _ => {}
        }
    }
    assert!(runtime.cover_regeneration_progress().unwrap().finished);
    drop(runtime);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(source).unwrap(), original);
}
