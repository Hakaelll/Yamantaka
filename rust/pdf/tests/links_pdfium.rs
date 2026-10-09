use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use yamantaka_domain::{
    Cancellation, Failure, PdfEngine,
    memory::{GlobalMemoryLimits, MemoryGovernor},
    pdf_destination::DestinationView,
    pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}
fn fixture(rotation: u16) -> Fixture {
    fixture_with_annotations(rotation, None)
}
fn fixture_with_annotations(rotation: u16, repeated: Option<usize>) -> Fixture {
    fixture_with_uri(rotation, repeated, "https://example.invalid")
}
fn fixture_with_uri(rotation: u16, repeated: Option<usize>, uri: &str) -> Fixture {
    let annotations = repeated.map_or_else(
        || "5 0 R 6 0 R 7 0 R 8 0 R".to_owned(),
        |count| "5 0 R ".repeat(count),
    );
    fixture_with_references(rotation, &annotations, uri)
}
fn fixture_with_references(rotation: u16, annotations: &str, uri: &str) -> Fixture {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>".into(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /CropBox [20 30 580 780] /Rotate {rotation} /Annots [{annotations}] >>"),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] >>".into(),
        "<< /Type /Annot /Subtype /Link /Rect [60 680 200 720] /QuadPoints [60 720 120 720 60 700 120 700 140 700 200 700 140 680 200 680] /Dest [4 0 R /XYZ 64 700 1.5] >>".into(),
        "<< /Type /Annot /Subtype /Link /Rect [60 600 200 620] /A << /S /GoTo /D [4 0 R /FitH 500] >> >>".into(),
        format!("<< /Type /Annot /Subtype /Link /Rect [60 500 200 520] /A << /S /URI /URI ({uri}) >> >>"),
        "<< /Type /Annot /Subtype /Link /Rect [0 0 10 10] /Dest [4 0 R /Fit] >>".into(),
    ];
    let mut bytes = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
    }
    let xref = bytes.len();
    bytes.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    let path = std::env::temp_dir().join(format!(
        "yamantaka-links-{}-{}.pdf",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, bytes).unwrap();
    Fixture(path)
}
fn engine(governor: MemoryGovernor) -> PdfWorker {
    PdfWorker::start_with_governor(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
        governor,
    )
    .unwrap()
}
#[test]
fn native_links_preserve_separate_regions_destinations_crop_rotation_and_source() {
    let governor = MemoryGovernor::default();
    let engine = engine(governor.clone());
    for rotation in [0, 90, 180, 270] {
        let fixture = fixture(rotation);
        let original = Sha256::digest(fs::read(&fixture.0).unwrap());
        let info = engine
            .open(
                "book".into(),
                fixture.0.clone(),
                None,
                Cancellation::default(),
            )
            .unwrap();
        let links = engine
            .try_page_links("book".into(), 0, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(links.page(), 0);
        assert_eq!(links.links().len(), 3);
        assert_eq!(links.unsupported_targets(), 0);
        let first = &links.links()[0];
        assert_eq!(first.annotation_index, 0);
        let yamantaka_domain::pdf_links::PdfLinkTarget::Internal(destination) = first.target else {
            panic!("expected internal destination");
        };
        assert_eq!(destination.page, 1);
        assert_eq!(
            destination.view,
            DestinationView::Coordinates {
                x: Some(64.0),
                y: Some(700.0),
                zoom: Some(1.5)
            }
        );
        assert_eq!(first.regions.len(), 2);
        for (region, expected) in first.regions.iter().zip([[90.0, 710.0], [170.0, 690.0]]) {
            let center = [
                (region.min()[0] + region.max()[0]) / 2.0,
                (region.min()[1] + region.max()[1]) / 2.0,
            ];
            let native = info.pages[0].coordinates().pdf_point(center);
            assert!(
                (native[0] - expected[0]).abs() < 0.01 && (native[1] - expected[1]).abs() < 0.01,
                "rotation {rotation}: {native:?}"
            );
        }
        assert_eq!(
            match links.links()[1].target {
                yamantaka_domain::pdf_links::PdfLinkTarget::Internal(destination) =>
                    destination.view,
                _ => panic!("expected local GoTo"),
            },
            DestinationView::FitWidth { top: Some(500.0) }
        );
        assert!(
            engine
                .try_page_links("book".into(), 1, Cancellation::default())
                .unwrap()
                .wait()
                .unwrap()
                .links()
                .is_empty()
        );
        let retained = links.clone();
        engine.close("book".into()).unwrap();
        drop(links);
        assert!(governor.usage().text > 0);
        drop(retained);
        assert_eq!(governor.usage().text, 0);
        assert_eq!(Sha256::digest(fs::read(&fixture.0).unwrap()), original);
    }
}
#[test]
fn cancelled_and_invalid_page_requests_release_resources() {
    let fixture = fixture(0);
    let governor = MemoryGovernor::default();
    let engine = engine(governor.clone());
    engine
        .open(
            "book".into(),
            fixture.0.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        engine
            .try_page_links("book".into(), 0, cancelled)
            .unwrap()
            .wait(),
        Err(Failure::Cancelled)
    ));
    for page in [2, u32::MAX] {
        assert!(
            engine
                .try_page_links("book".into(), page, Cancellation::default())
                .unwrap()
                .wait()
                .is_err()
        );
    }
    assert_eq!(governor.usage().text, 0);
    engine.close("book".into()).unwrap();
    let limited = engine_with_limit();
    limited
        .open(
            "book".into(),
            fixture.0.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    assert!(matches!(
        limited
            .try_page_links("book".into(), 0, Cancellation::default())
            .unwrap()
            .wait(),
        Err(Failure::MemoryBudget)
    ));
    limited.close("book".into()).unwrap();
}
fn engine_with_limit() -> PdfWorker {
    engine(MemoryGovernor::new(GlobalMemoryLimits {
        text: 1024,
        ..GlobalMemoryLimits::default()
    }))
}

#[test]
fn excessive_annotations_and_links_fail_without_retaining_partial_payloads() {
    let governor = MemoryGovernor::default();
    let engine = engine(governor.clone());
    for count in [513, 10_001] {
        let fixture = fixture_with_annotations(0, Some(count));
        engine
            .open(
                "book".into(),
                fixture.0.clone(),
                None,
                Cancellation::default(),
            )
            .unwrap();
        let error = engine
            .try_page_links("book".into(), 0, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap_err();
        assert!(matches!(error, Failure::Pdf(_)));
        assert_eq!(governor.usage().text, 0);
        engine.close("book".into()).unwrap();
    }
}

#[test]
fn native_uri_reader_checks_the_limit_before_allocation_and_preserves_existing_api() {
    use pdfium_render::prelude::*;
    let fixture = fixture(0);
    let original = Sha256::digest(fs::read(&fixture.0).unwrap());
    let worker = engine(MemoryGovernor::default());
    let pdfium = Pdfium::default();
    let document = pdfium.load_pdf_from_file(&fixture.0, None).unwrap();
    let page = document.pages().get(0).unwrap();
    let annotation = page.annotations().get(2).unwrap();
    let link = annotation.as_link_annotation().unwrap().link().unwrap();
    let action = link.action().unwrap();
    let uri = action.as_uri_action().unwrap();
    let expected = "https://example.invalid";
    for limit in [0, expected.len() - 1] {
        assert!(matches!(
            uri.uri_with_max_bytes(limit),
            Err(PdfiumError::UriLengthExceedsLimit)
        ));
    }
    for limit in [expected.len(), expected.len() + 1, usize::MAX] {
        assert_eq!(uri.uri_with_max_bytes(limit).unwrap(), expected);
    }
    assert_eq!(uri.uri().unwrap(), expected);
    assert_eq!(Sha256::digest(fs::read(&fixture.0).unwrap()), original);
    drop(worker);
}

#[test]
fn native_external_links_are_validated_bounded_accounted_and_never_opened_by_extraction() {
    use yamantaka_domain::pdf_links::PdfLinkTarget;
    for value in [
        "https://example.invalid/path",
        "http://example.invalid",
        "mailto:study",
        "file:///document.pdf",
        "javascript:alert",
        "https:",
        &format!("https://example.invalid/{}", "a".repeat(4096)),
    ] {
        let fixture = fixture_with_uri(0, None, value);
        let before = Sha256::digest(fs::read(&fixture.0).unwrap());
        let governor = MemoryGovernor::default();
        let engine = engine(governor.clone());
        engine
            .open(
                "book".into(),
                fixture.0.clone(),
                None,
                Cancellation::default(),
            )
            .unwrap();
        let links = engine
            .try_page_links("book".into(), 0, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap();
        let allowed = value.starts_with("http://")
            || value.starts_with("https://") && value.len() <= 4096
            || value.starts_with("mailto:");
        assert_eq!(links.links().len(), 3);
        assert_eq!(links.unsupported_targets(), if allowed { 0 } else { 1 });
        if allowed {
            let PdfLinkTarget::External(uri) = &links.links()[2].target else {
                panic!("expected external URI");
            };
            assert!(uri.as_str().starts_with(value.split(':').next().unwrap()));
            assert!(governor.usage().text >= uri.retained_bytes());
        }
        if !allowed {
            assert!(matches!(
                links.links()[2].target,
                PdfLinkTarget::Rejected(_)
            ));
            assert!(!links.links()[2].regions.is_empty());
        }
        drop(links);
        assert_eq!(governor.usage().text, 0);
        engine.close("book".into()).unwrap();
        assert_eq!(Sha256::digest(fs::read(&fixture.0).unwrap()), before);
    }
}

#[test]
fn aggregate_uri_capacity_fails_boundedly_and_the_worker_recovers() {
    let uri = format!(
        "https://example.invalid/{}",
        "a".repeat(4096 - "https://example.invalid/".len())
    );
    let fixture = fixture_with_references(0, &"7 0 R ".repeat(140), &uri);
    let governor = MemoryGovernor::default();
    let engine = engine(governor.clone());
    engine
        .open(
            "book".into(),
            fixture.0.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    assert!(matches!(
        engine
            .try_page_links("book".into(), 0, Cancellation::default())
            .unwrap()
            .wait(),
        Err(Failure::Pdf(_))
    ));
    assert_eq!(governor.usage().text, 0);
    assert!(
        engine
            .try_page_links("book".into(), 1, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap()
            .links()
            .is_empty()
    );
    assert_eq!(governor.usage().text, 0);
    engine.close("book".into()).unwrap();
}
