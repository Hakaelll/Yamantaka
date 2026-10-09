use super::*;
use yamantaka_domain::{annotations::marks::AnnotationMark, memory::*};
fn payload(governor: &MemoryGovernor, page: u32) -> AnnotationMarks {
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .unwrap();
    AnnotationMarks::from_reserved(page, Vec::<AnnotationMark>::new(), lease).unwrap()
}
fn reply(
    document: &str,
    page: u32,
    cancel: Cancellation,
    result: yamantaka_domain::Result<AnnotationMarks>,
) -> AnnotationMarksReply {
    AnnotationMarksReply {
        document: document.into(),
        page,
        cancel,
        result,
    }
}
#[test]
fn visible_pages_own_complete_snapshots_and_reject_replaced_or_cancelled_replies() {
    let governor = MemoryGovernor::default();
    let mut state = AnnotationPaint::default();
    state.retain("a", &[1, 2]);
    let cancel = Cancellation::default();
    state.pages.insert(1, Phase::Pending(cancel.clone()));
    state.accept(reply("old", 1, cancel.clone(), Ok(payload(&governor, 1))));
    state.accept(reply(
        "a",
        1,
        Cancellation::default(),
        Ok(payload(&governor, 1)),
    ));
    assert!(state.marks(1).is_none());
    state.accept(reply("a", 1, cancel.clone(), Ok(payload(&governor, 1))));
    assert!(state.ready());
    assert!(governor.usage().text > 0);
    state.retain("a", &[2]);
    assert_eq!(governor.usage().text, 0);
    let cancel = Cancellation::default();
    state.pages.insert(2, Phase::Pending(cancel.clone()));
    state.invalidate();
    assert!(cancel.is_cancelled());
    state.accept(reply("a", 2, cancel, Ok(payload(&governor, 2))));
    assert!(state.pages.is_empty());
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn failures_are_visible_and_invalidation_admits_a_fresh_request() {
    let governor = MemoryGovernor::default();
    let mut state = AnnotationPaint::default();
    state.retain("a", &[0]);
    let cancel = Cancellation::default();
    state.pages.insert(0, Phase::Pending(cancel.clone()));
    state.accept(reply("a", 0, cancel, Ok(payload(&governor, 1))));
    assert!(state.error().is_some());
    assert!(!state.ready());
    state.invalidate();
    let cancel = Cancellation::default();
    state.pages.insert(0, Phase::Pending(cancel.clone()));
    state.accept(reply("a", 0, cancel, Ok(payload(&governor, 0))));
    assert!(state.error().is_none());
    assert!(state.ready());
    state.retain("b", &[0]);
    assert_eq!(governor.usage().text, 0);
}
