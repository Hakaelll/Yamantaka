use super::*;

fn tab(id: &str, document: &str, page: u32) -> ReaderTab {
    let mut preferences = ReaderPreferences::new(document);
    preferences.zoom_mode = ZoomMode::FitWidth;
    preferences.zoom_level = 1.75;
    preferences.page_rotation = yamantaka_domain::reader_preferences::PageRotation::Deg90;
    preferences.horizontal_margin = 31.0;
    preferences.scroll_offset = 1500.0;
    ReaderTab::new(
        id.into(),
        document.into(),
        page,
        preferences,
        Some(SessionPanel::Text),
    )
    .unwrap()
}
#[test]
fn switch_waits_for_lookup_and_native_identity_then_restores_complete_context_once() {
    let mut state = TabState::default();
    state.model.open(tab("a", "first", 0), false).unwrap();
    state.model.open(tab("b", "second", 3), false).unwrap();
    state.model.activate("a");
    let mut plan = state.model.clone();
    plan.activate("b");
    let lookup = Cancellation::default();
    state.begin(plan, lookup.clone());
    assert_eq!(state.model.active().unwrap().id(), "a");
    assert!(
        state
            .lookup_target("second", &Cancellation::default())
            .is_none()
    );
    assert!(state.lookup_target("first", &lookup).is_none());
    assert_eq!(state.lookup_target("second", &lookup).unwrap().page(), 3);
    let opening = Cancellation::default();
    state.bind_open(opening.clone());
    assert!(state.lookup_target("second", &lookup).is_none());
    assert!(!state.accept_open("second", &Cancellation::default(), Cancellation::default()));
    let preferences = Cancellation::default();
    assert!(state.accept_open("second", &opening, preferences.clone()));
    assert_eq!(state.model.active().unwrap().id(), "b");
    assert!(
        state
            .take_restoration("second", &Cancellation::default())
            .is_none()
    );
    let restored = state.take_restoration("second", &preferences).unwrap();
    assert_eq!(restored.preferences().zoom_mode, ZoomMode::FitWidth);
    assert_eq!(
        restored.preferences().page_rotation,
        yamantaka_domain::reader_preferences::PageRotation::Deg90
    );
    assert_eq!(restored.preferences().horizontal_margin, 31.0);
    assert_eq!(restored.panel(), Some(SessionPanel::Text));
    assert!(state.take_restoration("second", &preferences).is_none());
    drop(state);
    assert!(!opening.is_cancelled());
    assert!(!preferences.is_cancelled());
}
#[test]
fn failed_and_superseded_switches_preserve_model_and_cancel_only_candidate() {
    let mut state = TabState::default();
    state.model.open(tab("a", "first", 0), false).unwrap();
    let mut plan = state.model.clone();
    plan.open(tab("b", "second", 1), false).unwrap();
    let lookup = Cancellation::default();
    state.begin(plan.clone(), lookup.clone());
    let opening = Cancellation::default();
    state.bind_open(opening.clone());
    state.fail("second", &Cancellation::default());
    assert!(!state.ready());
    state.fail("second", &opening);
    assert!(state.ready());
    assert!(lookup.is_cancelled());
    assert!(opening.is_cancelled());
    assert_eq!(state.model.tabs().len(), 1);
    assert_eq!(state.model.active().unwrap().id(), "a");
    let lookup = Cancellation::default();
    state.begin(plan.clone(), lookup.clone());
    let opening = Cancellation::default();
    state.bind_open(opening.clone());
    state.begin(plan, Cancellation::default());
    assert!(opening.is_cancelled());
    assert!(lookup.is_cancelled());
    assert!(!state.accept_open("second", &opening, Cancellation::default()));
}
#[test]
fn adoption_keeps_outgoing_scroll_changes_and_duplicate_reading_positions_independent() {
    let mut state = TabState::default();
    state.model.open(tab("a", "first", 0), false).unwrap();
    let mut plan = state.model.clone();
    plan.open(tab("b", "first", 5), true).unwrap();
    state.begin(plan, Cancellation::default());
    let mut preferences = state.model.active().unwrap().preferences().clone();
    preferences.scroll_offset = 2400.0;
    state.model.update_active(2, preferences, None).unwrap();
    state.adopt(Cancellation::default());
    assert_eq!(state.model.active().unwrap().id(), "b");
    let old = state.model.tabs().iter().find(|t| t.id() == "a").unwrap();
    assert_eq!(old.page(), 2);
    assert_eq!(old.preferences().scroll_offset, 2400.0);
    assert_eq!(state.model.active().unwrap().page(), 5);
}

#[test]
fn leaving_the_reader_revokes_lookup_but_retains_an_admitted_native_handoff() {
    let mut state = TabState::default();
    state.model.open(tab("a", "first", 0), false).unwrap();
    let mut plan = state.model.clone();
    plan.open(tab("b", "second", 1), false).unwrap();
    let lookup = Cancellation::default();
    state.begin(plan.clone(), lookup.clone());
    state.leave_destination();
    assert!(lookup.is_cancelled());
    assert!(state.ready());
    state.begin(plan, Cancellation::default());
    let opening = Cancellation::default();
    state.bind_open(opening.clone());
    state.leave_destination();
    assert!(!opening.is_cancelled());
    assert!(state.accept_open("second", &opening, Cancellation::default()));
    assert_eq!(state.model.active().unwrap().id(), "b");
}
