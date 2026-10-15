use super::*;
use crate::{memory::GlobalMemoryLimits, pdf_text::TextCharacter};

fn coordinates() -> PageCoordinateSpace {
    PageCoordinateSpace::new([0.0, 1.0], [1.0, 1.0], [0.0, 0.0]).unwrap()
}

fn text(page: u32, text: &str, governor: &MemoryGovernor) -> PdfPageText {
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .unwrap();
    let characters = text
        .char_indices()
        .map(|(index, character)| TextCharacter {
            bytes: index as u32..(index + character.len_utf8()) as u32,
            region: None,
        })
        .collect();
    PdfPageText::from_reserved(page, text.into(), characters, lease).unwrap()
}

#[test]
fn missing_geometry_cannot_create_candidates_or_invent_bibliographic_names() {
    let governor = MemoryGovernor::default();
    let page = text(0, "Geometry of Light\nBy Ada Lovelace", &governor);
    let mut scan = FrontMatter::new("book.pdf", None, None, &governor).unwrap();
    scan.observe(&page, coordinates(), &Cancellation::default())
        .unwrap();
    let evidence = scan.finish(&Cancellation::default()).unwrap();
    assert!(evidence.has_text);
    assert!(matches!(
        evidence.title,
        MetadataDecision::Unchanged(MetadataValue::Missing)
    ));
    assert!(matches!(
        evidence.author,
        MetadataDecision::Unchanged(MetadataValue::Missing)
    ));
    drop(evidence);
    drop(page);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn page_limits_uniqueness_and_cancellation_are_checked_before_consuming_evidence() {
    let governor = MemoryGovernor::default();
    let page = text(0, "Synthetic", &governor);
    let outside = text(6, "Outside the work bound", &governor);
    let mut scan = FrontMatter::new("book.pdf", None, None, &governor).unwrap();
    let stopped = Cancellation::default();
    stopped.cancel();
    assert!(matches!(
        scan.observe(&page, coordinates(), &stopped),
        Err(Failure::Cancelled)
    ));
    let cancel = Cancellation::default();
    scan.observe(&page, coordinates(), &cancel).unwrap();
    assert!(matches!(
        scan.observe(&page, coordinates(), &cancel),
        Err(Failure::Invalid(_))
    ));
    assert!(matches!(
        scan.observe(&outside, coordinates(), &cancel),
        Err(Failure::Invalid(_))
    ));
    assert!(matches!(scan.finish(&stopped), Err(Failure::Cancelled)));
    drop(page);
    drop(outside);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn memory_pressure_leaves_an_observation_unconsumed_and_releases_retained_evidence() {
    let text_governor = MemoryGovernor::default();
    let page = text(0, "Not yet observed", &text_governor);
    let governor = MemoryGovernor::new(GlobalMemoryLimits {
        text: 32 * 1024,
        ..Default::default()
    });
    let mut scan = FrontMatter::new("book.pdf", None, None, &governor).unwrap();
    assert!(matches!(
        scan.observe(&page, coordinates(), &Cancellation::default()),
        Err(Failure::MemoryBudget)
    ));
    let evidence = scan.finish(&Cancellation::default()).unwrap();
    assert!(!evidence.has_text);
    assert_eq!(governor.usage().text, 16 * 1024);
    drop(evidence);
    assert_eq!(governor.usage().total, 0);
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        text: 1024,
        ..Default::default()
    });
    assert!(matches!(
        FrontMatter::new("book.pdf", None, None, &denied),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(denied.usage().total, 0);
}

#[test]
fn corroboration_preserves_word_boundaries_and_does_not_merge_different_titles() {
    assert!(matches_value(
        &MetadataValue::Accepted("Geometry: of Light".into()),
        "GEOMETRY of Light"
    ));
    assert!(!matches_value(
        &MetadataValue::Accepted("Now here".into()),
        "Nowhere"
    ));
    assert!(!matches_value(&MetadataValue::Accepted("A B".into()), "AB"));
}
