use super::*;
use rusqlite::Connection;
use yamantaka_domain::LibraryRepository;
use yamantaka_storage::Database;
fn run(db: &Database, request: ExecutionRequest) -> Result<Arc<JournalEntry>> {
    let mut job = ExecutionJob::start(request, db.automation_executor()?)?;
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(10)).unwrap();
    drop(select);
    job.poll().unwrap().result
}
fn fresh(db: &Database, plan: &RulePlan, governor: &MemoryGovernor) -> Arc<RulePlan> {
    Arc::new(
        db.automation_reader().unwrap()()
            .unwrap()
            .prepare(plan.report().clone(), governor, &Cancellation::default())
            .unwrap(),
    )
}
fn state(sql: &Connection) -> String {
    sql.query_row("SELECT json_group_array(json_array(id,is_favorite,last_page_index,reading_progress,last_opened_at)) FROM (SELECT * FROM documents ORDER BY id)",[],|row|row.get(0)).unwrap()
}
pub(crate) fn verify(db: &Database, plan: Arc<RulePlan>, governor: &MemoryGovernor) {
    let sql = Connection::open(db.file_path().unwrap()).unwrap();
    let original = state(&sql);
    let old_mode: String = sql
        .query_row(
            "SELECT preferred_view_mode FROM documents WHERE id='book-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    sql.execute_batch("UPDATE documents SET is_favorite=1 WHERE id='book-0'; UPDATE documents SET preferred_view_mode='book' WHERE id='book-1'").unwrap();
    let plan = fresh(db, &plan, governor);
    let before = state(&sql);
    let receipt = run(
        db,
        ExecutionRequest::new(ExecutionCommand::Apply(plan.clone()), governor.clone()),
    )
    .unwrap();
    assert_eq!(receipt.status, ExecutionStatus::Completed);
    assert_eq!((receipt.completed, receipt.total), (3, 3));
    assert!(receipt.reversible);
    assert!(
        !sql.query_row(
            "SELECT is_favorite FROM documents WHERE id='book-0'",
            [],
            |row| row.get::<_, bool>(0)
        )
        .unwrap()
    );
    assert_eq!(sql.query_row("SELECT reading_mode FROM workspace_tabs WHERE workspace_id='plan-space' AND document_id='book-1'",[],|row|row.get::<_,String>(0)).unwrap(),"book");
    assert!(
        sql.query_row(
            "SELECT active_tab_id IS NOT NULL FROM workspaces WHERE id='plan-space'",
            [],
            |row| row.get::<_, bool>(0)
        )
        .unwrap()
    );
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM documents WHERE reading_progress=1",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        3
    );
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM workspace_tabs WHERE workspace_id='plan-space'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        3
    );
    let undo = run(
        db,
        ExecutionRequest::new(
            ExecutionCommand::Undo(receipt.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(undo.status, ExecutionStatus::RolledBack);
    assert!(!undo.reversible);
    assert_eq!(state(&sql), before);
    assert!(
        sql.query_row(
            "SELECT active_tab_id IS NULL FROM workspaces WHERE id='plan-space'",
            [],
            |row| row.get::<_, bool>(0)
        )
        .unwrap()
    );
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM collections WHERE normalized_name='new reading'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert!(
        run(
            db,
            ExecutionRequest::new(
                ExecutionCommand::Undo(receipt.revision.clone()),
                governor.clone()
            )
        )
        .is_err()
    );
    drop((receipt, undo));
    let request = ExecutionRequest::new(
        ExecutionCommand::Apply(fresh(db, &plan, governor)),
        governor.clone(),
    );
    let context = &request.context;
    let mut store = db.automation_executor().unwrap()().unwrap();
    let ExecutionCommand::Apply(reviewed) = &request.command else {
        unreachable!()
    };
    let cancelled = store
        .apply(reviewed, context, &mut |_| {
            if context.watch.completed() > 0 {
                context.cancel.cancel();
            }
            Ok(None)
        })
        .unwrap();
    assert_eq!(cancelled.status, ExecutionStatus::Cancelled);
    assert_eq!(cancelled.completed, 1);
    assert!(!cancelled.reversible);
    assert_eq!(state(&sql), before);
    drop((cancelled, store, request));
    let reviewed = fresh(db, &plan, governor);
    sql.execute_batch("UPDATE documents SET is_favorite=1 WHERE id='book-1'")
        .unwrap();
    let failed = run(
        db,
        ExecutionRequest::new(ExecutionCommand::Apply(reviewed), governor.clone()),
    )
    .unwrap();
    assert_eq!(failed.status, ExecutionStatus::Failed);
    assert!(!failed.reversible);
    assert!(
        sql.query_row(
            "SELECT is_favorite FROM documents WHERE id='book-1'",
            [],
            |row| row.get::<_, bool>(0)
        )
        .unwrap()
    );
    sql.execute_batch("UPDATE documents SET is_favorite=0 WHERE id='book-1'")
        .unwrap();
    assert_eq!(state(&sql), before);
    drop(failed);
    sql.execute_batch("CREATE TRIGGER reject_automation_queue BEFORE INSERT ON workspace_tabs WHEN NEW.document_id='book-1' BEGIN SELECT RAISE(ABORT,'fixture queue failure'); END;").unwrap();
    let failed = run(
        db,
        ExecutionRequest::new(
            ExecutionCommand::Apply(fresh(db, &plan, governor)),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(failed.status, ExecutionStatus::Failed);
    assert!(!failed.reversible);
    assert_eq!(state(&sql), before);
    sql.execute_batch("DROP TRIGGER reject_automation_queue")
        .unwrap();
    drop(failed);
    let receipt = run(
        db,
        ExecutionRequest::new(
            ExecutionCommand::Apply(fresh(db, &plan, governor)),
            governor.clone(),
        ),
    )
    .unwrap();
    let opened: String = sql
        .query_row(
            "SELECT last_opened_at FROM documents WHERE id='book-2'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    sql.execute_batch("UPDATE documents SET last_page_index=0,reading_progress=0.25,last_opened_at='later-reading' WHERE id='book-2'").unwrap();
    let conflict = run(
        db,
        ExecutionRequest::new(
            ExecutionCommand::Undo(receipt.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(conflict.status, ExecutionStatus::Failed);
    assert!(conflict.reversible);
    assert_eq!(conflict.error_code.as_deref(), Some("rollback_conflict"));
    assert_eq!(
        sql.query_row(
            "SELECT reading_progress FROM documents WHERE id='book-2'",
            [],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        0.25
    );
    sql.execute("UPDATE documents SET last_page_index=1,reading_progress=1,last_opened_at=? WHERE id='book-2'",[opened]).unwrap();
    let recovered = run(
        db,
        ExecutionRequest::new(
            ExecutionCommand::Undo(conflict.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(recovered.status, ExecutionStatus::RolledBack);
    assert_eq!(state(&sql), before);
    drop((receipt, conflict, recovered));
    let receipt = run(
        db,
        ExecutionRequest::new(
            ExecutionCommand::Apply(fresh(db, &plan, governor)),
            governor.clone(),
        ),
    )
    .unwrap();
    sql.execute(
        "UPDATE operation_journal SET phase='changed-without-timestamp' WHERE id=?",
        [&receipt.revision.id],
    )
    .unwrap();
    assert!(
        run(
            db,
            ExecutionRequest::new(
                ExecutionCommand::Undo(receipt.revision.clone()),
                governor.clone()
            )
        )
        .is_err()
    );
    sql.execute(
        "UPDATE operation_journal SET phase='completed' WHERE id=?",
        [&receipt.revision.id],
    )
    .unwrap();
    sql.execute_batch("UPDATE documents SET file_size=file_size+1 WHERE id='book-2'")
        .unwrap();
    let conflict = run(
        db,
        ExecutionRequest::new(
            ExecutionCommand::Undo(receipt.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(conflict.status, ExecutionStatus::Failed);
    assert!(conflict.reversible);
    assert_eq!(
        sql.query_row(
            "SELECT reading_progress FROM documents WHERE id='book-2'",
            [],
            |row| row.get::<_, f64>(0)
        )
        .unwrap(),
        1.0
    );
    sql.execute_batch("UPDATE documents SET file_size=file_size-1 WHERE id='book-2'; UPDATE operation_journal SET status='running',phase='applying' WHERE operation_kind='rustAutomation.v1' AND resume_capability='rollback'").unwrap();
    drop(conflict);
    let reopened = Database::open(&db.file_path().unwrap()).unwrap();
    let page = reopened
        .automation_journals(0, governor, &Cancellation::default())
        .unwrap();
    let interrupted = page
        .entries
        .iter()
        .find(|entry| entry.revision.id == receipt.revision.id)
        .unwrap();
    assert_eq!(interrupted.status, ExecutionStatus::Interrupted);
    assert!(interrupted.reversible);
    let recovered = run(
        &reopened,
        ExecutionRequest::new(
            ExecutionCommand::Undo(interrupted.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(recovered.status, ExecutionStatus::RolledBack);
    assert_eq!(state(&sql), before);
    assert!(
        sql.query_row(
            "SELECT count(*) FROM operation_journal WHERE operation_kind='rustAutomation.v1'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap()
            >= 6
    );
    assert_eq!(
        sql.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    sql.execute_batch("UPDATE documents SET is_favorite=0 WHERE id='book-0'")
        .unwrap();
    sql.execute(
        "UPDATE documents SET preferred_view_mode=? WHERE id='book-1'",
        [old_mode],
    )
    .unwrap();
    assert_eq!(state(&sql), original);
    super::inherited_verification::verify(db);
    super::legacy_recovery_verification::verify(db);
}
