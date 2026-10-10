use super::*;
fn reply(
    request: &ReadingGoalRequest,
    result: yamantaka_domain::Result<Option<ReadingGoal>>,
) -> ReadingGoalReply {
    ReadingGoalReply {
        document: request.document.clone(),
        change: request.change,
        cancel: request.cancel.clone(),
        result,
    }
}
#[test]
fn stale_reads_and_wrong_write_tokens_cannot_change_the_selected_book() {
    let mut state = GoalState::default();
    let first = state.select("a").unwrap();
    let second = state.select("b").unwrap();
    assert!(first.cancel.is_cancelled());
    let goal = ReadingGoal::new(Some(12), Some(25)).unwrap();
    state.accept(reply(&first, Ok(Some(goal))));
    assert!(!state.loaded);
    let mut unrelated = reply(&second, Ok(Some(goal)));
    unrelated.cancel = Cancellation::default();
    state.accept(unrelated);
    assert!(!state.loaded);
    state.accept(reply(&second, Ok(None)));
    assert!(state.loaded);
    let write = state.save(Some(goal)).unwrap();
    assert!(state.save(None).is_none());
    let mut unrelated = reply(&write, Ok(Some(goal)));
    unrelated.cancel = Cancellation::default();
    state.accept(unrelated);
    assert!(state.writing.is_some());
    assert_eq!(state.goal, None);
    state.accept(reply(&write, Ok(Some(goal))));
    assert_eq!(state.goal, Some(goal));
    assert!(state.writing.is_none());
}
#[test]
fn accepted_write_survives_selection_change_and_failures_keep_the_draft() {
    let mut state = GoalState::default();
    let read = state.select("a").unwrap();
    state.accept(reply(&read, Ok(None)));
    state.edit();
    state.pages = "12".into();
    state.minutes = "25".into();
    let goal = state.draft().unwrap();
    let write = state.save(goal).unwrap();
    state.accept(reply(
        &write,
        Err(yamantaka_domain::Failure::Invalid("failed".into())),
    ));
    assert!(state.editing);
    assert_eq!(state.pages, "12");
    assert!(state.error.is_some());
    assert_eq!(state.goal, None);
    let retry = state.save(goal).unwrap();
    assert!(!retry.cancel.same_request(&write.cancel));
    let read = state.select("b").unwrap();
    state.accept(reply(&read, Ok(None)));
    assert!(!retry.cancel.is_cancelled());
    state.accept(reply(&retry, Ok(goal)));
    assert_eq!(state.document.as_deref(), Some("b"));
    assert_eq!(state.goal, None);
    assert!(state.writing.is_none());
}
#[test]
fn optional_blank_targets_clear_and_invalid_drafts_cannot_be_saved() {
    let mut state = GoalState::default();
    assert_eq!(state.draft().unwrap(), None);
    for (pages, minutes) in [
        ("0", ""),
        ("10001", ""),
        ("", "1441"),
        ("1.5", ""),
        ("-1", ""),
        ("+2", ""),
        ("letters", ""),
    ] {
        state.pages = pages.into();
        state.minutes = minutes.into();
        assert!(state.draft().is_err());
    }
    state.pages = "10000".into();
    state.minutes = "1440".into();
    assert_eq!(
        state.draft().unwrap(),
        Some(ReadingGoal::new(Some(10000), Some(1440)).unwrap())
    );
}
