use std::{fs, sync::Arc};
use yamantaka_domain::{
    Cancellation, Failure, RenderKey, RenderedPage,
    cover_cache::{CoverCacheRequest, CoverImageCache},
    memory::{GlobalMemoryLimits, MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
};
use yamantaka_storage::cover_cache::CoverCache;

fn image(governor: &MemoryGovernor) -> RenderedPage {
    let mut bytes = TrackedBytesBuilder::try_zeroed(governor, MemoryCategory::Pages, 16).unwrap();
    bytes.as_mut_slice().copy_from_slice(&[
        10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
    ]);
    RenderedPage::from_pixels(
        RenderKey {
            document_id: "original".into(),
            page: 1,
            region: None,
            physical_width: 2,
            physical_height: 2,
            rotation: 0,
            generation: 1,
        },
        bytes.finish(),
    )
    .unwrap()
}
fn load(
    cache: &CoverCache,
    identity: [u8; 32],
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> yamantaka_domain::Result<Option<Arc<RenderedPage>>> {
    cache.load(
        CoverCacheRequest {
            identity,
            document: "current",
            page: 1,
            generation: 42,
        },
        governor,
        cancel,
    )
}

#[test]
fn cache_survives_restart_and_rebinds_request_identity_with_governed_pixels() {
    let root = tempfile::tempdir().unwrap();
    let governor = MemoryGovernor::default();
    let pixels = image(&governor);
    let cache = CoverCache::new(root.path().join("covers"), 4096);
    let cancel = Cancellation::default();
    cache.save([3; 32], &pixels, &cancel).unwrap();
    drop(cache);
    let reopened = CoverCache::new(root.path().join("covers"), 4096);
    let before = governor.usage().total;
    let actual = load(&reopened, [3; 32], &governor, &cancel)
        .unwrap()
        .unwrap();
    assert_eq!(
        actual.pixels(),
        &[
            10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255
        ]
    );
    assert_eq!(actual.key.document_id, "current");
    assert_eq!(
        (
            actual.key.page,
            actual.key.generation,
            actual.width,
            actual.height
        ),
        (1, 42, 2, 2)
    );
    assert_eq!(governor.usage().total, before + 16);
    drop(actual);
    assert_eq!(governor.usage().total, before);
    assert!(
        load(&reopened, [4; 32], &governor, &cancel)
            .unwrap()
            .is_none()
    );
}

#[test]
fn corrupt_truncated_oversized_and_wrong_identity_records_are_cache_misses() {
    let root = tempfile::tempdir().unwrap();
    let cache = CoverCache::new(root.path().into(), 4096);
    let governor = MemoryGovernor::default();
    let pixels = image(&governor);
    let cancel = Cancellation::default();
    cache.save([3; 32], &pixels, &cancel).unwrap();
    let path = fs::read_dir(root.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let valid = fs::read(&path).unwrap();
    for damaged in [
        valid[..12].to_vec(),
        {
            let mut data = valid.clone();
            data[40..44].copy_from_slice(&1u32.to_le_bytes());
            data[44..48].copy_from_slice(&4u32.to_le_bytes());
            data
        },
        {
            let mut data = valid.clone();
            data[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
            data
        },
        {
            let mut data = valid.clone();
            data[8] = 9;
            data
        },
        {
            let mut data = valid.clone();
            data[80] ^= 1;
            data
        },
        {
            let mut data = valid.clone();
            data.push(0);
            data
        },
    ] {
        fs::write(&path, damaged).unwrap();
        let before = governor.usage().total;
        assert!(load(&cache, [3; 32], &governor, &cancel).unwrap().is_none());
        assert_eq!(governor.usage().total, before);
    }
    fs::write(&path, &valid).unwrap();
    assert!(load(&cache, [3; 32], &governor, &cancel).unwrap().is_some());
}

#[test]
fn source_page_path_and_file_revision_change_the_cache_identity_without_writing_pdf() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.pdf");
    fs::write(&path, b"synthetic PDF source").unwrap();
    let original = fs::read(&path).unwrap();
    let cache = CoverCache::new(root.path().join("cache"), 4096);
    let cancel = Cancellation::default();
    let initial = cache.identity(&path, 0, &cancel).unwrap();
    assert_eq!(cache.identity(&path, 0, &cancel).unwrap(), initial);
    assert_ne!(cache.identity(&path, 1, &cancel).unwrap(), initial);
    let other = root.path().join("other.pdf");
    fs::write(&other, &original).unwrap();
    assert_ne!(cache.identity(&other, 0, &cancel).unwrap(), initial);
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::write(&path, b"changed-- PDF source").unwrap();
    assert_ne!(cache.identity(&path, 0, &cancel).unwrap(), initial);
}

#[test]
fn cancellation_and_memory_denial_do_not_replace_a_valid_record_or_leak_pixels() {
    let root = tempfile::tempdir().unwrap();
    let cache = CoverCache::new(root.path().into(), 4096);
    let governor = MemoryGovernor::default();
    let pixels = image(&governor);
    cache
        .save([3; 32], &pixels, &Cancellation::default())
        .unwrap();
    let path = fs::read_dir(root.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let original = fs::read(&path).unwrap();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        cache.save([3; 32], &pixels, &cancelled),
        Err(Failure::Cancelled)
    ));
    assert!(matches!(
        load(&cache, [3; 32], &governor, &cancelled),
        Err(Failure::Cancelled)
    ));
    assert_eq!(fs::read(&path).unwrap(), original);
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        total: 0,
        ..Default::default()
    });
    assert!(matches!(
        load(&cache, [3; 32], &denied, &Cancellation::default()),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(denied.usage().total, 0);
}

#[test]
fn disk_quota_evicts_only_owned_records_and_small_limit_skips_writing() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("keep.txt"), b"unrelated").unwrap();
    fs::write(
        root.path().join("é".repeat(32) + ".rgba"),
        b"unrelated unicode",
    )
    .unwrap();
    let governor = MemoryGovernor::default();
    let pixels = image(&governor);
    let cache = CoverCache::new(root.path().into(), 96);
    let cancel = Cancellation::default();
    cache.save([1; 32], &pixels, &cancel).unwrap();
    cache.save([2; 32], &pixels, &cancel).unwrap();
    assert!(load(&cache, [1; 32], &governor, &cancel).unwrap().is_none());
    assert!(load(&cache, [2; 32], &governor, &cancel).unwrap().is_some());
    assert_eq!(
        fs::read(root.path().join("keep.txt")).unwrap(),
        b"unrelated"
    );
    assert_eq!(
        fs::read(root.path().join("é".repeat(32) + ".rgba")).unwrap(),
        b"unrelated unicode"
    );
    CoverCache::new(root.path().into(), 95)
        .save([3; 32], &pixels, &cancel)
        .unwrap();
    assert!(load(&cache, [3; 32], &governor, &cancel).unwrap().is_none());
    let larger = CoverCache::new(root.path().into(), 288);
    larger.save([4; 32], &pixels, &cancel).unwrap();
    larger.save([5; 32], &pixels, &cancel).unwrap();
    let reduced = CoverCache::new(root.path().into(), 96);
    let _ = load(&reduced, [5; 32], &governor, &cancel).unwrap();
    let owned_count = fs::read_dir(root.path())
        .unwrap()
        .filter(|entry| {
            let name = entry.as_ref().unwrap().file_name();
            let name = name.to_string_lossy();
            name.len() == 69 && name.is_ascii() && name.ends_with(".rgba")
        })
        .count();
    assert_eq!(
        owned_count, 1,
        "A smaller reopened quota must apply without a new write"
    );
}
