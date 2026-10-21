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
    let copy = database
        .change_workspace(
            &WorkspaceChange::Duplicate {
                expected: (&source).into(),
                name: "Study copy".into(),
            },
            &cancel,
        )
        .unwrap();
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
    database.integrity().unwrap();
}
