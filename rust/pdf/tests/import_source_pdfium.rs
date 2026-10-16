use std::{fs, path::PathBuf};
use yamantaka_domain::{
    Cancellation, Failure, PdfEngine, pdf_text::TextGeometryPurpose, pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn metadata_text_work_bound_does_not_truncate_or_limit_reader_text() {
    let root = root();
    let path = root.join("build/rust-fixtures/front-matter-work-limit.pdf");
    let before = fs::read(&path).unwrap();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    let info = worker
        .open("source".into(), path.clone(), None, Cancellation::default())
        .unwrap();
    let background = worker
        .try_text_geometry(
            "source".into(),
            0,
            TextGeometryPurpose::FrontMatter,
            Cancellation::default(),
        )
        .unwrap()
        .wait();
    assert!(matches!(background, Err(Failure::TextWorkLimit)));
    assert_eq!(governor.usage().text, 0);
    let text = worker
        .try_text_geometry(
            "source".into(),
            0,
            TextGeometryPurpose::Reader,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert!(text.characters().len() > 16_000);
    assert!(text.text().starts_with("word word"));
    drop(text);
    drop(info);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn source_revision_has_opened_file_identity_and_obeys_cancellation_and_close() {
    let root = root();
    let path = root.join("build/rust-fixtures/front-matter-missing.pdf");
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let info = worker
        .open("source".into(), path.clone(), None, Cancellation::default())
        .unwrap();
    let ticket = worker
        .try_source_revision("source".into(), Cancellation::default())
        .unwrap();
    let epoch = ticket.epoch;
    let revision = ticket.wait().unwrap();
    assert_eq!(revision.canonical_path, path.canonicalize().unwrap());
    assert_eq!(revision.size, fs::metadata(&path).unwrap().len());
    worker.set_visible_documents(&["source".into()]).unwrap();
    let text = worker
        .try_text("source".into(), 0, Cancellation::default())
        .unwrap();
    assert!(
        worker
            .try_open(
                "source".into(),
                root.join("build/rust-fixtures/textual.pdf"),
                None,
                yamantaka_domain::pdf_ticket::PdfOpenPurpose::Background,
                Cancellation::default()
            )
            .is_err()
    );
    assert!(text.wait().unwrap().contains("Geometry of Light"));
    let latest = worker
        .try_source_revision("source".into(), Cancellation::default())
        .unwrap();
    assert_eq!(latest.epoch, epoch);
    assert_eq!(latest.wait().unwrap(), revision);
    let stopped = Cancellation::default();
    stopped.cancel();
    assert!(matches!(
        worker
            .try_source_revision("source".into(), stopped)
            .unwrap()
            .wait(),
        Err(Failure::Cancelled)
    ));
    worker.close("source".into()).unwrap();
    assert!(
        worker
            .try_source_revision("source".into(), Cancellation::default())
            .unwrap()
            .wait()
            .is_err()
    );
    drop(info);
}
