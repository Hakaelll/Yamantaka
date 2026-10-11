use super::*;
use crate::reader_preferences::{PageRotation, ReaderTheme, ZoomMode};

fn tab(index: usize) -> ReaderTab {
    let document = format!("document-{index}");
    ReaderTab::new(
        format!("tab-{index}"),
        document.clone(),
        index as u32,
        ReaderPreferences::new(&document),
        None,
    )
    .unwrap()
}

#[test]
fn continuity_restore_is_atomic_and_successful_identity_reuse_removes_stale_closure() {
    let mut model = DocumentTabs::default();
    model.open(tab(0), true).unwrap();
    model.close("tab-0");
    model.open(tab(1), true).unwrap();
    for closed in [
        vec![tab(2), tab(2)],
        vec![tab(1)],
        (0..=MAX_CLOSED_TABS).map(tab).collect(),
    ] {
        assert!(
            model
                .restore_continuity(vec![tab(1)], Some("tab-1"), closed)
                .is_err()
        );
        assert_eq!(model.active().unwrap().id(), "tab-1");
        assert_eq!(model.closed()[0].id(), "tab-0");
    }
    assert!(
        model
            .restore_continuity(vec![tab(2)], Some("missing"), vec![])
            .is_err()
    );
    model.open(tab(0), true).unwrap();
    assert!(model.closed().is_empty());
    model.close("tab-0");
    model.restore(vec![tab(0)], None).unwrap();
    assert!(model.closed().is_empty());
}
#[test]
fn opening_reuses_a_documents_snapshot_and_force_new_keeps_independent_positions() {
    let mut tabs = DocumentTabs::default();
    tabs.open(tab(0), false).unwrap();
    let mut preferences = ReaderPreferences::new("document-0");
    preferences.zoom_mode = ZoomMode::Custom;
    preferences.zoom_level = 1.75;
    preferences.scroll_offset = 1234.5;
    preferences.page_rotation = PageRotation::Deg90;
    preferences.reading_theme = ReaderTheme::Sepia;
    tabs.update_active(8, preferences.clone(), Some(SessionPanel::Annotations))
        .unwrap();
    let second = ReaderTab::new(
        "another-tab".into(),
        "document-0".into(),
        1,
        ReaderPreferences::new("document-0"),
        None,
    )
    .unwrap();
    let reused = tabs.open(second.clone(), false).unwrap();
    assert!(reused.reused);
    assert_eq!(reused.selected, "tab-0");
    assert_eq!(tabs.tabs.len(), 1);
    assert_eq!(tabs.active().unwrap().preferences(), &preferences);
    assert_eq!(tabs.active().unwrap().page(), 8);
    tabs.open(second, true).unwrap();
    assert_eq!(tabs.tabs.len(), 2);
    assert_eq!(tabs.active().unwrap().page(), 1);
    assert_eq!(tabs.hot_documents(), ["document-0"]);
    tabs.activate("tab-0");
    assert_eq!(
        tabs.active().unwrap().panel(),
        Some(SessionPanel::Annotations)
    );
    assert_eq!(tabs.active().unwrap().preferences(), &preferences);
}
#[test]
fn twelve_tab_limit_evicts_the_last_unpinned_inactive_tab_and_preserves_it_for_reopening() {
    let mut tabs = DocumentTabs::default();
    for index in 0..MAX_TABS {
        tabs.open(tab(index), false).unwrap();
    }
    assert!(tabs.toggle_pinned("tab-10"));
    let opened = tabs.open(tab(12), false).unwrap();
    assert_eq!(opened.evicted.as_ref().unwrap().id(), "tab-9");
    assert_eq!(tabs.tabs.len(), MAX_TABS);
    assert_eq!(tabs.active().unwrap().id(), "tab-12");
    assert_eq!(tabs.closed.last().unwrap().id(), "tab-9");
    let before: Vec<_> = tabs.tabs.iter().map(|tab| tab.id.clone()).collect();
    assert!(tabs.reopen_last_closed().is_err());
    assert_eq!(tabs.closed.len(), 1);
    assert_eq!(
        tabs.tabs
            .iter()
            .map(|tab| tab.id.clone())
            .collect::<Vec<_>>(),
        before
    );
    assert!(tabs.close("tab-12").is_some());
    assert_eq!(
        tabs.reopen_last_closed().unwrap().as_deref(),
        Some("tab-12")
    );
    assert_eq!(tabs.closed.len(), 1);
}
#[test]
fn pinned_limits_and_full_pinned_workspaces_do_not_mutate_on_failure() {
    let mut tabs = DocumentTabs::default();
    for index in 0..5 {
        tabs.open(tab(index), false).unwrap();
    }
    for index in 0..MAX_PINNED_TABS {
        assert!(tabs.toggle_pinned(&format!("tab-{index}")));
    }
    assert!(tabs.open(tab(10).with_pinned(true), false).is_err());
    assert_eq!(tabs.tabs.len(), 5);
    assert!(!tabs.toggle_pinned("tab-3"));
    assert!(!tabs.tabs[3].pinned());
    assert!(tabs.toggle_pinned("tab-0"));
    assert!(tabs.toggle_pinned("tab-3"));
    tabs.restore(
        (0..MAX_TABS)
            .map(|index| tab(index).with_pinned(true))
            .collect(),
        Some("tab-0"),
    )
    .unwrap();
    assert!(tabs.open(tab(12), false).is_err());
    assert_eq!(tabs.tabs.len(), MAX_TABS);
    assert!(tabs.closed.is_empty());
    assert_eq!(tabs.active().unwrap().id(), "tab-0");
}
#[test]
fn adjacent_activation_bulk_closing_and_ten_closure_limit_follow_tab_order() {
    let mut tabs = DocumentTabs::default();
    for index in 0..6 {
        tabs.open(tab(index), false).unwrap();
    }
    tabs.toggle_pinned("tab-4");
    tabs.activate("tab-2");
    let closed = tabs.close("tab-2").unwrap();
    assert_eq!(closed.page(), 2);
    assert_eq!(tabs.active().unwrap().id(), "tab-3");
    assert!(tabs.close_to_right("tab-1"));
    assert_eq!(tabs.active().unwrap().id(), "tab-1");
    assert_eq!(
        tabs.tabs.iter().map(ReaderTab::id).collect::<Vec<_>>(),
        ["tab-0", "tab-1", "tab-4"]
    );
    assert!(tabs.close_others("tab-0"));
    assert_eq!(
        tabs.tabs.iter().map(ReaderTab::id).collect::<Vec<_>>(),
        ["tab-0", "tab-4"]
    );
    for index in 20..40 {
        tabs.open(tab(index), false).unwrap();
        tabs.close(&format!("tab-{index}"));
    }
    assert_eq!(tabs.closed.len(), MAX_CLOSED_TABS);
    assert_eq!(tabs.closed.first().unwrap().id(), "tab-30");
    assert_eq!(
        tabs.reopen_last_closed().unwrap().as_deref(),
        Some("tab-39")
    );
}
#[test]
fn order_cycling_and_duplicate_document_tabs_keep_at_most_two_distinct_documents_hot() {
    let mut tabs = DocumentTabs::default();
    for index in 0..6 {
        tabs.open(tab(index), false).unwrap();
    }
    tabs.activate("tab-0");
    assert_eq!(tabs.hot_documents(), ["document-0", "document-5"]);
    assert_eq!(tabs.relative(true).unwrap().id(), "tab-5");
    assert_eq!(tabs.relative(false).unwrap().id(), "tab-1");
    assert!(tabs.reorder(5, 1));
    assert_eq!(tabs.relative(false).unwrap().id(), "tab-5");
    let duplicate = ReaderTab::new(
        "duplicate".into(),
        "document-0".into(),
        20,
        ReaderPreferences::new("document-0"),
        None,
    )
    .unwrap();
    tabs.open(duplicate, true).unwrap();
    assert_eq!(tabs.hot_documents().len(), MAX_HOT_DOCUMENTS);
    assert_eq!(
        tabs.tabs
            .iter()
            .filter(|tab| tab.resource == TabResourceState::Active)
            .count(),
        1
    );
    assert!(
        tabs.tabs
            .iter()
            .any(|tab| tab.resource == TabResourceState::Suspended)
    );
    let before = tabs
        .hot_documents()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert!(!tabs.reorder(99, 0));
    assert!(!tabs.activate("missing"));
    assert_eq!(
        tabs.hot_documents(),
        before.iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert!(tabs.reorder(0, usize::MAX));
    assert_eq!(tabs.tabs.last().unwrap().id(), "tab-0");
}
#[test]
fn invalid_restoration_and_snapshot_updates_leave_existing_state_intact() {
    let mut tabs = DocumentTabs::default();
    tabs.open(tab(0), false).unwrap();
    tabs.open(tab(1), false).unwrap();
    tabs.close("tab-1");
    let before = tabs.active().unwrap().clone();
    assert!(tabs.open(tab(0), true).is_err());
    assert!(!tabs.close_others("missing"));
    assert!(!tabs.close_to_right("missing"));
    assert!(tabs.restore(vec![tab(2), tab(2)], Some("tab-2")).is_err());
    assert!(
        tabs.restore((0..=MAX_TABS).map(tab).collect(), None)
            .is_err()
    );
    assert!(
        tabs.update_active(9, ReaderPreferences::new("another-document"), None)
            .is_err()
    );
    assert_eq!(tabs.active(), Some(&before));
    assert_eq!(tabs.closed.len(), 1);
    let mut invalid = ReaderPreferences::new("document-0");
    invalid.zoom_level = f64::MAX;
    assert!(tabs.update_active(9, invalid, None).is_err());
    assert_eq!(tabs.active(), Some(&before));
    tabs.restore(vec![tab(3), tab(4)], Some("missing")).unwrap();
    assert_eq!(tabs.active().unwrap().id(), "tab-3");
    tabs.restore(vec![], None).unwrap();
    assert!(tabs.active().is_none());
    assert!(tabs.relative(false).is_none());
    assert!(tabs.hot_documents().is_empty());
}

#[test]
fn refreshing_a_closed_snapshot_is_validated_and_keeps_identity_and_pinning() {
    let mut tabs = DocumentTabs::default();
    tabs.open(tab(0).with_pinned(true), false).unwrap();
    tabs.close("tab-0");
    let mut preferences = ReaderPreferences::new("document-0");
    preferences.zoom_level = 1.75;
    preferences.scroll_offset = 1500.0;
    assert!(
        tabs.refresh_closed("tab-0", 3, preferences.clone(), Some(SessionPanel::Text))
            .unwrap()
    );
    let saved = tabs.closed()[0].clone();
    assert!(saved.pinned());
    assert_eq!(saved.page(), 3);
    assert_eq!(saved.preferences(), &preferences);
    assert!(
        tabs.refresh_closed("missing", 4, preferences.clone(), None)
            .is_ok_and(|changed| !changed)
    );
    preferences.document_id = "wrong".into();
    assert!(tabs.refresh_closed("tab-0", 4, preferences, None).is_err());
    assert_eq!(tabs.closed()[0], saved);
    tabs.reopen_last_closed().unwrap();
    assert_eq!(tabs.active().unwrap().page(), 3);
    assert_eq!(tabs.active().unwrap().panel(), Some(SessionPanel::Text));
}
