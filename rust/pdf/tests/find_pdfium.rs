use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use yamantaka_domain::{
    Cancellation, Failure, PdfEngine,
    memory::{GlobalMemoryLimits, MemoryGovernor},
    pdf_find::PdfFindQuery,
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
    let mut text = "BT /F1 5 Tf 60 760 Td (needle Needle needlework nEeDlE) Tj\n".to_owned();
    for _ in 0..100 {
        text.push_str("0 -5 Td (needle) Tj\n");
    }
    text.push_str("ET");
    let other = "BT /F1 12 Tf 60 700 Td (caf\\351 caf\\351s) Tj ET";
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>".into(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /CropBox [20 30 580 780] /Rotate {rotation} /Resources << /Font << /F1 7 0 R >> >> /Contents 4 0 R >>"),
        format!("<< /Length {} >>\nstream\n{text}\nendstream", text.len()),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /Resources << /Font << /F1 7 0 R >> >> /Contents 6 0 R >>".into(),
        format!("<< /Length {} >>\nstream\n{other}\nendstream", other.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into(),
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
        "yamantaka-find-{}-{}.pdf",
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
fn find(
    engine: &PdfWorker,
    page: u32,
    text: &str,
    case: bool,
    words: bool,
    offset: u32,
) -> std::sync::Arc<yamantaka_domain::pdf_find::PdfPageMatches> {
    engine
        .try_find_page(
            "book".into(),
            page,
            PdfFindQuery::new(text, case, words).unwrap(),
            offset,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap()
}

#[test]
fn native_search_preserves_match_order_options_unicode_and_page_geometry() {
    let fixture = fixture(0);
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
    let first = find(&engine, 0, "needle", false, false, 0);
    assert_eq!(first.matches().len(), 80);
    assert_eq!(first.next_match(), Some(80));
    assert_eq!(first.matches().first().unwrap().ordinal, 0);
    assert_eq!(first.matches().last().unwrap().ordinal, 79);
    assert!(
        first
            .matches()
            .iter()
            .all(|matched| !matched.rectangles.is_empty() && matched.position(0).is_some())
    );
    let second = find(&engine, 0, "needle", false, false, 80);
    assert_eq!(second.matches().len(), 24);
    assert_eq!(second.next_match(), None);
    let exact = find(&engine, 0, "needle", true, true, 80);
    assert_eq!(exact.matches().len(), 21);
    let words = find(&engine, 0, "needle", false, true, 80);
    assert_eq!(words.matches().len(), 23);
    let unicode = find(&engine, 1, "café", true, true, 0);
    assert_eq!(unicode.matches().len(), 1);
    assert!(unicode.matches()[0].snippet.contains("café"));
    assert!(
        find(&engine, 1, "missing", false, false, 0)
            .matches()
            .is_empty()
    );
    let clone = first.clone();
    engine.close("book".into()).unwrap();
    drop(first);
    drop(second);
    drop(exact);
    drop(words);
    drop(unicode);
    assert!(governor.usage().text > 0);
    assert_eq!(clone.matches().len(), 80);
    drop(clone);
    assert_eq!(governor.usage().text, 0);
    assert_eq!(Sha256::digest(fs::read(&fixture.0).unwrap()), before);
}

#[test]
fn crop_and_intrinsic_rotation_keep_the_same_native_match_center() {
    let engine = engine(MemoryGovernor::default());
    let mut centers = Vec::new();
    for rotation in [0, 90, 180, 270] {
        let fixture = fixture(rotation);
        let info = engine
            .open(
                "book".into(),
                fixture.0.clone(),
                None,
                Cancellation::default(),
            )
            .unwrap();
        let mut current_centers = Vec::new();
        let mut offset = 0;
        loop {
            let matches = find(&engine, 0, "needle", true, true, offset);
            for matched in matches.matches() {
                let rect = matched.rectangles[0];
                let center = [
                    (rect.min()[0] + rect.max()[0]) / 2.0,
                    (rect.min()[1] + rect.max()[1]) / 2.0,
                ];
                current_centers.push(info.pages[0].coordinates().pdf_point(center));
            }
            if let Some(next) = matches.next_match() {
                offset = next;
            } else {
                break;
            }
        }
        current_centers.sort_by(|a, b| a[1].total_cmp(&b[1]).then(a[0].total_cmp(&b[0])));
        centers.push(current_centers);
        engine.close("book".into()).unwrap();
    }
    for actual in &centers {
        assert_eq!(actual.len(), 101);
        for (center, expected) in actual.iter().zip(&centers[0]) {
            for axis in 0..2 {
                assert!(
                    (center[axis] - expected[axis]).abs() < 1e-4,
                    "native center {center:?} expected {expected:?}"
                );
            }
        }
    }
    let top = centers[0].last().unwrap();
    assert!((60.0..90.0).contains(&top[0]));
    assert!((760.0..770.0).contains(&top[1]));
}

#[test]
fn budget_cancellation_invalid_cursors_and_closed_documents_are_recoverable() {
    let fixture = fixture(0);
    let governor = MemoryGovernor::new(GlobalMemoryLimits {
        text: 1024,
        ..Default::default()
    });
    let engine = engine(governor.clone());
    engine
        .open(
            "book".into(),
            fixture.0.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let query = PdfFindQuery::new("needle", false, false).unwrap();
    assert!(matches!(
        engine
            .try_find_page("book".into(), 0, query.clone(), 0, Cancellation::default())
            .unwrap()
            .wait(),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(governor.usage().text, 0);
    governor.set_limits(Default::default());
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        engine
            .try_find_page("book".into(), 0, query.clone(), 0, cancelled)
            .unwrap()
            .wait(),
        Err(Failure::Cancelled)
    ));
    for (page, offset) in [(2, 0), (0, 999), (0, 10000)] {
        assert!(
            engine
                .try_find_page(
                    "book".into(),
                    page,
                    query.clone(),
                    offset,
                    Cancellation::default()
                )
                .unwrap()
                .wait()
                .is_err()
        );
        assert_eq!(governor.usage().text, 0);
    }
    assert_eq!(
        find(&engine, 0, "needle", false, false, 0).matches().len(),
        80
    );
    engine.close("book".into()).unwrap();
    assert!(
        engine
            .try_find_page("book".into(), 0, query, 0, Cancellation::default())
            .unwrap()
            .wait()
            .is_err()
    );
    assert_eq!(governor.usage().text, 0);
}
