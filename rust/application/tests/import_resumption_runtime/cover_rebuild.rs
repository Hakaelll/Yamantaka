use super::*;
use std::path::Path;
use yamantaka_application::{background_password::Purpose, cover_rebuild::Request};
use yamantaka_domain::{
    LibraryQuery,
    cover_cache::{CoverCacheOperation, CoverImageCache},
    memory::MemoryGovernor,
};
fn preserved(path: &Path, id: &str) -> (String, String) {
    let db =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    (db.query_row("SELECT json_array(title_override,author_override,last_opened_at,file_modified_at) FROM documents WHERE id=?",[id],|r|r.get(0)).unwrap(),
     db.query_row("SELECT json_array(type,color,note,selected_text,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at) FROM annotations WHERE id='cover-sentinel'",[],|r|r.get(0)).unwrap())
}
pub(super) fn verify(
    path: &Path,
    source: &Path,
    root: &Path,
    cache: Arc<yamantaka_storage::cover_cache::CoverCache>,
) {
    let db = Database::open(path).unwrap();
    let document = db
        .query(&LibraryQuery::default())
        .unwrap()
        .into_iter()
        .find(|d| d.path == source)
        .unwrap();
    db.execute_batch(&format!("UPDATE document_covers SET source_page_index=1,selection_mode='manual' WHERE document_id='{}'; UPDATE documents SET title_override='Preserved manual title',author_override='Preserved manual author' WHERE id='{}'; INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) VALUES('cover-sentinel','{}',0,'pageComment','violet','Preserved annotation','{}','2026-11-01','2026-11-01');",document.id,document.id,document.id,document.fingerprint)).unwrap();
    drop(db);
    let data = preserved(path, &document.id);
    for outcome in ["success", "cancel", "changed", "slots"] {
        cache
            .maintain(CoverCacheOperation::Clear, &Cancellation::default())
            .unwrap();
        let db = Database::open(path).unwrap();
        let document = db.get(&document.id).unwrap().unwrap();
        let mut revision = db.cover_revision(&document.id).unwrap();
        let governor = MemoryGovernor::new(Default::default());
        let engine = PdfWorker::start_with_cover_cache(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
            governor.clone(),
            Some(cache.clone()),
        )
        .unwrap();
        let control = engine.clone();
        if outcome == "slots" {
            control
                .set_visible_documents(&["pinned-a".into(), "pinned-b".into()])
                .unwrap();
            for id in ["pinned-a", "pinned-b"] {
                control
                    .open(
                        id.into(),
                        root.join("build/rust-fixtures/textual.pdf"),
                        None,
                        Cancellation::default(),
                    )
                    .unwrap();
            }
        }
        let runtime = Runtime::start(db, engine, || {}).unwrap();
        if outcome == "slots" {
            runtime
                .submit(Action::VisibleDocuments(vec![
                    "pinned-a".into(),
                    "pinned-b".into(),
                ]))
                .unwrap();
        }
        let cancel = Cancellation::default();
        runtime
            .submit(Action::RebuildCover(Box::new(Request {
                document: document.clone(),
                automatic: None,
                governor,
                cancel: cancel.clone(),
            })))
            .unwrap();
        if outcome != "slots" {
            let prompt = password::challenge(&runtime);
            assert!(prompt.purpose() == Purpose::Cover);
            assert!(!prompt.rejected());
            if outcome == "cancel" {
                drop(prompt);
            } else {
                if outcome == "changed" {
                    let changed = Database::open(path).unwrap();
                    changed
                        .execute_batch(&format!(
                            "UPDATE document_covers SET source_page_index=0 WHERE document_id='{}'",
                            document.id
                        ))
                        .unwrap();
                    revision = changed.cover_revision(&document.id).unwrap();
                }
                prompt
                    .answer("incorrect synthetic credential".into())
                    .unwrap();
                let retry = password::challenge(&runtime);
                assert!(retry.rejected());
                retry.answer("fixture-password".into()).unwrap();
            }
        }
        loop {
            if let Update::CoverRebuilt(reply) = runtime
                .updates
                .recv_timeout(Duration::from_secs(10))
                .unwrap()
            {
                assert_eq!(reply.document, document.id);
                assert!(reply.cancel.same_request(&cancel));
                match outcome {
                    "success" => reply.result.unwrap(),
                    "slots" => assert!(matches!(reply.result, Err(Failure::NativeSlotsBusy))),
                    "cancel" => assert!(matches!(reply.result, Err(Failure::Cancelled))),
                    "changed" => assert!(matches!(reply.result, Err(Failure::Invalid(_)))),
                    _ => unreachable!(),
                }
                break;
            }
        }
        if outcome == "success" {
            let image = control
                .try_cover(
                    document.id.clone(),
                    source.into(),
                    1,
                    91,
                    Cancellation::default(),
                )
                .unwrap()
                .wait()
                .unwrap();
            let expected = control
                .try_cover(
                    "chosen-page-reference".into(),
                    root.join("build/rust-fixtures/textual.pdf"),
                    1,
                    91,
                    Cancellation::default(),
                )
                .unwrap()
                .wait()
                .unwrap();
            assert_eq!(
                [image.width, image.height],
                [expected.width, expected.height]
            );
            assert_eq!(image.pixels(), expected.pixels());
            assert_eq!(image.key.page, 1);
        }
        if outcome == "slots" {
            for id in ["pinned-a", "pinned-b"] {
                assert!(
                    control
                        .text(id.into(), 0, Cancellation::default())
                        .unwrap()
                        .contains("Yamantaka")
                );
            }
        }
        drop(runtime);
        drop(control);
        let db = Database::open_read_only(path).unwrap();
        assert_eq!(db.cover_revision(&document.id).unwrap(), revision);
        assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 2);
        assert_eq!(
            preserved(path, &document.id),
            data,
            "cover rebuild modified catalogue metadata or an annotation"
        );
    }
}
