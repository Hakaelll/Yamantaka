use super::*;

#[test]
fn superseding_an_open_cancels_only_its_owned_scopes_and_rejects_late_replies() {
    let mut state = ReaderOpening::default();
    let current_reader = Cancellation::default();
    let old = state.begin("a".into());
    let old_preferences = state.preferences().unwrap();
    let next = state.begin("b".into());
    let next_preferences = state.preferences().unwrap();
    assert!(old.is_cancelled());
    assert!(old_preferences.is_cancelled());
    assert!(!current_reader.is_cancelled());
    assert!(state.accept("a", &old).is_none());
    assert!(!state.fail("a", &old, false));
    assert!(state.accept("b", &Cancellation::default()).is_none());
    assert!(state.busy());
    let adopted = state.accept("b", &next).unwrap();
    assert!(adopted.same_request(&next_preferences));
    assert!(!adopted.is_cancelled());
    assert!(!next.is_cancelled());
    assert!(!state.busy());
    assert!(state.accept("b", &next).is_none());
}
#[test]
fn password_retries_keep_the_preference_scope_but_receive_new_native_identity() {
    let mut state = ReaderOpening::default();
    let first = state.begin("protected".into());
    let preferences = state.preferences().unwrap();
    assert!(state.fail("protected", &first, true));
    assert!(!state.busy());
    let retry = state.retry("protected".into());
    assert!(state.busy());
    assert!(first.is_cancelled());
    assert!(!retry.same_request(&first));
    assert!(!preferences.is_cancelled());
    assert!(state.accept("protected", &first).is_none());
    let adopted = state.accept("protected", &retry).unwrap();
    assert!(adopted.same_request(&preferences));
    assert!(!retry.is_cancelled());
}
#[test]
fn terminal_failure_and_explicit_cancellation_release_the_candidate_without_cancelling_adoption() {
    let mut state = ReaderOpening::default();
    let token = state.begin("a".into());
    let preferences = state.preferences().unwrap();
    assert!(state.fail("a", &token, false));
    assert!(token.is_cancelled());
    assert!(preferences.is_cancelled());
    assert!(!state.busy());
    let token = state.begin("b".into());
    token.cancel();
    assert!(state.accept("b", &token).is_none());
    assert!(state.fail("b", &token, true));
    assert!(!state.busy());
    assert!(state.preferences().is_none());
    let token = state.begin("c".into());
    let preferences = state.accept("c", &token).unwrap();
    drop(state);
    assert!(!token.is_cancelled());
    assert!(!preferences.is_cancelled());
}
