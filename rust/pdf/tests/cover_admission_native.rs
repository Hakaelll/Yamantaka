use std::path::PathBuf;
use yamantaka_domain::{Cancellation, Failure, PdfEngine, pdf_ticket::AsyncPdfEngine};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

#[test]
fn cover_jobs_reclaim_only_unprotected_handles_and_release_the_temporary_slot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("build/rust-fixtures/automatic-cover.pdf");
    let original = std::fs::read(&path).unwrap();
    for detect in [false, true] {
        let worker = PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap();
        let governor = worker.memory_governor();
        let expected = worker
            .try_cover(
                "baseline".into(),
                path.clone(),
                1,
                1,
                Cancellation::default(),
            )
            .unwrap()
            .wait()
            .unwrap()
            .pixels()
            .to_vec();
        for id in ["reader", "warm"] {
            worker
                .open(id.into(), path.clone(), None, Cancellation::default())
                .unwrap();
        }
        worker.set_visible_documents(&["reader".into()]).unwrap();
        let cancel = Cancellation::default();
        cancel.cancel();
        assert!(matches!(
            worker
                .try_cover("cancelled".into(), path.clone(), 1, 2, cancel)
                .unwrap()
                .wait(),
            Err(Failure::Cancelled)
        ));
        assert!(
            !worker
                .text("warm".into(), 0, Cancellation::default())
                .unwrap()
                .is_empty()
        );
        let image = if detect {
            let result = worker
                .try_detect_cover("gallery".into(), path.clone(), 3, Cancellation::default())
                .unwrap()
                .wait()
                .unwrap();
            assert_eq!(result.choice.page, 1);
            result.image
        } else {
            worker
                .try_cover(
                    "gallery".into(),
                    path.clone(),
                    1,
                    3,
                    Cancellation::default(),
                )
                .unwrap()
                .wait()
                .unwrap()
        };
        assert_eq!(image.pixels(), expected);
        assert!(
            !worker
                .text("reader".into(), 0, Cancellation::default())
                .unwrap()
                .is_empty()
        );
        assert!(
            worker
                .text("warm".into(), 0, Cancellation::default())
                .is_err(),
            "Eviction must clear the warm text cache as well as its handle"
        );
        worker
            .open("next".into(), path.clone(), None, Cancellation::default())
            .unwrap();
        worker
            .set_visible_documents(&["reader".into(), "next".into()])
            .unwrap();
        assert!(
            worker
                .try_cover(
                    "blocked".into(),
                    path.clone(),
                    1,
                    4,
                    Cancellation::default()
                )
                .unwrap()
                .wait()
                .is_err()
        );
        assert!(
            worker
                .try_detect_cover("blocked".into(), path.clone(), 5, Cancellation::default())
                .unwrap()
                .wait()
                .is_err()
        );
        for id in ["reader", "next"] {
            assert!(
                !worker
                    .text(id.into(), 0, Cancellation::default())
                    .unwrap()
                    .is_empty()
            );
        }
        drop(image);
        drop(worker);
        assert_eq!(governor.usage().total, 0);
    }
    assert_eq!(std::fs::read(path).unwrap(), original);
}
