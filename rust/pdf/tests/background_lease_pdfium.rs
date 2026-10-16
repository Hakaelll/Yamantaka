use std::{fs, path::PathBuf};
use yamantaka_domain::{
    Cancellation, Failure, PdfEngine,
    pdf_text::TextGeometryPurpose,
    pdf_ticket::{AsyncPdfEngine, PdfOpenPurpose},
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn cover_work_cannot_evict_an_active_background_analysis_handle() {
    let root = root();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    let reader = root.join("build/rust-fixtures/textual.pdf");
    let analysis = root.join("build/rust-fixtures/front-matter-missing.pdf");
    let cover = root.join("build/rust-fixtures/automatic-cover.pdf");
    let originals = [
        fs::read(&reader).unwrap(),
        fs::read(&analysis).unwrap(),
        fs::read(&cover).unwrap(),
    ];
    let expected = worker
        .try_cover(
            "expected".into(),
            cover.clone(),
            1,
            1,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap()
        .pixels()
        .to_vec();
    let reader_info = worker
        .open(
            "reader".into(),
            reader.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    worker.set_visible_documents(&["reader".into()]).unwrap();
    let analysis_info = worker
        .try_open(
            "analysis".into(),
            analysis.clone(),
            None,
            PdfOpenPurpose::Background,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    let result = worker
        .try_cover("cover".into(), cover.clone(), 1, 2, Cancellation::default())
        .unwrap()
        .wait();
    assert!(
        matches!(result, Err(Failure::NativeSlotsBusy)),
        "Cover work must wait for background analysis to release its handle"
    );
    let text = worker
        .try_text_geometry(
            "analysis".into(),
            0,
            TextGeometryPurpose::FrontMatter,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert!(text.text().contains("Geometry of Light"));
    worker.close("analysis".into()).unwrap();
    let image = worker
        .try_cover("cover".into(), cover.clone(), 1, 3, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(image.pixels(), expected);
    assert!(
        worker
            .text("reader".into(), 0, Cancellation::default())
            .unwrap()
            .contains("Yamantaka Rust")
    );
    for (path, bytes) in [reader, analysis, cover].into_iter().zip(originals) {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    drop((reader_info, analysis_info, text, image));
    drop(worker);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn a_third_background_open_waits_for_release_without_replacing_either_active_analysis() {
    let root = root();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    let analysis = root.join("build/rust-fixtures/front-matter-missing.pdf");
    let mut infos = Vec::new();
    for id in ["first", "second"] {
        infos.push(
            worker
                .try_open(
                    id.into(),
                    analysis.clone(),
                    None,
                    PdfOpenPurpose::Background,
                    Cancellation::default(),
                )
                .unwrap()
                .wait()
                .unwrap(),
        );
    }
    let third = worker
        .try_open(
            "third".into(),
            root.join("build/rust-fixtures/automatic-cover.pdf"),
            None,
            PdfOpenPurpose::Background,
            Cancellation::default(),
        )
        .unwrap();
    let text = worker
        .try_text_geometry(
            "first".into(),
            0,
            TextGeometryPurpose::Reader,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert!(text.text().contains("Geometry of Light"));
    assert!(
        !third.is_ready(),
        "Both active background handles retain their leases"
    );
    worker.close("second".into()).unwrap();
    let third = third.wait().unwrap();
    assert_eq!(third.pages.len(), 7);
    assert!(
        worker
            .text("first".into(), 0, Cancellation::default())
            .unwrap()
            .contains("Geometry of Light")
    );
    drop((infos, text, third));
    drop(worker);
    assert_eq!(governor.usage().total, 0);
}
