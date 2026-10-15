use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use yamantaka_domain::{
    Cancellation, PdfEngine, RenderedPage, Result,
    cover_cache::{CoverCacheRequest, CoverImageCache},
    memory::MemoryGovernor,
    pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::cover_cache::CoverCache;

struct ObservedCache {
    cache: CoverCache,
    hits: AtomicUsize,
}
impl CoverImageCache for ObservedCache {
    fn identity(&self, path: &Path, page: u32, cancel: &Cancellation) -> Result<[u8; 32]> {
        self.cache.identity(path, page, cancel)
    }
    fn load(
        &self,
        request: CoverCacheRequest<'_>,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Option<Arc<RenderedPage>>> {
        let result = self.cache.load(request, governor, cancel)?;
        if result.is_some() {
            self.hits.fetch_add(1, Ordering::SeqCst);
        }
        Ok(result)
    }
    fn save(&self, identity: [u8; 32], image: &RenderedPage, cancel: &Cancellation) -> Result<()> {
        self.cache.save(identity, image, cancel)
    }
}
fn fixture() -> (PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        root.join("build/rust-fixtures/textual.pdf"),
    )
}

#[test]
fn native_cover_reuses_persistent_pixels_after_worker_restart_and_repairs_corruption() {
    let root = tempfile::tempdir().unwrap();
    let (dll, path) = fixture();
    let original = std::fs::read(&path).unwrap();
    let governor = MemoryGovernor::default();
    let cache = Arc::new(ObservedCache {
        cache: CoverCache::new(root.path().into(), 4096 * 1024),
        hits: AtomicUsize::new(0),
    });
    let engine = PdfWorker::start_with_cover_cache(
        &dll,
        MemoryLimits::default(),
        governor.clone(),
        Some(cache.clone()),
    )
    .unwrap();
    let first = engine
        .try_cover("first".into(), path.clone(), 1, 10, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!((first.width, first.height), (512, 384));
    assert_eq!(cache.hits.load(Ordering::SeqCst), 0);
    let expected = first.pixels().to_vec();
    drop(first);
    drop(engine);
    assert_eq!(governor.usage().total, 0);
    let restarted = PdfWorker::start_with_cover_cache(
        &dll,
        MemoryLimits::default(),
        governor.clone(),
        Some(cache.clone()),
    )
    .unwrap();
    let second = restarted
        .try_cover(
            "second".into(),
            path.clone(),
            1,
            20,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(second.pixels(), expected);
    assert_eq!(
        (
            second.key.document_id.as_str(),
            second.key.page,
            second.key.generation
        ),
        ("second", 1, 20)
    );
    assert_eq!(cache.hits.load(Ordering::SeqCst), 1);
    drop(second);
    drop(restarted);
    let record = std::fs::read_dir(root.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(&record, b"damaged synthetic cache").unwrap();
    let repaired = PdfWorker::start_with_cover_cache(
        &dll,
        MemoryLimits::default(),
        governor.clone(),
        Some(cache.clone()),
    )
    .unwrap();
    let third = repaired
        .try_cover("third".into(), path.clone(), 1, 30, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(third.pixels(), expected);
    assert_eq!(cache.hits.load(Ordering::SeqCst), 1);
    drop(third);
    drop(repaired);
    let verified = cache
        .cache
        .load(
            CoverCacheRequest {
                identity: cache.identity(&path, 1, &Cancellation::default()).unwrap(),
                document: "verified",
                page: 1,
                generation: 40,
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(verified.pixels(), expected);
    drop(verified);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[test]
fn unwritable_cache_does_not_prevent_native_cover_delivery() {
    let root = tempfile::tempdir().unwrap();
    let cache_path = root.path().join("occupied");
    std::fs::write(&cache_path, b"synthetic blocker").unwrap();
    let (dll, path) = fixture();
    let governor = MemoryGovernor::default();
    let engine = PdfWorker::start_with_cover_cache(
        &dll,
        MemoryLimits::default(),
        governor.clone(),
        Some(Arc::new(CoverCache::new(cache_path.clone(), 4096 * 1024))),
    )
    .unwrap();
    let image = engine
        .try_cover("fallback".into(), path, 1, 1, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!((image.width, image.height), (512, 384));
    assert_eq!(std::fs::read(cache_path).unwrap(), b"synthetic blocker");
    drop(image);
    drop(engine);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn cached_cover_remains_available_when_both_native_readers_are_protected() {
    let root = tempfile::tempdir().unwrap();
    let (dll, path) = fixture();
    let original = std::fs::read(&path).unwrap();
    let governor = MemoryGovernor::default();
    let cache = Arc::new(ObservedCache {
        cache: CoverCache::new(root.path().into(), 4096 * 1024),
        hits: AtomicUsize::new(0),
    });
    let engine = PdfWorker::start_with_cover_cache(
        &dll,
        MemoryLimits::default(),
        governor.clone(),
        Some(cache.clone()),
    )
    .unwrap();
    let first = engine
        .try_cover("seed".into(), path.clone(), 1, 1, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    let expected = first.pixels().to_vec();
    drop(first);
    for id in ["left", "right"] {
        engine
            .open(id.into(), path.clone(), None, Cancellation::default())
            .unwrap();
    }
    engine
        .set_visible_documents(&["left".into(), "right".into()])
        .unwrap();
    let cached = engine
        .try_cover(
            "library".into(),
            path.clone(),
            1,
            2,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        (
            cached.key.document_id.as_str(),
            cached.key.page,
            cached.key.generation
        ),
        ("library", 1, 2)
    );
    assert_eq!(cached.pixels(), expected);
    assert_eq!(cache.hits.load(Ordering::SeqCst), 1);
    for id in ["left", "right"] {
        assert!(
            !engine
                .text(id.into(), 0, Cancellation::default())
                .unwrap()
                .is_empty()
        );
    }
    assert!(
        engine
            .try_cover(
                "cache-miss".into(),
                path.clone(),
                0,
                3,
                Cancellation::default()
            )
            .unwrap()
            .wait()
            .is_err()
    );
    drop(cached);
    let record = std::fs::read_dir(root.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(record, b"damaged synthetic cache").unwrap();
    assert!(
        engine
            .try_cover(
                "corrupt-cache".into(),
                path.clone(),
                1,
                4,
                Cancellation::default()
            )
            .unwrap()
            .wait()
            .is_err()
    );
    for id in ["left", "right"] {
        assert!(
            !engine
                .text(id.into(), 0, Cancellation::default())
                .unwrap()
                .is_empty()
        );
    }
    drop(engine);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(path).unwrap(), original);
}
