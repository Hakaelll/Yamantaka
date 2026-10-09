use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use yamantaka_domain::{
    Cancellation, Failure, PdfEngine,
    annotations::{
        AnnotationColor, AnnotationDraft, AnnotationKind, AnnotationPlacement, AnnotationSource,
    },
    memory::{GlobalMemoryLimits, MemoryGovernor},
    pdf_ticket::AsyncPdfEngine,
    reader_preferences::PageRotation,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

fn fixture(rotation: u16, contents: &str) -> Fixture {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>".into(),
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /CropBox [20 30 580 780] /Rotate {rotation} /Resources << /Font << /F1 7 0 R >> >> /Contents 4 0 R >>"
        ),
        format!(
            "<< /Length {} >>\nstream\n{contents}\nendstream",
            contents.len()
        ),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /Contents 6 0 R >>".into(),
        "<< /Length 0 >>\nstream\n\nendstream".into(),
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
        "yamantaka-text-{}-{}.pdf",
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
fn native_unicode_character_ranges_crop_rotations_and_retained_memory() {
    for rotation in [0, 90, 180, 270] {
        let fixture = fixture(rotation, "BT /F1 12 Tf 60 700 Td (caf\\351 study) Tj ET");
        let before = Sha256::digest(fs::read(&fixture.0).unwrap());
        let governor = MemoryGovernor::default();
        let worker = engine(governor.clone());
        worker
            .open(
                "study".into(),
                fixture.0.clone(),
                None,
                Cancellation::default(),
            )
            .unwrap();
        let page = worker
            .try_text_geometry("study".into(), 0, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(page.page(), 0);
        assert_eq!(page.selected_text(0..4).unwrap(), "café");
        assert_eq!(page.selected_text(3..4).unwrap(), "é");
        assert_eq!(page.selected_text(5..10).unwrap(), "study");
        let region = page.characters()[0].region.unwrap();
        let center = [
            (region.min()[0] + region.max()[0]) / 2.0,
            (region.min()[1] + region.max()[1]) / 2.0,
        ];
        match rotation {
            0 => assert!(center[0] < 0.2 && center[1] < 0.2),
            90 => assert!(center[0] > 0.8 && center[1] < 0.2),
            180 => assert!(center[0] > 0.8 && center[1] > 0.8),
            270 => assert!(center[0] < 0.2 && center[1] > 0.8),
            _ => unreachable!(),
        }
        for character in page.characters() {
            if let Some(region) = character.region {
                assert!(
                    region
                        .min()
                        .into_iter()
                        .chain(region.max())
                        .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
                );
            }
        }
        let empty = worker
            .try_text_geometry("study".into(), 1, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap();
        assert!(empty.text().is_empty());
        assert!(empty.characters().is_empty());
        {
            use yamantaka_domain::text_selection::{
                GeometryCoverage, SelectionPosition, TextSelection,
            };
            let selection = TextSelection::new(
                SelectionPosition::new(1, 0).unwrap(),
                SelectionPosition::new(0, 1).unwrap(),
            );
            let copy = selection
                .copy_text(&[page.clone(), empty.clone()], &governor)
                .unwrap();
            assert_eq!(copy.text(), "afé study");
            assert_eq!(selection.page(&page).unwrap().unwrap().text(), "afé study");
            let selected = selection.page(&page).unwrap().unwrap();
            let fingerprint = format!("{before:x}");
            for kind in [
                AnnotationKind::Highlight,
                AnnotationKind::Underline,
                AnnotationKind::Strikeout,
                AnnotationKind::TextNote,
                AnnotationKind::PageMarker,
                AnnotationKind::PageComment,
            ] {
                let draft = AnnotationDraft::from_selection(
                    AnnotationSource {
                        document_id: "study",
                        fingerprint: &fingerprint,
                        rotation: PageRotation::Deg0,
                    },
                    &selected,
                    kind,
                    AnnotationColor::Violet,
                    Some("Native study note"),
                    &governor,
                )
                .unwrap();
                assert_eq!(draft.placement(), AnnotationPlacement::Attached);
                assert_eq!(draft.values().selected_text.as_deref(), Some("afé study"));
                assert_eq!(draft.values().context_before.as_deref(), Some("c"));
                assert!(draft.values().context_after.is_none());
                assert_eq!(draft.values().kind, kind);
                if kind == AnnotationKind::PageComment {
                    assert!(draft.values().rects.is_empty());
                } else {
                    assert_eq!(
                        draft
                            .values()
                            .rects
                            .iter()
                            .map(|rect| rect.region())
                            .collect::<Vec<_>>(),
                        selected.ink_regions().collect::<Vec<_>>()
                    );
                }
            }
            assert_eq!(
                selection.page(&page).unwrap().unwrap().geometry_coverage(),
                GeometryCoverage::Complete
            );
        }
        drop(empty);
        worker.close("study".into()).unwrap();
        assert!(governor.usage().text > 0);
        let retained = page.clone();
        drop(page);
        assert_eq!(retained.selected_text(0..4).unwrap(), "café");
        drop(retained);
        assert_eq!(governor.usage().text, 0);
        assert_eq!(before, Sha256::digest(fs::read(&fixture.0).unwrap()));
    }
}

#[test]
fn cancellation_invalid_page_and_budget_failure_release_all_text_work() {
    let fixture = fixture(0, "BT /F1 12 Tf 60 700 Td (study) Tj ET");
    let governor = MemoryGovernor::default();
    let worker = engine(governor.clone());
    worker
        .open(
            "study".into(),
            fixture.0.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        worker
            .try_text_geometry("study".into(), 0, cancel)
            .unwrap()
            .wait(),
        Err(Failure::Cancelled)
    ));
    assert!(
        worker
            .try_text_geometry("study".into(), 9, Cancellation::default())
            .unwrap()
            .wait()
            .is_err()
    );
    assert_eq!(governor.usage().text, 0);
    governor.set_limits(GlobalMemoryLimits {
        text: 1024,
        ..GlobalMemoryLimits::default()
    });
    assert!(matches!(
        worker
            .try_text_geometry("study".into(), 0, Cancellation::default())
            .unwrap()
            .wait(),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(governor.usage().text, 0);
    governor.set_limits(GlobalMemoryLimits::default());
    assert_eq!(
        worker
            .try_text_geometry("study".into(), 0, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap()
            .text(),
        "study"
    );
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn oversized_native_text_is_rejected_before_character_vector_allocation() {
    let contents = (0..5)
        .map(|line| {
            format!(
                "BT /F1 0.001 Tf 60 {} Td ({}) Tj ET\n",
                700 - line * 2,
                "a".repeat(26_000)
            )
        })
        .collect::<String>();
    let fixture = fixture(0, &contents);
    let governor = MemoryGovernor::default();
    let worker = engine(governor.clone());
    worker
        .open(
            "study".into(),
            fixture.0.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let result = worker
        .try_text_geometry("study".into(), 0, Cancellation::default())
        .unwrap()
        .wait();
    let diagnostic = result
        .as_ref()
        .map(|page| page.characters().len())
        .map_err(|error| error.to_string());
    assert!(
        matches!(result, Err(Failure::Pdf(message)) if message.contains("character work limit")),
        "{diagnostic:?}"
    );
    assert_eq!(governor.usage().text, 0);
    let empty = worker
        .try_text_geometry("study".into(), 1, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    assert!(empty.text().is_empty());
    drop(empty);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn cropped_out_characters_keep_text_without_inventing_selection_geometry() {
    let fixture = fixture(0, "BT /F1 12 Tf 5 400 Td (i) Tj ET");
    let governor = MemoryGovernor::default();
    let worker = engine(governor.clone());
    worker
        .open(
            "study".into(),
            fixture.0.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let page = worker
        .try_text_geometry("study".into(), 0, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(page.selected_text(0..1).unwrap(), "i");
    assert!(page.characters()[0].region.is_none());
    {
        use yamantaka_domain::text_selection::{
            GeometryCoverage, SelectionPosition, TextSelection,
        };
        let selection = TextSelection::new(
            SelectionPosition::new(0, 0).unwrap(),
            SelectionPosition::new(0, 1).unwrap(),
        );
        assert_eq!(
            selection.page(&page).unwrap().unwrap().geometry_coverage(),
            GeometryCoverage::Unavailable
        );
        let draft = AnnotationDraft::from_selection(
            AnnotationSource {
                document_id: "study",
                fingerprint: "cropped-fixture",
                rotation: PageRotation::Deg0,
            },
            &selection.page(&page).unwrap().unwrap(),
            AnnotationKind::Highlight,
            AnnotationColor::Yellow,
            None,
            &governor,
        )
        .unwrap();
        assert_eq!(
            draft.placement(),
            AnnotationPlacement::PageNoteFallback {
                requested: AnnotationKind::Highlight,
                coverage: GeometryCoverage::Unavailable
            }
        );
        assert!(draft.values().rects.is_empty());
        assert_eq!(draft.values().selected_text.as_deref(), Some("i"));
        assert_eq!(
            selection
                .copy_text(std::slice::from_ref(&page), &governor)
                .unwrap()
                .text(),
            "i"
        );
    }
    drop(page);
    assert_eq!(governor.usage().text, 0);
}
