use yamantaka_domain::{Cancellation, LibraryRepository, workspaces::*};
use yamantaka_storage::Database;

pub fn preserved_split_references(document: &str) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("workspaces.sqlite3");
    let mut database = Database::open(&path).unwrap();
    database.execute_batch(document).unwrap();
    database.execute_batch(r#"
        INSERT INTO workspaces(id,name,active_tab_id,panel_width,split_json,query_json,grouping,sort_mode,density,visible_columns_json,focus_mode,created_at,updated_at)
        VALUES('source','Imported study','old-tab',360,'{"enabled":true,"secondaryTabId":"old-tab","axis":"horizontal","ratio":0.6,"custom":42}','{"custom":"retained"}','author','title','compact','["document","author"]',1,'2026-10-01','2026-10-01');
        INSERT INTO workspace_tabs(id,workspace_id,document_id,sort_order,queue_status,next_action)
        VALUES('old-tab','source','original',0,'blocked','Read the appendix');
    "#).unwrap();
    let cancel = Cancellation::default();
    let source = database.workspaces(0, &cancel).unwrap().entries.remove(0);
    let legacy = database.workspace(&(&source).into(), &cancel).unwrap();
    let comparison = legacy.view.parsed_query().unwrap().comparison.unwrap();
    assert_eq!(comparison.layout, ComparisonLayout::Stacked);
    assert_eq!(comparison.document, "original");
    assert_eq!(comparison.ratio, 0.6);
    assert_eq!(comparison.scroll_offset, Some(0.0));
    assert!(comparison.bidirectional);
    assert!(legacy.comparison_warning.is_none());
    database.execute_batch("UPDATE workspaces SET split_json=json_set(split_json,'$.comparisonMode',json('true')) WHERE id='source'").unwrap();
    let unsupported = database.workspace(&(&source).into(), &cancel).unwrap();
    assert_eq!(unsupported.tabs.tabs().len(), 1);
    assert!(
        unsupported
            .view
            .parsed_query()
            .unwrap()
            .comparison
            .is_none()
    );
    assert!(unsupported.comparison_warning.is_some());
    database.execute_batch("UPDATE workspaces SET split_json=json_remove(split_json,'$.comparisonMode') WHERE id='source'").unwrap();
    let copied = database
        .change_workspace(
            &WorkspaceChange::Duplicate {
                expected: (&source).into(),
                name: "Study copy".into(),
            },
            &cancel,
        )
        .unwrap();
    let WorkspaceChanged::Saved(copied) = copied else {
        panic!("Duplicate must return a saved workspace");
    };
    let copy = copied.id;
    let entries = database.workspaces(0, &cancel).unwrap().entries;
    let copied = entries.iter().find(|item| item.id == copy).unwrap();
    let readings = database.workspace(&copied.into(), &cancel).unwrap();
    assert_eq!(
        readings.tabs.tabs()[0].queue().next_action(),
        Some("Read the appendix")
    );
    let probe = rusqlite::Connection::open(&path).unwrap();
    let check = |id: &str| {
        let (split,query,grouping,focus,width):(String,String,String,bool,f64)=probe.query_row("SELECT split_json,query_json,grouping,focus_mode,panel_width FROM workspaces WHERE id=?",[id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?))).unwrap();
        let split: serde_json::Value = serde_json::from_str(&split).unwrap();
        let tab: String = probe
            .query_row(
                "SELECT id FROM workspace_tabs WHERE workspace_id=?",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(split["secondaryTabId"], tab);
        assert_eq!(split["enabled"], true);
        assert_eq!(split["axis"], "horizontal");
        assert_eq!(split["custom"], 42);
        assert_eq!(query, r#"{"custom":"retained"}"#);
        assert_eq!(grouping, "author");
        assert!(focus);
        assert_eq!(width, 360.0);
        tab
    };
    let before = check(&copy);
    assert_ne!(before, "old-tab");
    database
        .change_workspace(
            &WorkspaceChange::Replace {
                expected: copied.into(),
                tabs: Box::new(readings.tabs),
                view: None,
            },
            &cancel,
        )
        .unwrap();
    assert_ne!(check(&copy), before);
    assert_eq!(check("source"), "old-tab");
    assert!(
        database
            .change_workspace(
                &WorkspaceChange::Rename {
                    expected: copied.into(),
                    name: "Stale write".into()
                },
                &cancel
            )
            .is_err()
    );
    let current = database
        .workspaces(0, &cancel)
        .unwrap()
        .entries
        .into_iter()
        .find(|item| item.id == copy)
        .unwrap();
    let current_readings = database.workspace(&(&current).into(), &cancel).unwrap();
    let mut current_tabs = current_readings.tabs;
    let active = current_tabs.active().unwrap();
    current_tabs
        .update_active(
            active.page(),
            active.preferences().clone(),
            Some(yamantaka_domain::reading_sessions::restoration::SessionPanel::Positions),
        )
        .unwrap();
    let mut view = current_readings.view;
    view.query = serde_json::json!({"text":"Appendix","list":true});
    view.focus = false;
    let updated = database
        .change_workspace(
            &WorkspaceChange::Replace {
                expected: (&current).into(),
                tabs: Box::new(current_tabs),
                view: Some(Box::new(view)),
            },
            &cancel,
        )
        .unwrap();
    let WorkspaceChanged::Saved(updated) = updated else {
        panic!("Replace must return its committed revision");
    };
    let snapshot = database.workspace(&(&updated).into(), &cancel).unwrap();
    assert_eq!(
        snapshot.tabs.active().unwrap().panel(),
        Some(yamantaka_domain::reading_sessions::restoration::SessionPanel::Positions)
    );
    assert_eq!(snapshot.view.query["custom"], "retained");
    assert_eq!(snapshot.view.parsed_query().unwrap().text, "Appendix");
    assert!(snapshot.view.parsed_query().unwrap().list);
    assert!(!snapshot.view.focus);
    let focused = database
        .change_workspace(
            &WorkspaceChange::Focus {
                expected: (&updated).into(),
                enabled: true,
            },
            &cancel,
        )
        .unwrap();
    let WorkspaceChanged::Saved(focused) = focused else {
        panic!("Focus must return its committed revision");
    };
    assert!(
        database
            .workspace(&(&focused).into(), &cancel)
            .unwrap()
            .view
            .focus
    );
    assert_ne!(focused.updated_at, updated.updated_at);
    let before = database.workspace(&(&focused).into(), &cancel).unwrap();
    let old_tab = before.tabs.tabs()[0].id().to_owned();
    let deleted = database
        .change_workspace(
            &WorkspaceChange::Delete {
                expected: (&focused).into(),
            },
            &cancel,
        )
        .unwrap();
    let WorkspaceChanged::Deleted(undo) = deleted else {
        panic!("Deletion must return an undo receipt");
    };
    assert!(
        database
            .startup_workspace(Some(&(&focused).into()), &cancel)
            .is_err()
    );
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(
        database
            .change_workspace(
                &WorkspaceChange::UndoDelete {
                    token: undo.token.clone()
                },
                &cancelled
            )
            .is_err()
    );
    let restored = database
        .change_workspace(
            &WorkspaceChange::UndoDelete {
                token: undo.token.clone(),
            },
            &cancel,
        )
        .unwrap();
    let WorkspaceChanged::Saved(restored) = restored else {
        panic!("Undo must return the restored workspace");
    };
    assert_eq!(restored.id, focused.id);
    assert_ne!(restored.updated_at, focused.updated_at);
    let restored_readings = database
        .startup_workspace(Some(&(&restored).into()), &cancel)
        .unwrap()
        .unwrap();
    assert_eq!(restored_readings.tabs.tabs()[0].id(), old_tab);
    assert_eq!(restored_readings.view.query["custom"], "retained");
    assert!(
        database
            .change_workspace(&WorkspaceChange::UndoDelete { token: undo.token }, &cancel)
            .is_err()
    );
    let reference = WorkspaceRevision::from(&restored);
    let mut continuity = restored_readings.tabs;
    continuity
        .set_workspace_reference(Some(reference.clone()))
        .unwrap();
    let continuity_path = root.path().join("continuity.json");
    yamantaka_storage::reader_continuity::save(&continuity_path, &continuity.into()).unwrap();
    let reread = yamantaka_storage::reader_continuity::load(&continuity_path)
        .unwrap()
        .unwrap();
    assert_eq!(reread.tabs().workspace_reference(), Some(&reference));
    assert_eq!(
        reread.tabs().active().unwrap().panel(),
        Some(yamantaka_domain::reading_sessions::restoration::SessionPanel::Positions)
    );
    let context: yamantaka_domain::reading_sessions::ReaderSessionContext =
        serde_json::from_value(serde_json::json!({"panelSection":"positions"})).unwrap();
    let restoration =
        yamantaka_domain::reading_sessions::restoration::ReaderSessionRestoration::try_from(
            context,
        )
        .unwrap();
    assert_eq!(
        restoration.panel(),
        Some(yamantaka_domain::reading_sessions::restoration::SessionPanel::Positions)
    );
    let comparison = WorkspaceComparison {
        bidirectional: false,
        swapped: true,
        differences: std::collections::BTreeSet::from([1, 3]),
        layout: ComparisonLayout::Stacked,
        scroll_offset: None,
        primary: "original".into(),
        document: "original".into(),
        page: 3,
        point: [0.6, 0.2],
        zoom_mode: "custom".into(),
        zoom: 1.6,
        theme: "sepia".into(),
        rotation: 90,
        horizontal_margin: 48.0,
        vertical_margin: 24.0,
        ratio: 0.6,
        follow_page: true,
        follow_zoom: false,
        follow_position: false,
    };
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&continuity_path).unwrap()).unwrap();
    assert_eq!(legacy["version"], 1);
    legacy["retainedExtension"] = serde_json::json!({"value":42});
    std::fs::write(&continuity_path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let context = yamantaka_domain::reader_continuity::ReaderContinuity::new(
        reread.tabs().clone(),
        Some(comparison.clone()),
    )
    .unwrap();
    yamantaka_storage::reader_continuity::save(&continuity_path, &context).unwrap();
    let restored = yamantaka_storage::reader_continuity::load(&continuity_path)
        .unwrap()
        .unwrap();
    assert_eq!(restored, context);
    let encoded = std::fs::read(&continuity_path).unwrap();
    let current: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(current["version"], 2);
    assert_eq!(
        current["retainedExtension"],
        serde_json::json!({"value":42})
    );
    for invalid in [serde_json::json!(99), serde_json::json!(2)] {
        let mut unsupported = current.clone();
        unsupported["version"] = invalid.clone();
        if invalid == 2 {
            unsupported["comparison"]["primary"] = "different-reading".into();
        }
        let bytes = serde_json::to_vec(&unsupported).unwrap();
        std::fs::write(&continuity_path, &bytes).unwrap();
        assert!(yamantaka_storage::reader_continuity::load(&continuity_path).is_err());
        assert!(yamantaka_storage::reader_continuity::save(&continuity_path, &context).is_err());
        assert_eq!(std::fs::read(&continuity_path).unwrap(), bytes);
    }
    std::fs::write(&continuity_path, &encoded).unwrap();
    let mut forgotten = context.clone();
    forgotten.forget_documents(&["unrelated".into()]);
    assert_eq!(forgotten.comparison(), Some(&comparison));
    forgotten.forget_documents(&["original".into()]);
    assert!(forgotten.comparison().is_none());
    let change = WorkspaceChange::Comparison {
        expected: reference.clone(),
        value: Some(comparison.clone()),
    };
    let WorkspaceChanged::Saved(with_comparison) =
        database.change_workspace(&change, &cancel).unwrap()
    else {
        panic!("Comparison must return its committed revision");
    };
    assert!(database.change_workspace(&change, &cancel).is_err());
    let snapshot = database
        .startup_workspace(Some(&(&with_comparison).into()), &cancel)
        .unwrap()
        .unwrap();
    assert_eq!(
        snapshot.view.parsed_query().unwrap().comparison,
        Some(comparison.clone())
    );
    assert_eq!(snapshot.reference_document.unwrap().id, "original");
    assert_eq!(
        snapshot.tabs.active().unwrap().panel(),
        Some(yamantaka_domain::reading_sessions::restoration::SessionPanel::Positions)
    );
    assert_eq!(snapshot.view.query["custom"], "retained");
    let mut invalid = comparison;
    invalid.point = [0.5, 1.1];
    assert!(
        database
            .change_workspace(
                &WorkspaceChange::Comparison {
                    expected: (&with_comparison).into(),
                    value: Some(invalid)
                },
                &cancel
            )
            .is_err()
    );
    let WorkspaceChanged::Saved(without_comparison) = database
        .change_workspace(
            &WorkspaceChange::Comparison {
                expected: (&with_comparison).into(),
                value: None,
            },
            &cancel,
        )
        .unwrap()
    else {
        panic!("Comparison removal must return its revision");
    };
    let snapshot = database
        .workspace(&(&without_comparison).into(), &cancel)
        .unwrap();
    assert!(snapshot.view.parsed_query().unwrap().comparison.is_none());
    assert!(snapshot.reference_document.is_none());
    assert_eq!(
        snapshot.tabs.active().unwrap().panel(),
        Some(yamantaka_domain::reading_sessions::restoration::SessionPanel::Positions)
    );
    assert_eq!(snapshot.view.query["custom"], "retained");
    database.integrity().unwrap();
}
