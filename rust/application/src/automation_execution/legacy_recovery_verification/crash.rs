use super::*;
use rusqlite::OptionalExtension;
use std::{
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};
const CHILD: &str = "YAMANTAKA_RECOVERY_CRASH_CHILD";
fn seed_many(path: &Path) -> Database {
    let db = Database::open(path).unwrap();
    let sql = Connection::open(path).unwrap();
    sql.execute_batch("WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<999) INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT printf('many-%04d',x),printf('many-%04d.pdf',x),printf('many-%04d.pdf',x),'fixture.pdf','Synthetic recovery',2,1,'source','','before','synthetic' FROM n;").unwrap();
    let mut receipts = serde_json::Map::new();
    for index in 0..1000 {
        receipts.insert(format!("r{index:04}"),serde_json::json!({"kind":"restoreFavorite","documentId":format!("many-{index:04}"),"value":true}));
    }
    sql.execute("INSERT INTO operation_journal(id,operation_kind,status,total_items,completed_items,undo_json,started_at,updated_at,phase,resume_capability) VALUES('legacy-recovery','automation','interrupted',1000,1000,?,'','before','applying','rollback')",[serde_json::Value::Object(receipts).to_string()]).unwrap();
    db
}
pub(crate) fn child_entry() -> bool {
    let Some(path) = std::env::var_os(CHILD) else {
        return false;
    };
    let db = Database::open(Path::new(&path)).unwrap();
    let governor = MemoryGovernor::new(Default::default());
    let plan = review(&db, &governor).unwrap();
    execute(
        &db,
        ExecutionRequest::new(ExecutionCommand::RecoverInherited(plan), governor),
    )
    .unwrap();
    true
}
pub(super) fn verify(original: &Database) {
    let path = original
        .file_path()
        .unwrap()
        .with_file_name("legacy-cancel.sqlite3");
    let db = seed_many(&path);
    let governor = MemoryGovernor::new(Default::default());
    let request = ExecutionRequest::new(
        ExecutionCommand::RecoverInherited(review(&db, &governor).unwrap()),
        governor.clone(),
    );
    let cancel = request.context.cancel.clone();
    let watch = request.context.watch.clone();
    let mut job = ExecutionJob::start(request, db.automation_executor().unwrap()).unwrap();
    let start = Instant::now();
    let result = loop {
        if watch.completed() > 0 {
            cancel.cancel();
        }
        if let Some(reply) = job.poll() {
            break reply.result.unwrap();
        }
        assert!(start.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(result.status, ExecutionStatus::Cancelled);
    assert!(result.completed > 0);
    assert!(!result.reversible);
    drop((result, job));
    let sql = Connection::open(&path).unwrap();
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM documents WHERE is_favorite=1",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(legacy(&db, &governor).status, ExecutionStatus::Interrupted);
    assert_eq!(governor.usage().total, 0);
    let path = original
        .file_path()
        .unwrap()
        .with_file_name("legacy-crash.sqlite3");
    let db = seed_many(&path);
    let sql = Connection::open(&path).unwrap();
    sql.busy_timeout(Duration::from_millis(100)).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact","file_comparison::tests::background_comparison_rejects_partial_hash_collision_and_confirms_full_copy","--nocapture"]).env(CHILD,&path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::from(std::fs::File::create(path.with_extension("child.log")).unwrap()));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().unwrap();
    let start = Instant::now();
    let mut saw_commit = false;
    while start.elapsed() < Duration::from_secs(10) {
        let progress=sql.query_row("SELECT completed_items FROM operation_journal WHERE operation_kind='rustAutomation.v1' AND compatibility_hash='yamantaka-legacy-recovery-v1' AND status='running' LIMIT 1",[],|row|row.get::<_,i64>(0)).optional().ok().flatten();
        if progress.is_some_and(|count| count > 0) {
            saw_commit = true;
            break;
        }
        if child.try_wait().unwrap().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        saw_commit,
        "The owned child did not reach a committed recovery prefix"
    );
    let reopened = Database::open(&path).unwrap();
    let page = reopened
        .automation_journals(0, &governor, &Cancellation::default())
        .unwrap();
    let interrupted = page.entries.iter().find(|entry| entry.recovery).unwrap();
    assert_eq!(interrupted.status, ExecutionStatus::Interrupted);
    assert!(interrupted.completed > 0 && interrupted.reversible);
    assert!(
        sql.query_row(
            "SELECT count(*) FROM documents WHERE is_favorite=1",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap()
            > 0
    );
    let undo = execute(
        &reopened,
        ExecutionRequest::new(
            ExecutionCommand::Undo(interrupted.revision.clone()),
            governor.clone(),
        ),
    )
    .unwrap();
    assert_eq!(undo.status, ExecutionStatus::RolledBack);
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM documents WHERE is_favorite=1",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        legacy(&reopened, &governor).status,
        ExecutionStatus::Interrupted
    );
    assert_eq!(
        sql.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    drop((page, undo, db));
    assert_eq!(governor.usage().total, 0);
}
