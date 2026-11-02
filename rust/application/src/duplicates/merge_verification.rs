use super::*;
use yamantaka_domain::{LibraryRepository, duplicates::merge::*};
use yamantaka_storage::{Database, backup, preferences::Preferences};
fn state(db: &Database, path: &std::path::Path) -> serde_json::Value {
    db.export_backup(path, &Preferences::default(), &Cancellation::default())
        .unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    value.as_object_mut().unwrap().remove("exportedAt");
    value.as_object_mut().unwrap().remove("manifest");
    value
}
pub(crate) fn verify(source: &Database, governor: &MemoryGovernor) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("merge.sqlite3");
    source.snapshot(&path).unwrap();
    let mut db = Database::open(&path).unwrap();
    db.execute_batch(r#"
        UPDATE documents SET title_override=CASE WHEN id='book-0' THEN 'Kept correction' ELSE 'Incoming correction' END,author_override=CASE WHEN id='book-0' THEN 'Kept author' ELSE 'Incoming author' END,is_favorite=(id='book-1'),reading_progress=CASE WHEN id='book-0' THEN 0.2 ELSE 0.8 END,last_page_index=(id='book-1') WHERE id IN ('book-0','book-1');
        INSERT INTO document_notes VALUES('book-0','Kept note',''),('book-1','Incoming note','');
        INSERT INTO document_preferences(document_id,view_mode,reading_theme,zoom_mode,zoom_level,page_layout,page_rotation,horizontal_margin,vertical_margin,controls_auto_hide,scroll_offset,updated_at,cover_policy,initial_side) VALUES('book-1','book','ivory','fitWidth',1.5,'double',90,12,8,0,27,'','firstPageIsCover','right');
        INSERT INTO bookmarks VALUES('kept-mark','book-0',0,NULL,'',''),('incoming-collision','book-1',0,'Imported label','',''),('incoming-mark','book-1',1,'Second page','','');
        INSERT INTO reading_sessions VALUES('merge-session','book-1','2026-10-01T10:00:00Z','2026-10-01T10:01:00Z',0,1,12);
        INSERT INTO reading_sessions VALUES('merge-unfinished','book-1','2026-09-30T10:00:00Z',NULL,0,1,5);
        INSERT INTO annotations(id,document_id,page_index,type,color,note,selected_text,rects_json,page_rotation,document_fingerprint,created_at,updated_at,review_state) VALUES('merge-note','book-1',1,'highlight','yellow','Keep geometry','Passage','[{"x":0.1,"y":0.2,"width":0.3,"height":0.1}]',90,'inherited','','','accepted');
        INSERT INTO annotation_tags VALUES('merge-note','tag-id');
        INSERT INTO document_tags VALUES('book-1','tag-id');
        INSERT INTO collection_documents VALUES('collection-id','book-1','');
        INSERT INTO workspaces(id,name,active_tab_id,created_at,updated_at) VALUES('merge-space','Merge desk','incoming-tab','',''),('merge-second-space','Second desk','only-tab','','');
        INSERT INTO workspace_tabs(id,workspace_id,document_id,sort_order,page_index,queue_status,blocking_reason,next_action) VALUES('kept-tab','merge-space','book-0',0,0,'pending',NULL,NULL),('incoming-tab','merge-space','book-1',1,1,'blocked','Review','Read'),('only-tab','merge-second-space','book-1',0,1,'inReview',NULL,'Compare');
        INSERT INTO document_relations(id,source_document_id,target_document_id,relation_type,created_at) VALUES('collapsed-edge','book-0','book-1','related',''),('kept-edge','book-0','book-2','related',''),('incoming-edge','book-1','book-2','related',''),('unique-edge','book-2','book-1','sourceOf','');
        INSERT INTO document_revision_groups VALUES('merge-revision','Editions','book-1','','');
        INSERT INTO document_revision_members VALUES('merge-revision','book-0',''),('merge-revision','book-1','');
        INSERT INTO document_covers VALUES('book-0',0,'cache-a','cache-b','','',0.5,'automatic',4,'contain',0.7),('book-1',1,'cache-c','cache-d','','',1,'manual',4,'contain',0.75);
        INSERT INTO document_topics VALUES('book-0','shared',0.5,'automatic',4,'legacy',''),('book-1','shared',1,'manual',4,'legacy',''),('book-1','incoming-topic',1,'manual',4,'legacy','');
        INSERT INTO document_enrichment(document_id,fingerprint,analyzer_version,status,topics_manual) VALUES('book-1','legacy',4,'completed',1);
        INSERT INTO search_index_state VALUES('book-0','legacy','indexed',2,'',NULL),('book-1','legacy','excluded',0,'',NULL);
        INSERT INTO library_search(document_id,page_index,kind,title,body,tags) VALUES('book-0',0,'page','A','Retire old text',''),('book-1',0,'page','B','Retire duplicate text','');
        INSERT INTO app_metadata VALUES('reading_goals_v1','{"book-1":{"pages":12,"minutes":30,"future":null},"unrelated":{"future":true}}',''),('reading_session_contexts_v1','{"merge-session":{"zoomLevel":1.5,"future":"retained"}}','');
    "#).unwrap();
    let report = super::verification_test::run(&db, governor);
    let selection = || Selection::from_group(&report.groups[0], "book-0", "book-1").unwrap();
    let mut store = db
        .duplicate_merge_store(serde_json::to_value(Preferences::default()).unwrap())
        .unwrap()()
    .unwrap();
    let cancel = Cancellation::default();
    let choices = Choices {
        title: Choice::Incoming,
        author: Choice::Kept,
        note: Choice::Incoming,
    };
    let plan = store.prepare(selection(), governor, &cancel).unwrap();
    assert!(plan.title.conflict() && plan.author.conflict() && plan.note.conflict());
    assert_eq!(plan.affected["annotations"], 1);
    let mut tabs = yamantaka_domain::reader_tabs::DocumentTabs::default();
    let mut preferences = yamantaka_domain::reader_preferences::ReaderPreferences::new("book-1");
    preferences.zoom_level = 1.5;
    tabs.open(
        yamantaka_domain::reader_tabs::ReaderTab::new(
            "recovered-tab".into(),
            "book-1".into(),
            1,
            preferences.clone(),
            None,
        )
        .unwrap(),
        true,
    )
    .unwrap();
    tabs.open(
        yamantaka_domain::reader_tabs::ReaderTab::new(
            "closed-tab".into(),
            "book-1".into(),
            0,
            preferences,
            None,
        )
        .unwrap(),
        true,
    )
    .unwrap();
    tabs.close("closed-tab");
    db.save_reader_continuity(&yamantaka_domain::reader_continuity::ReaderContinuity::from(tabs))
        .unwrap();
    let before = state(&db, &root.path().join("before.json"));
    let original_path = &plan.selection.incoming.file.path;
    let original_bytes = std::fs::read(original_path).unwrap();
    let original_time = original_path.metadata().unwrap().modified().unwrap();
    let mut changed_bytes = original_bytes.clone();
    *changed_bytes.last_mut().unwrap() = b'Z';
    std::fs::write(original_path, &changed_bytes).unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(original_path)
        .unwrap()
        .set_modified(original_time)
        .unwrap();
    assert!(super::merge_sources::freeze(&plan.selection, governor, &cancel).is_err());
    std::fs::write(original_path, &original_bytes).unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(original_path)
        .unwrap()
        .set_modified(original_time)
        .unwrap();
    {
        let guard = super::merge_sources::freeze(&plan.selection, governor, &cancel).unwrap();
        #[cfg(windows)]
        assert!(
            std::fs::OpenOptions::new()
                .write(true)
                .open(original_path)
                .is_err()
        );
        guard.verify().unwrap();
    }

    db.execute_batch(
        "UPDATE document_notes SET note='Changed after review' WHERE document_id='book-1'",
    )
    .unwrap();
    assert!(
        store
            .apply(&plan, &choices, governor, &cancel, &mut |selection| {
                super::merge_sources::freeze(selection, governor, &cancel)
            })
            .is_err()
    );
    db.execute_batch("UPDATE document_notes SET note='Incoming note' WHERE document_id='book-1'")
        .unwrap();
    drop(plan);
    let plan = store.prepare(selection(), governor, &cancel).unwrap();
    db.execute_batch("CREATE TRIGGER reject_merge_fixture BEFORE DELETE ON documents WHEN OLD.id='book-1' BEGIN SELECT RAISE(ABORT,'injected consolidation failure'); END;").unwrap();
    assert!(
        store
            .apply(&plan, &choices, governor, &cancel, &mut |selection| {
                super::merge_sources::freeze(selection, governor, &cancel)
            })
            .is_err()
    );
    db.execute_batch("DROP TRIGGER reject_merge_fixture")
        .unwrap();
    assert_eq!(state(&db, &root.path().join("failed.json")), before);
    let receipt = store
        .apply(&plan, &choices, governor, &cancel, &mut |selection| {
            super::merge_sources::freeze(selection, governor, &cancel)
        })
        .unwrap();
    assert_eq!(receipt.kept, "book-0");
    assert_eq!(receipt.incoming, "book-1");
    assert_eq!(
        yamantaka_storage::identity::full_sha256(&receipt.backup, &cancel).unwrap(),
        receipt.backup_sha256
    );
    let original = backup::preview(&receipt.backup, &cancel).unwrap();
    assert_eq!(
        state(
            &original.database,
            &root.path().join("restored-before.json")
        ),
        before
    );
    assert_eq!(
        db.canonical_document_id("book-1").unwrap().as_deref(),
        Some("book-0")
    );
    assert_eq!(
        db.reader_continuity()
            .unwrap()
            .unwrap()
            .tabs()
            .active()
            .unwrap()
            .document(),
        "book-1"
    );
    let recovered = crate::reader_continuity::ContinuityWriter::default()
        .execute(
            &mut Database::open(&path).unwrap(),
            crate::reader_continuity::ContinuityRequest {
                change: crate::reader_continuity::ContinuityChange::Load,
                cancel: Cancellation::default(),
            },
        )
        .result
        .unwrap()
        .unwrap();
    assert_eq!(recovered.tabs().active().unwrap().id(), "recovered-tab");
    assert_eq!(recovered.tabs().active().unwrap().document(), "book-0");
    assert_eq!(recovered.tabs().active().unwrap().page(), 1);
    assert_eq!(
        recovered.tabs().active().unwrap().preferences().zoom_level,
        1.5
    );
    assert_eq!(recovered.tabs().closed()[0].document(), "book-0");
    assert_eq!(
        db.reader_continuity()
            .unwrap()
            .unwrap()
            .tabs()
            .active()
            .unwrap()
            .document(),
        "book-1"
    );
    let proof_db = rusqlite::Connection::open(&path).unwrap();
    let proof: String = proof_db
        .query_row(
            "SELECT summary_json FROM operation_journal WHERE id=?",
            [&receipt.history],
            |row| row.get(0),
        )
        .unwrap();
    proof_db
        .execute(
            "UPDATE operation_journal SET summary_json='{}' WHERE id=?",
            [&receipt.history],
        )
        .unwrap();
    assert!(db.canonical_document_id("book-1").is_err());
    proof_db
        .execute(
            "UPDATE operation_journal SET summary_json=?1 WHERE id=?2",
            rusqlite::params![proof, receipt.history],
        )
        .unwrap();
    let after = state(&db, &root.path().join("after.json"));
    let tables = &after["tables"];
    assert_eq!(tables["documents"].as_array().unwrap().len(), 2);
    let kept = tables["documents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "book-0")
        .unwrap();
    assert_eq!(kept["title_override"], "Incoming correction");
    assert_eq!(kept["author_override"], "Kept author");
    assert_eq!(kept["is_favorite"], 1);
    assert_eq!(kept["last_page_index"], 1);
    assert_eq!(kept["reading_progress"], 0.8);
    assert_eq!(tables["document_notes"][0]["note"], "Incoming note");
    assert_eq!(tables["annotations"][0]["id"], "merge-note");
    assert_eq!(tables["annotations"][0]["document_id"], "book-0");
    assert_eq!(tables["annotations"][0]["origin_document_id"], "book-1");
    assert_eq!(
        tables["annotations"][0]["rects_json"],
        before["tables"]["annotations"][0]["rects_json"]
    );
    assert_eq!(
        tables["document_preferences"][0]["cover_policy"],
        "firstPageIsCover"
    );
    assert_eq!(tables["document_preferences"][0]["initial_side"], "right");
    assert_eq!(tables["bookmarks"].as_array().unwrap().len(), 2);
    assert_eq!(tables["reading_sessions"][0]["document_id"], "book-0");
    assert_eq!(tables["document_covers"][0]["source_page_index"], 1);
    assert_eq!(tables["document_covers"][0]["selection_mode"], "manual");
    assert_eq!(tables["document_relations"].as_array().unwrap().len(), 2);
    assert_eq!(tables["workspace_tabs"].as_array().unwrap().len(), 2);
    assert_eq!(
        tables["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == "merge-space")
            .unwrap()["active_tab_id"],
        "kept-tab"
    );
    assert_eq!(
        tables["document_revision_groups"][0]["current_document_id"],
        "book-0"
    );
    assert_eq!(tables["document_topics"].as_array().unwrap().len(), 2);
    assert!(
        tables["document_topics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["origin"] == "manual")
    );
    for table in [
        "tags",
        "collections",
        "reading_profiles",
        "automation_rules",
        "annotation_tags",
    ] {
        assert_eq!(
            tables[table], before["tables"][table],
            "Unrelated or identity-bound records changed: {table}"
        );
    }
    for table in ["reading_sessions", "document_preferences"] {
        let mut expected = before["tables"][table].clone();
        for row in expected.as_array_mut().unwrap() {
            if row["document_id"] == "book-1" {
                row["document_id"] = "book-0".into();
            }
        }
        assert_eq!(
            tables[table], expected,
            "Historical fields changed: {table}"
        );
    }
    let mut annotation = before["tables"]["annotations"][0].clone();
    annotation["document_id"] = "book-0".into();
    annotation["origin_document_id"] = "book-1".into();
    annotation["origin_fingerprint"] = annotation["document_fingerprint"].clone();
    assert_eq!(tables["annotations"][0], annotation);
    assert_eq!(
        db.cover_snapshot("book-0").unwrap().plan,
        yamantaka_domain::cover_snapshot::CoverPlan::Render(1)
    );
    let original_journals = tables["operation_journal"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["operation_kind"] != "rustDuplicateMerge.v1")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::json!(original_journals),
        before["tables"]["operation_journal"]
    );
    assert_eq!(
        tables["operation_journal"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["operation_kind"] == "rustDuplicateMerge.v1")
            .count(),
        1
    );
    let sql = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM library_search WHERE document_id IN ('book-0','book-1')",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        sql.query_row(
            "SELECT status FROM search_index_state WHERE document_id='book-0'",
            [],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        "excluded"
    );
    let goals: String = sql
        .query_row(
            "SELECT value FROM app_metadata WHERE key='reading_goals_v1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let goals: serde_json::Value = serde_json::from_str(&goals).unwrap();
    assert_eq!(goals["book-0"], goals["book-1"]);
    assert_eq!(goals["unrelated"]["future"], true);
    assert!(
        store
            .apply(&plan, &choices, governor, &cancel, &mut |selection| {
                super::merge_sources::freeze(selection, governor, &cancel)
            })
            .is_err()
    );
    drop((plan, report, store));
    assert_eq!(governor.usage().total, 0);
}
