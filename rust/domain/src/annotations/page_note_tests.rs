use super::*;
#[test]
fn page_note_retains_unicode_source_and_no_fabricated_text_or_geometry() {
    let governor = MemoryGovernor::default();
    let source = AnnotationSource {
        document_id: "scanned",
        fingerprint: "original-sha",
        rotation: PageRotation::Deg270,
    };
    let draft = AnnotationDraft::page_note(
        source,
        7,
        AnnotationColor::Blue,
        Some("  Étude 学习  "),
        &governor,
    )
    .unwrap();
    let values = draft.values();
    assert_eq!(values.page_index, 7);
    assert_eq!(values.kind, AnnotationKind::PageComment);
    assert_eq!(values.page_rotation, 270);
    assert_eq!(values.document_fingerprint, "original-sha");
    assert_eq!(values.note.as_deref(), Some("Étude 学习"));
    assert!(
        values.selected_text.is_none()
            && values.context_before.is_none()
            && values.context_after.is_none()
            && values.rects.is_empty()
    );
    assert_eq!(draft.placement(), AnnotationPlacement::Attached);
    drop(draft);
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn page_note_uses_shared_identity_utf16_and_memory_admission_limits() {
    let governor = MemoryGovernor::default();
    let source = || AnnotationSource {
        document_id: "a",
        fingerprint: "fa",
        rotation: PageRotation::Deg0,
    };
    assert!(
        AnnotationDraft::page_note(
            source(),
            0,
            AnnotationColor::Yellow,
            Some(&"𐐷".repeat(2001)),
            &governor
        )
        .is_err()
    );
    let empty =
        AnnotationDraft::page_note(source(), 0, AnnotationColor::Yellow, Some("  "), &governor)
            .unwrap();
    assert!(empty.values().note.is_none());
    drop(empty);
    assert!(
        AnnotationDraft::page_note(
            AnnotationSource {
                document_id: "",
                ..source()
            },
            0,
            AnnotationColor::Yellow,
            None,
            &governor
        )
        .is_err()
    );
    let denied = MemoryGovernor::new(crate::memory::GlobalMemoryLimits {
        text: 1,
        ..Default::default()
    });
    assert!(matches!(
        AnnotationDraft::page_note(source(), 0, AnnotationColor::Yellow, Some("Study"), &denied),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(denied.usage().text, 0);
    assert_eq!(governor.usage().text, 0);
}
