use std::{fs, io::Write, path::PathBuf, sync::Arc};
use yamantaka_domain::{
    Cancellation, PdfEngine, memory::MemoryGovernor, pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::cover_cache::CoverCache;

fn paths() -> (PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        root.join("build/rust-fixtures/textual.pdf"),
        root.join("build/rust-fixtures/automatic-cover.pdf"),
    )
}

#[test]
fn warm_cover_requests_do_not_reuse_another_path_under_the_same_document_id() {
    let (dll, old, requested) = paths();
    for detect in [false, true] {
        let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
        let governor = worker.memory_governor();
        let expected = worker
            .try_cover(
                "expected".into(),
                requested.clone(),
                1,
                1,
                Cancellation::default(),
            )
            .unwrap()
            .wait()
            .unwrap()
            .pixels()
            .to_vec();
        for id in ["warm", "reader"] {
            worker
                .open(id.into(), old.clone(), None, Cancellation::default())
                .unwrap();
        }
        worker.set_visible_documents(&["reader".into()]).unwrap();
        let image = if detect {
            let result = worker
                .try_detect_cover("warm".into(), requested.clone(), 2, Cancellation::default())
                .unwrap()
                .wait()
                .unwrap();
            assert_eq!(result.choice.page, 1);
            result.image
        } else {
            worker
                .try_cover(
                    "warm".into(),
                    requested.clone(),
                    1,
                    2,
                    Cancellation::default(),
                )
                .unwrap()
                .wait()
                .unwrap()
        };
        assert!(
            image.pixels() == expected,
            "The cover must use the requested file"
        );
        assert!(
            worker
                .text("warm".into(), 0, Cancellation::default())
                .is_err()
        );
        assert!(
            worker
                .text("reader".into(), 0, Cancellation::default())
                .unwrap()
                .contains("Yamantaka Rust")
        );
        drop(image);
        drop(worker);
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn changed_paths_preserve_protected_readers_and_only_allow_verified_cached_pixels() {
    let (dll, old, requested) = paths();
    for cached in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let governor = MemoryGovernor::default();
        let worker = PdfWorker::start_with_cover_cache(
            &dll,
            MemoryLimits::default(),
            governor.clone(),
            cached.then(|| {
                Arc::new(CoverCache::new(root.path().into(), 4 * 1024 * 1024))
                    as Arc<dyn yamantaka_domain::cover_cache::CoverImageCache>
            }),
        )
        .unwrap();
        let expected = worker
            .try_cover(
                "seed".into(),
                requested.clone(),
                1,
                1,
                Cancellation::default(),
            )
            .unwrap()
            .wait()
            .unwrap()
            .pixels()
            .to_vec();
        for id in ["active", "other"] {
            worker
                .open(id.into(), old.clone(), None, Cancellation::default())
                .unwrap();
        }
        worker
            .set_visible_documents(&["active".into(), "other".into()])
            .unwrap();
        let result = worker
            .try_cover(
                "active".into(),
                requested.clone(),
                1,
                2,
                Cancellation::default(),
            )
            .unwrap()
            .wait();
        if cached {
            assert!(
                result.unwrap().pixels() == expected,
                "The cached cover must use the requested file"
            );
        } else {
            assert!(result.is_err());
        }
        assert!(
            worker
                .try_detect_cover(
                    "active".into(),
                    requested.clone(),
                    3,
                    Cancellation::default()
                )
                .unwrap()
                .wait()
                .is_err()
        );
        for id in ["active", "other"] {
            assert!(
                worker
                    .text(id.into(), 0, Cancellation::default())
                    .unwrap()
                    .contains("Yamantaka Rust")
            );
        }
        drop(worker);
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn changed_files_are_rejected_when_protected_and_detached_when_warm() {
    let (dll, original, _) = paths();
    for protected in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source.pdf");
        fs::copy(&original, &path).unwrap();
        let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
        let governor = worker.memory_governor();
        worker
            .open("source".into(), path.clone(), None, Cancellation::default())
            .unwrap();
        assert!(
            worker
                .text("source".into(), 0, Cancellation::default())
                .unwrap()
                .contains("Yamantaka Rust")
        );
        if protected {
            worker.set_visible_documents(&["source".into()]).unwrap();
        }
        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"\n% synthetic source revision\n")
            .unwrap();
        let result = worker
            .try_cover("source".into(), path.clone(), 0, 2, Cancellation::default())
            .unwrap()
            .wait();
        if protected {
            assert!(result.is_err());
            assert!(
                worker
                    .text("source".into(), 0, Cancellation::default())
                    .unwrap()
                    .contains("Yamantaka Rust")
            );
        } else {
            assert!(result.is_ok());
            assert!(
                worker
                    .text("source".into(), 0, Cancellation::default())
                    .is_err()
            );
        }
        drop(result);
        drop(worker);
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn a_changed_header_is_detected_even_when_file_size_and_modification_time_are_preserved() {
    let (dll, original, _) = paths();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("same-metadata.pdf");
    fs::copy(&original, &path).unwrap();
    let before = fs::metadata(&path).unwrap();
    let modified = before.modified().unwrap();
    let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let governor = worker.memory_governor();
    worker
        .open("source".into(), path.clone(), None, Cancellation::default())
        .unwrap();
    let mut bytes = fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"%PDF-1."));
    bytes[7] = if bytes[7] == b'7' { b'6' } else { b'7' };
    fs::write(&path, bytes).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
    let after = fs::metadata(&path).unwrap();
    assert_eq!(after.len(), before.len());
    assert_eq!(after.modified().unwrap(), modified);
    let image = worker
        .try_cover("source".into(), path, 0, 2, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    assert!(
        worker
            .text("source".into(), 0, Cancellation::default())
            .is_err()
    );
    drop(image);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
}
