use super::*;
use yamantaka_domain::{
    PdfEngine, cover_cache::CoverCacheOperation, memory::GlobalMemoryLimits,
    pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

#[test]
fn cleanup_preserves_sources_and_unknown_files_without_blocking_native_covers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("book.pdf");
    fs::copy(root.join("build/rust-fixtures/textual.pdf"), &source).unwrap();
    let original = Sha256::digest(fs::read(&source).unwrap());
    let cancel = Cancellation::default();
    let cache = Arc::new(CoverCache::new(
        temp.path().join("covers"),
        1024 * 1024 * 16,
    ));
    verify_preserved_timestamp_identity(&cache, temp.path());
    let absent = cache
        .maintain(CoverCacheOperation::Inspect, &cancel)
        .unwrap();
    assert_eq!(absent.inventory.unwrap().bytes, 0);
    assert!(!cache.root.exists());
    let engine = PdfWorker::start_with_cover_cache(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
        MemoryGovernor::new(GlobalMemoryLimits::default()),
        Some(cache.clone()),
    )
    .unwrap();
    engine
        .open("reader".into(), source.clone(), None, cancel.clone())
        .unwrap();
    engine.set_visible_documents(&["reader".into()]).unwrap();
    let image = engine
        .try_cover("cover".into(), source.clone(), 1, 1, cancel.clone())
        .unwrap()
        .wait()
        .unwrap();
    let expected_bytes = (HEADER + image.pixels().len()) as u64;
    fs::write(cache.root.join("notes.txt"), b"keep").unwrap();
    fs::create_dir(cache.root.join("nested")).unwrap();
    fs::write(cache.root.join("nested/book.pdf"), b"keep nested file").unwrap();
    fs::write(
        cache.root.join(format!("{}.rgba", "b".repeat(64))),
        b"future version",
    )
    .unwrap();
    let unicode = format!("{}é.rgba", "a".repeat(62));
    assert_eq!(unicode.len(), 69);
    fs::write(cache.root.join(&unicode), b"keep unicode name").unwrap();
    let report = cache
        .maintain(CoverCacheOperation::Inspect, &cancel)
        .unwrap();
    let inventory = report.inventory.unwrap();
    assert_eq!(
        (inventory.files, inventory.bytes, inventory.preserved),
        (1, expected_bytes, 4)
    );
    assert_eq!(report.failed_files, 0);

    let guard = cache.writer.lock().unwrap();
    cache
        .checked_quota
        .store(false, std::sync::atomic::Ordering::Release);
    let ticket = engine
        .try_cover("busy-cache".into(), source.clone(), 1, 2, cancel.clone())
        .unwrap();
    let rendered = ticket
        .readiness()
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("Native PDF work waited for the cache writer")
        .unwrap();
    assert_eq!(rendered.pixels(), image.pixels());
    let waiting_cancel = Cancellation::default();
    let waiting_scope = waiting_cancel.clone();
    let waiting_cache = cache.clone();
    let waiting = std::thread::spawn(move || {
        waiting_cache.maintain(CoverCacheOperation::Clear, &waiting_scope)
    });
    waiting_cancel.cancel();
    let cancelled = waiting.join().unwrap().unwrap();
    assert!(cancelled.cancelled);
    assert!(cancelled.inventory.is_none());
    assert_eq!(cancelled.removed_files, 0);
    drop(guard);

    let report = cache.maintain(CoverCacheOperation::Clear, &cancel).unwrap();
    assert_eq!(
        (
            report.removed_files,
            report.removed_bytes,
            report.failed_files
        ),
        (1, expected_bytes, 0)
    );
    assert_eq!(
        (
            report.inventory.unwrap().files,
            report.inventory.unwrap().preserved
        ),
        (0, 4)
    );
    assert_eq!(Sha256::digest(fs::read(&source).unwrap()), original);
    assert_eq!(fs::read(cache.root.join("notes.txt")).unwrap(), b"keep");
    assert_eq!(
        fs::read(cache.root.join("nested/book.pdf")).unwrap(),
        b"keep nested file"
    );
    assert_eq!(
        fs::read(cache.root.join(&unicode)).unwrap(),
        b"keep unicode name"
    );
    assert!(
        engine
            .text("reader".into(), 0, cancel.clone())
            .unwrap()
            .contains("Yamantaka Rust")
    );
    let regenerated = engine
        .try_cover("regenerated".into(), source.clone(), 1, 3, cancel.clone())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(regenerated.key.page, 1);
    assert_eq!(regenerated.pixels(), image.pixels());
    assert_eq!(
        cache
            .maintain(CoverCacheOperation::Inspect, &cancel)
            .unwrap()
            .inventory
            .unwrap()
            .files,
        1
    );
}

fn verify_preserved_timestamp_identity(cache: &CoverCache, directory: &Path) {
    use std::{
        fs::FileTimes,
        io::{Seek, SeekFrom},
    };
    let path = directory.join("same-date.pdf");
    fs::write(&path, vec![b'a'; 65_540]).unwrap();
    let modified = path.metadata().unwrap().modified().unwrap();
    let cancel = Cancellation::default();
    let before = cache.identity(&path, 0, &cancel).unwrap();
    {
        let mut file = fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.seek(SeekFrom::Start(65_536)).unwrap();
        file.write_all(b"edit").unwrap();
        file.set_times(FileTimes::new().set_modified(modified))
            .unwrap();
    }
    assert_eq!(path.metadata().unwrap().modified().unwrap(), modified);
    assert_eq!(path.metadata().unwrap().len(), 65_540);
    assert_ne!(
        before,
        cache.identity(&path, 0, &cancel).unwrap(),
        "A changed PDF reused its previous pixel-cache identity"
    );
}
