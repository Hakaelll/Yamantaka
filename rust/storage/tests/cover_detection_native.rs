use std::path::PathBuf;
use yamantaka_domain::{
    Cancellation, Failure,
    memory::{GlobalMemoryLimits, MemoryGovernor},
    pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

fn paths() -> (PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        root.join("build/rust-fixtures/automatic-cover.pdf"),
    )
}

#[test]
fn native_detection_selects_actual_cover_pixels_and_preserves_original() {
    let (dll, path) = paths();
    let before = std::fs::read(&path).unwrap();
    let governor = MemoryGovernor::default();
    let worker =
        PdfWorker::start_with_governor(&dll, MemoryLimits::default(), governor.clone()).unwrap();
    let cancellation = Cancellation::default();
    let ticket = worker
        .try_detect_cover("automatic".into(), path.clone(), 31, cancellation.clone())
        .unwrap();
    assert!(ticket.cancellation.same_request(&cancellation));
    let detected = ticket.wait().unwrap();
    assert_eq!(detected.choice.page, 1);
    assert!(detected.choice.confidence >= 0.45);
    assert_eq!(
        (
            detected.image.key.document_id.as_str(),
            detected.image.key.page,
            detected.image.key.generation
        ),
        ("automatic", 1, 31)
    );
    let exact = worker
        .try_cover(
            "comparison".into(),
            path.clone(),
            1,
            32,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(detected.image.pixels(), exact.pixels());
    drop(exact);
    drop(detected);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn cancelled_and_memory_denied_detection_return_errors_without_retaining_resources() {
    let (dll, path) = paths();
    let governor = MemoryGovernor::new(GlobalMemoryLimits {
        total: 0,
        ..Default::default()
    });
    let worker =
        PdfWorker::start_with_governor(&dll, MemoryLimits::default(), governor.clone()).unwrap();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        worker
            .try_detect_cover("cancelled".into(), path.clone(), 1, cancelled)
            .unwrap()
            .wait(),
        Err(Failure::Cancelled)
    ));
    assert!(matches!(
        worker
            .try_detect_cover("denied".into(), path, 2, Cancellation::default())
            .unwrap()
            .wait(),
        Err(Failure::MemoryBudget)
    ));
    drop(worker);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn detection_tickets_keep_bounded_admission_until_consumed_and_leave_close_available() {
    let (dll, path) = paths();
    let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    let mut tickets = Vec::new();
    for generation in 0..128 {
        tickets.push(
            worker
                .try_detect_cover(
                    "bounded".into(),
                    path.clone(),
                    generation,
                    cancelled.clone(),
                )
                .unwrap(),
        );
    }
    assert!(
        worker
            .try_detect_cover("overflow".into(), path.clone(), 129, cancelled.clone())
            .is_err()
    );
    worker.try_close("unused".into()).unwrap().wait().unwrap();
    for ticket in tickets {
        assert!(matches!(ticket.wait(), Err(Failure::Cancelled)));
    }
    assert!(matches!(
        worker
            .try_detect_cover("admitted".into(), path, 130, cancelled)
            .unwrap()
            .wait(),
        Err(Failure::Cancelled)
    ));
}
