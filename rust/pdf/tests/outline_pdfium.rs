use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use yamantaka_domain::{
    Cancellation, Failure, PdfEngine,
    pdf_destination::{DestinationView, NavigationDestination},
    pdf_geometry::PageCoordinateSpace,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn title(value: &str) -> String {
    format!(
        "<FEFF{}>",
        value
            .encode_utf16()
            .map(|unit| format!("{unit:04X}"))
            .collect::<String>()
    )
}
fn fixture(mode: &str) -> Fixture {
    let heading = if mode == "title-limit" {
        "x".repeat(5000)
    } else {
        "Résumé · 宇宙".into()
    };
    let first_target = if mode == "bad-target" {
        "999 0 R"
    } else {
        "3 0 R"
    };
    let next = if mode == "cycle" { "7 0 R" } else { "9 0 R" };
    let mut objects = vec![
        format!("<< /Type /Catalog /Pages 2 0 R {} >>", if mode=="empty" { "" } else { "/Outlines 6 0 R" }),
        "<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>".into(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources <<>> /Contents 5 0 R >>".into(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources <<>> /Contents 5 0 R >>".into(),
        "<< /Length 0 >>\nstream\n\nendstream".into(),
        "<< /Type /Outlines /First 7 0 R /Last 11 0 R /Count 5 >>".into(),
        format!("<< /Title {} /Parent 6 0 R /Next {next} /First 8 0 R /Last 8 0 R /Count 1 /Dest [{first_target} /XYZ 40 150 1.5] >>", title(&heading)),
        format!("<< /Title {} /Parent 7 0 R /Dest [4 0 R /FitH 170] >>", title("Nested passage")),
        "<< /Title (External) /Parent 6 0 R /Prev 7 0 R /Next 10 0 R /A << /S /URI /URI (https://example.invalid/) >> >>".into(),
        "<< /Title (Rectangle) /Parent 6 0 R /Prev 9 0 R /Next 11 0 R /A << /S /GoTo /D [4 0 R /FitR 20 30 100 150] >> >>".into(),
        "<< /Title (Heading) /Parent 6 0 R /Prev 10 0 R >>".into(),
    ];
    if mode == "entry-limit" {
        objects.truncate(6);
        objects[5] = "<< /Type /Outlines /First 7 0 R /Last 10007 0 R /Count 10001 >>".into();
        for index in 0..10001 {
            let sibling = if index == 10000 {
                String::new()
            } else {
                format!("/Next {} 0 R", index + 8)
            };
            objects.push(format!("<< /Title (Passage) /Parent 6 0 R {sibling} >>"));
        }
    }
    if mode.starts_with("fit-bounds") || mode == "object-limit" || mode == "object-work-limit" {
        objects[6] = format!(
            "<< /Title {} /Parent 6 0 R /Next 9 0 R /First 8 0 R /Last 8 0 R /Count 1 /Dest [3 0 R /FitB] >>",
            title(&heading)
        );
        let content = match mode {
            "object-limit" => "0 0 1 1 re f\n".repeat(100_001),
            "object-work-limit" => {
                objects[7] = format!(
                    "<< /Title {} /Parent 7 0 R /Dest [3 0 R /FitB] >>",
                    title("Repeated bounds")
                );
                "0 0 1 1 re f\n".repeat(60_000)
            }
            "fit-bounds-cropped" => "-20 -40 300 400 re f".into(),
            _ => "20 30 80 120 re f".into(),
        };
        if mode == "fit-bounds-cropped" {
            objects[2] = "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 220] /CropBox [20 30 120 180] /Resources <<>> /Contents 5 0 R >>".into();
        }
        objects[4] = format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        );
    }
    if let Some(rotation) = mode.strip_prefix("rotated-") {
        objects[2] = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 220] /CropBox [20 30 120 180] /Rotate {rotation} /Resources <<>> /Contents 5 0 R >>"
        );
    }
    let mut pdf = b"%PDF-1.7\n".to_vec();
    let mut offsets = vec![0];
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in &offsets[1..] {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    let path = std::env::temp_dir().join(format!(
        "yamantaka-outline-{}-{}.pdf",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    std::io::Write::write_all(&mut file, &pdf).unwrap();
    Fixture(path)
}
fn engine() -> PdfWorker {
    PdfWorker::start(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap()
}

#[test]
fn native_outline_preserves_unicode_hierarchy_destinations_and_shared_memory_lifetime() {
    let fixture = fixture("normal");
    let before = Sha256::digest(fs::read(&fixture.0).unwrap());
    let engine = engine();
    let governor = engine.memory_governor();
    engine
        .open("a".into(), fixture.0.clone(), None, Cancellation::default())
        .unwrap();
    let outline = engine.outline("a".into(), Cancellation::default()).unwrap();
    let entries = outline.entries();
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].title, "Résumé · 宇宙");
    assert_eq!(entries[0].parent, None);
    assert_eq!(
        entries[0].destination,
        Some(NavigationDestination {
            page: 0,
            content_bounds: None,
            coordinates: PageCoordinateSpace::new([0.0, 200.0], [200.0, 200.0], [0.0, 0.0])
                .unwrap(),
            view: DestinationView::Coordinates {
                x: Some(40.0),
                y: Some(150.0),
                zoom: Some(1.5)
            }
        })
    );
    assert_eq!(entries[1].title, "Nested passage");
    assert_eq!(entries[1].parent, Some(0));
    assert_eq!(
        entries[1].destination,
        Some(NavigationDestination {
            page: 1,
            content_bounds: None,
            coordinates: PageCoordinateSpace::new([0.0, 200.0], [200.0, 200.0], [0.0, 0.0])
                .unwrap(),
            view: DestinationView::FitWidth { top: Some(170.0) }
        })
    );
    assert_eq!(entries[2].title, "External");
    assert_eq!(entries[2].destination, None);
    assert_eq!(
        entries[3].destination,
        Some(NavigationDestination {
            page: 1,
            content_bounds: None,
            coordinates: PageCoordinateSpace::new([0.0, 200.0], [200.0, 200.0], [0.0, 0.0])
                .unwrap(),
            view: DestinationView::FitRectangle {
                left: 20.0,
                bottom: 30.0,
                right: 100.0,
                top: 150.0
            }
        })
    );
    assert_eq!(entries[4].destination, None);
    let retained = governor.usage().text;
    assert!(retained > 0 && retained < 4 * 1024 * 1024);
    let other_owner = outline.clone();
    engine.close("a".into()).unwrap();
    drop(outline);
    assert_eq!(governor.usage().text, retained);
    drop(other_owner);
    assert_eq!(governor.usage().text, 0);
    assert_eq!(Sha256::digest(fs::read(&fixture.0).unwrap()), before);
}

#[test]
fn empty_cyclic_invalid_and_cancelled_outlines_are_bounded_and_recoverable() {
    let engine = engine();
    let governor = engine.memory_governor();
    for (mode, count) in [("empty", 0), ("cycle", 2)] {
        let fixture = fixture(mode);
        engine
            .open("a".into(), fixture.0.clone(), None, Cancellation::default())
            .unwrap();
        let outline = engine.outline("a".into(), Cancellation::default()).unwrap();
        assert_eq!(outline.entries().len(), count);
        drop(outline);
        assert_eq!(governor.usage().text, 0);
        engine.close("a".into()).unwrap();
    }
    for mode in [
        "title-limit",
        "bad-target",
        "entry-limit",
        "object-limit",
        "object-work-limit",
    ] {
        let fixture = fixture(mode);
        engine
            .open("a".into(), fixture.0.clone(), None, Cancellation::default())
            .unwrap();
        assert!(matches!(
            engine.outline("a".into(), Cancellation::default()),
            Err(Failure::Pdf(_))
        ));
        assert_eq!(governor.usage().text, 0);
        let cancelled = Cancellation::default();
        cancelled.cancel();
        assert!(matches!(
            engine.outline("a".into(), cancelled),
            Err(Failure::Cancelled)
        ));
        engine.close("a".into()).unwrap();
        assert!(engine.outline("a".into(), Cancellation::default()).is_err());
    }
    let fixture = fixture("normal");
    engine
        .open("a".into(), fixture.0.clone(), None, Cancellation::default())
        .unwrap();
    let original_limits = governor.limits();
    governor.set_limits(yamantaka_domain::memory::GlobalMemoryLimits {
        text: 1024 * 1024,
        ..original_limits
    });
    assert!(matches!(
        engine.outline("a".into(), Cancellation::default()),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(governor.usage().text, 0);
    governor.set_limits(original_limits);
    let outline = engine.outline("a".into(), Cancellation::default()).unwrap();
    assert_eq!(outline.entries().len(), 5);
    drop(outline);
    engine.close("a".into()).unwrap();
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn native_destination_geometry_matches_cropped_pages_at_every_intrinsic_rotation() {
    let engine = engine();
    for (rotation, expected) in [
        (0, [0.2, 0.2]),
        (90, [0.8, 0.2]),
        (180, [0.8, 0.8]),
        (270, [0.2, 0.8]),
    ] {
        let fixture = fixture(&format!("rotated-{rotation}"));
        let info = engine
            .open(
                "geometry".into(),
                fixture.0.clone(),
                None,
                Cancellation::default(),
            )
            .unwrap();
        let outline = engine
            .outline("geometry".into(), Cancellation::default())
            .unwrap();
        let destination = outline.entries()[0].destination.unwrap();
        assert_eq!(info.pages[0].coordinates(), destination.coordinates);
        let cloned = info.clone();
        assert!(std::sync::Arc::ptr_eq(&info.pages, &cloned.pages));
        let retained = engine.memory_governor().usage().other;
        assert!(retained > 0);
        let actual = destination.coordinates.normalized_point([40.0, 150.0]);
        for axis in 0..2 {
            assert!(
                (actual[axis] - expected[axis]).abs() < 1e-6,
                "rotation {rotation}: {actual:?}"
            );
        }
        assert_eq!(destination.coordinates.pdf_point(actual), [40.0, 150.0]);
        engine.close("geometry".into()).unwrap();
        drop(info);
        assert_eq!(engine.memory_governor().usage().other, retained);
        assert_eq!(cloned.pages[0].coordinates(), destination.coordinates);
        drop(cloned);
        assert_eq!(engine.memory_governor().usage().other, 0);
    }
}

#[test]
fn fit_bounds_uses_page_content_instead_of_the_crop_rectangle() {
    let engine = engine();
    for (mode, expected) in [
        ("fit-bounds", [20.0, 30.0, 100.0, 150.0]),
        ("fit-bounds-cropped", [20.0, 30.0, 120.0, 180.0]),
    ] {
        let fixture = fixture(mode);
        let before = Sha256::digest(fs::read(&fixture.0).unwrap());
        engine
            .open(
                "bounds".into(),
                fixture.0.clone(),
                None,
                Cancellation::default(),
            )
            .unwrap();
        let outline = engine
            .outline("bounds".into(), Cancellation::default())
            .unwrap();
        let destination = outline.entries()[0].destination.unwrap();
        assert_eq!(destination.view, DestinationView::FitBounds);
        assert_eq!(destination.content_bounds.unwrap().bounds(), expected);
        engine.close("bounds".into()).unwrap();
        assert_eq!(Sha256::digest(fs::read(&fixture.0).unwrap()), before);
    }
}
