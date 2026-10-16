use std::{fs, path::PathBuf};
use yamantaka_domain::{
    Cancellation, PdfEngine,
    front_matter::{EnrichmentEvidence, EvidenceKind, FrontMatter, MetadataDecision},
    import_metadata::MetadataValue,
    memory::MemoryGovernor,
    pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

fn analyze(name: &str) -> (EnrichmentEvidence, MemoryGovernor) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join(format!("build/rust-fixtures/{name}.pdf"));
    let before = fs::read(&source).unwrap();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    let cancel = Cancellation::default();
    let info = worker
        .open("source".into(), source.clone(), None, cancel.clone())
        .unwrap();
    let mut scan = FrontMatter::new(
        "synthetic.pdf",
        info.title.as_deref(),
        info.author.as_deref(),
        &governor,
    )
    .unwrap();
    for page in 0..info.pages.len().min(6) as u32 {
        let text = worker
            .try_text_geometry(
                "source".into(),
                page,
                yamantaka_domain::pdf_text::TextGeometryPurpose::FrontMatter,
                cancel.clone(),
            )
            .unwrap()
            .wait()
            .unwrap();
        scan.observe(&text, info.pages[page as usize].coordinates(), &cancel)
            .unwrap();
    }
    let result = scan.finish(&cancel).unwrap();
    drop(info);
    drop(worker);
    assert_eq!(fs::read(source).unwrap(), before);
    assert_eq!(governor.usage().total, 16 * 1024);
    (result, governor)
}

#[test]
fn repeated_prominent_title_is_proposed_and_an_explicit_byline_can_fill_missing_author() {
    let (result, governor) = analyze("front-matter-missing");
    let MetadataDecision::Review {
        candidate: title, ..
    } = &result.title
    else {
        panic!("Expected repeated title evidence")
    };
    assert_eq!(title.value, "Geometry of Light");
    assert_eq!(title.page, 0);
    assert_eq!(title.evidence, EvidenceKind::RepeatedTitleWithByline);
    assert!(title.bounds.min()[1] < 0.3 && title.bounds.max()[1] > title.bounds.min()[1]);
    let MetadataDecision::Autofill(author) = &result.author else {
        panic!("Expected explicit byline")
    };
    assert_eq!(author.value, "Ada Lovelace");
    assert_eq!(author.evidence, EvidenceKind::ExplicitByline);
    assert!(result.has_text);
    drop(result);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn conflicting_plausible_metadata_is_proposed_for_review_instead_of_overwritten() {
    let (result, governor) = analyze("front-matter-conflict");
    let MetadataDecision::Review { current, candidate } = &result.title else {
        panic!("Expected title review")
    };
    assert_eq!(
        current,
        &MetadataValue::Accepted("Old project title".into())
    );
    assert_eq!(candidate.value, "Geometry of Light");
    let MetadataDecision::Review { current, candidate } = &result.author else {
        panic!("Expected author review")
    };
    assert_eq!(current, &MetadataValue::Accepted("Old Person".into()));
    assert_eq!(candidate.value, "Ada Lovelace");
    drop(result);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn matching_metadata_beats_a_larger_publisher_logo_and_single_visual_titles_need_review() {
    let (result, governor) = analyze("front-matter-logo");
    assert!(
        matches!(&result.title, MetadataDecision::Corroborated {value,page:0} if value == "Geometry of Light")
    );
    assert!(
        matches!(&result.author, MetadataDecision::Corroborated {value,page:0} if value == "Ada Lovelace")
    );
    drop(result);
    assert_eq!(governor.usage().total, 0);
    let (result, governor) = analyze("front-matter-single");
    assert!(
        matches!(&result.title, MetadataDecision::Review {candidate,..} if candidate.value == "Geometry of Light")
    );
    assert!(
        matches!(&result.author, MetadataDecision::Autofill(candidate) if candidate.value == "Ada Lovelace")
    );
    drop(result);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn dense_body_pages_and_image_only_pages_do_not_create_bibliographic_guesses() {
    let (result, governor) = analyze("front-matter-body");
    assert!(
        matches!(&result.title, MetadataDecision::Unchanged(MetadataValue::Accepted(value)) if value == "Contents and Context")
    );
    assert!(matches!(&result.author, MetadataDecision::Unchanged(_)));
    assert!(result.has_text);
    drop(result);
    assert_eq!(governor.usage().total, 0);
    let (result, governor) = analyze("scanned");
    assert!(!result.has_text);
    assert!(matches!(&result.title, MetadataDecision::Unchanged(_)));
    assert!(matches!(&result.author, MetadataDecision::Unchanged(_)));
    drop(result);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn multiple_explicit_authors_require_review_instead_of_silently_choosing_the_first() {
    for name in ["front-matter-ambiguous", "front-matter-ambiguous-metadata"] {
        let (result, governor) = analyze(name);
        assert!(
            matches!(&result.author, MetadataDecision::Review {candidate,..} if candidate.evidence == EvidenceKind::ConflictingBylines)
        );
        drop(result);
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn intrinsic_page_rotation_does_not_change_textual_title_and_author_decisions() {
    for rotation in [90, 180, 270] {
        let (result, governor) = analyze(&format!("front-matter-rotated-{rotation}"));
        assert!(
            matches!(&result.title, MetadataDecision::Review {candidate,..} if candidate.value == "Geometry of Light"),
            "Title changed at {rotation} degrees: {:?}",
            result.title
        );
        assert!(
            matches!(&result.author, MetadataDecision::Autofill(candidate) if candidate.value == "Ada Lovelace"),
            "Author changed at {rotation} degrees: {:?}",
            result.author
        );
        drop(result);
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn a_repeated_large_logo_does_not_replace_the_nearby_title_when_metadata_is_missing() {
    let (result, governor) = analyze("front-matter-logo-missing");
    assert!(
        matches!(&result.title, MetadataDecision::Review {candidate,..} if candidate.value == "Geometry of Light")
    );
    drop(result);
    assert_eq!(governor.usage().total, 0);
}
