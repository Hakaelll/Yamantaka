use super::*;
use crate::automation_plan::{PlanJob, PlanRequest, PreparedPlan};
use rusqlite::Connection;
use yamantaka_domain::{LibraryRepository, automation::legacy::LegacyRecoveryPlan};
use yamantaka_storage::Database;
#[path = "legacy_recovery_verification/crash.rs"]
mod crash;
pub(crate) use crash::child_entry;
fn legacy(db: &Database, governor: &MemoryGovernor) -> Arc<JournalEntry> {
    db.automation_journals(0, governor, &Cancellation::default())
        .unwrap()
        .entries
        .into_iter()
        .find(|entry| entry.revision.id == "legacy-recovery")
        .unwrap()
}
fn review(db: &Database, governor: &MemoryGovernor) -> Result<Arc<LegacyRecoveryPlan>> {
    let entry = legacy(db, governor);
    let request = PlanRequest::new_legacy(entry.revision.clone(), governor.clone());
    let mut job = PlanJob::start(request, db.automation_reader()?)?;
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(10)).unwrap();
    drop(select);
    job.poll().unwrap().result.map(|prepared| match prepared {
        PreparedPlan::Legacy(plan) => plan,
        PreparedPlan::Rule(_) => panic!("Unexpected rule plan"),
    })
}
fn execute(db: &Database, request: ExecutionRequest) -> Result<Arc<JournalEntry>> {
    let mut job = ExecutionJob::start(request, db.automation_executor()?)?;
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(10)).unwrap();
    drop(select);
    job.poll().unwrap().result
}
fn snapshot(sql: &Connection) -> Vec<Vec<String>> {
    let mut output = Vec::new();
    for table in [
        "documents",
        "document_tags",
        "collection_documents",
        "workspace_tabs",
        "workspaces",
    ] {
        let mut statement = sql
            .prepare(&format!("SELECT * FROM {table} ORDER BY 1,2"))
            .unwrap();
        let mut rows = statement.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let mut values = vec![table.into()];
            for index in 0..row.as_ref().column_count() {
                if table == "workspaces" && row.as_ref().column_name(index).unwrap() == "updated_at"
                {
                    continue;
                }
                values.push(format!("{:?}", row.get_ref(index).unwrap()));
            }
            output.push(values);
        }
    }
    output
}
fn seed(sql: &Connection) {
    sql.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,last_page_index,reading_progress) VALUES('d0','fixture-0.pdf','fixture-0.pdf','fixture-0.pdf','Paper one',3,1,'source','','before-0','synthetic',2,0.9),('d1','fixture-1.pdf','fixture-1.pdf','fixture-1.pdf','Paper two',3,1,'source','','before-1','synthetic',0,0.2); INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('lt','Imported refs','imported refs','violet','',''); INSERT INTO document_tags(document_id,tag_id) VALUES('d0','lt'); INSERT INTO collections(id,name,normalized_name,created_at,updated_at) VALUES('lc','Imported sources','imported sources','',''); INSERT INTO collection_documents(collection_id,document_id,added_at) VALUES('lc','d0','old-added'); INSERT INTO workspaces(id,name,active_tab_id,split_json,created_at,updated_at) VALUES('lw','Recovery desk','q0',' {\"enabled\":true,\"secondaryTabId\":\"q0\",\"vendor\":{\"keep\":\"Ω\"}} ','','workspace-before'); INSERT INTO workspace_tabs(id,workspace_id,document_id,sort_order,is_pinned,page_index,zoom_level,page_layout,reading_mode,scroll_offset,queue_status,blocking_reason,next_action) VALUES('q0','lw','d0',0,1,1,1.7,'facing','book',31,'inReview','Keep Ω','Compare sources'),('q1','lw','d1',1,0,0,1,'single','document',0,'pending',NULL,NULL); INSERT INTO library_search(document_id,page_index,kind,title,body,tags) VALUES('d0',-1,'tag','Paper one','','Imported refs');").unwrap();
    let undo = r#" {"a":{"kind":"removeTag","documentId":"d0","tagId":"lt","changed":true},"b":{"kind":"removeCollection","documentId":"d0","collectionId":"lc","changed":true},"c":{"kind":"removeQueue","documentId":"d0","workspaceId":"lw","changed":true},"d":{"kind":"restoreProgress","documentId":"d0","pageIndex":1,"pageCount":3,"viewMode":"book","theme":"sepia"},"e":{"kind":"restoreFavorite","documentId":"d0","value":true},"f":{"kind":"restoreFavorite","documentId":"d1","value":true},"g":{"kind":"removeTag","documentId":"absent-skipped","tagId":"lt","changed":false}} "#;
    sql.execute("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,undo_json,started_at,updated_at,phase,resume_capability,interruption_reason,error_code) VALUES('legacy-recovery','automation','interrupted','{}',2,2,?,'','legacy-before','applying','rollback','process_interrupted','old-code')",[undo]).unwrap();
}
pub(crate) fn verify(original: &Database) {
    let path = original
        .file_path()
        .unwrap()
        .with_file_name("legacy-recovery.sqlite3");
    let mut db = Database::open(&path).unwrap();
    let sql = Connection::open(&path).unwrap();
    seed(&sql);
    let governor = MemoryGovernor::new(Default::default());
    let before = snapshot(&sql);
    let raw: String = sql
        .query_row(
            "SELECT undo_json FROM operation_journal WHERE id='legacy-recovery'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let plan = review(&db, &governor).unwrap();
    assert_eq!(
        (plan.documents.len(), plan.targets.len(), plan.skipped),
        (2, 3, 1)
    );
    assert_eq!(plan.documents[0].id, "d1");
    assert_eq!(snapshot(&sql), before);
    let done = execute(
        &db,
        ExecutionRequest::new(
            ExecutionCommand::RecoverInherited(plan.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(done.status, ExecutionStatus::Completed);
    assert!(done.recovery && done.reversible);
    assert_eq!(done.completed, 2);
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM documents WHERE is_favorite=1",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        2
    );
    assert_eq!(
        sql.query_row(
            "SELECT reading_progress FROM documents WHERE id='d0'",
            [],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        0.5
    );
    assert_eq!(
        sql.query_row(
            "SELECT preferred_view_mode||':'||preferred_theme FROM documents WHERE id='d0'",
            [],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        "book:sepia"
    );
    assert_eq!(
        sql.query_row("SELECT count(*) FROM workspace_tabs", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        sql.query_row(
            "SELECT active_tab_id FROM workspaces WHERE id='lw'",
            [],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        "q1"
    );
    assert_eq!(
        sql.query_row(
            "SELECT json_extract(split_json,'$.vendor.keep') FROM workspaces WHERE id='lw'",
            [],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        "Ω"
    );
    assert_eq!(
        sql.query_row("SELECT count(*) FROM document_tags", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        sql.query_row("SELECT count(*) FROM collection_documents", [], |row| row
            .get::<_, i64>(
            0
        ))
        .unwrap(),
        0
    );
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM library_search WHERE kind='tag' AND document_id='d0'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(legacy(&db, &governor).status, ExecutionStatus::RolledBack);
    assert!(review(&db, &governor).is_err());
    let undo = execute(
        &db,
        ExecutionRequest::new(
            ExecutionCommand::Undo(done.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(undo.status, ExecutionStatus::RolledBack);
    assert_eq!(snapshot(&sql), before);
    assert!(matches!(
        legacy(&db, &governor).origin,
        JournalOrigin::Flutter { has_undo: true, .. }
    ));
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM library_search WHERE kind='tag' AND document_id='d0'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    drop((plan, done, undo));
    let plan = review(&db, &governor).unwrap();
    sql.execute_batch("UPDATE documents SET is_favorite=1 WHERE id='d1'")
        .unwrap();
    let failed = execute(
        &db,
        ExecutionRequest::new(ExecutionCommand::RecoverInherited(plan), governor.clone()),
    )
    .unwrap();
    assert_eq!(failed.status, ExecutionStatus::Failed);
    assert!(!failed.reversible);
    assert!(
        sql.query_row(
            "SELECT is_favorite FROM documents WHERE id='d1'",
            [],
            |row| row.get::<_, bool>(0)
        )
        .unwrap()
    );
    sql.execute_batch("UPDATE documents SET is_favorite=0 WHERE id='d1'")
        .unwrap();
    drop(failed);
    assert_eq!(snapshot(&sql), before);
    sql.execute_batch("CREATE TRIGGER reject_legacy_delete BEFORE DELETE ON document_tags BEGIN SELECT RAISE(ABORT,'synthetic legacy failure'); END;").unwrap();
    let failed = execute(
        &db,
        ExecutionRequest::new(
            ExecutionCommand::RecoverInherited(review(&db, &governor).unwrap()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(failed.status, ExecutionStatus::Failed);
    assert!(!failed.reversible);
    assert_eq!(failed.completed, 1);
    assert_eq!(snapshot(&sql), before);
    drop(failed);
    sql.execute_batch("DROP TRIGGER reject_legacy_delete")
        .unwrap();
    let done = execute(
        &db,
        ExecutionRequest::new(
            ExecutionCommand::RecoverInherited(review(&db, &governor).unwrap()),
            governor.clone(),
        ),
    )
    .unwrap();
    sql.execute_batch("UPDATE workspace_tabs SET next_action='Later queue note' WHERE id='q1'")
        .unwrap();
    let conflict = execute(
        &db,
        ExecutionRequest::new(
            ExecutionCommand::Undo(done.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(conflict.status, ExecutionStatus::Failed);
    assert!(conflict.reversible);
    assert_eq!(
        sql.query_row(
            "SELECT next_action FROM workspace_tabs WHERE id='q1'",
            [],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        "Later queue note"
    );
    sql.execute_batch("UPDATE workspace_tabs SET next_action=NULL WHERE id='q1'")
        .unwrap();
    let restored = execute(
        &db,
        ExecutionRequest::new(
            ExecutionCommand::Undo(conflict.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(restored.status, ExecutionStatus::RolledBack);
    assert_eq!(snapshot(&sql), before);
    drop((done, conflict, restored));
    let entry = legacy(&db, &governor);
    let tiny = MemoryGovernor::new(yamantaka_domain::memory::GlobalMemoryLimits {
        total: 1,
        ..Default::default()
    });
    assert!(matches!(
        db.automation_reader().unwrap()().unwrap().prepare_legacy(
            &entry.revision,
            &tiny,
            &Cancellation::default()
        ),
        Err(Failure::MemoryBudget)
    ));
    drop(entry);
    for invalid in [
        r#"{"x":{"kind":"futureUndo","documentId":"d0"}}"#,
        r#"{"x":{"kind":"restoreFavorite","documentId":"d0","value":true,"vendor":7}}"#,
        r#"{"x":{"kind":"restoreFavorite","documentId":"d0","value":true},"x":{"kind":"restoreFavorite","documentId":"d0","value":false}}"#,
    ] {
        sql.execute(
            "UPDATE operation_journal SET undo_json=? WHERE id='legacy-recovery'",
            [invalid],
        )
        .unwrap();
        assert!(review(&db, &governor).is_err());
        assert_eq!(
            sql.query_row(
                "SELECT undo_json FROM operation_journal WHERE id='legacy-recovery'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
            invalid
        );
        assert_eq!(snapshot(&sql), before);
    }
    sql.execute(
        "UPDATE operation_journal SET undo_json=? WHERE id='legacy-recovery'",
        [&raw],
    )
    .unwrap();
    db.save_position("d0", 1).unwrap();
    assert_eq!(db.get("d0").unwrap().unwrap().reading_progress, 0.5);
    sql.execute_batch("UPDATE documents SET page_count=1 WHERE id='d0'")
        .unwrap();
    db.save_position("d0", 0).unwrap();
    assert_eq!(db.get("d0").unwrap().unwrap().reading_progress, 1.0);
    sql.execute_batch("UPDATE documents SET page_count=0 WHERE id='d0'")
        .unwrap();
    db.save_position("d0", 0).unwrap();
    assert_eq!(db.get("d0").unwrap().unwrap().reading_progress, 0.0);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(
        sql.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    crash::verify(original);
}
