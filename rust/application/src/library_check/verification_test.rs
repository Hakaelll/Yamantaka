use super::*;
use rusqlite::{Connection, params};
use std::{fs, path::Path, time::Duration};
use yamantaka_storage::{Database, identity};
fn drive(job: &mut LibraryCheckJob, db: &mut Database) -> bool {
    if !job.needs_step() {
        let mut select = Select::new();
        job.register(&mut select);
        select.ready_timeout(Duration::from_secs(5)).unwrap();
    }
    job.step(db)
}
pub(crate) fn verify(root: &Path, governor: &MemoryGovernor) {
    let database_path = root.join("batch-library.sqlite3");
    let mut db = Database::open(&database_path).unwrap();
    let request = || LibraryCheckRequest {
        governor: governor.clone(),
        cancel: Cancellation::default(),
    };
    let mut empty = LibraryCheckJob::start(request(), &db).unwrap();
    assert!(drive(&mut empty, &mut db));
    assert_eq!(empty.progress().phase, LibraryCheckPhase::Completed);
    assert_eq!(empty.progress().counts.checked(), 0);
    drop(empty);
    let sql = Connection::open(&database_path).unwrap();
    let bytes = vec![b'a'; 65549];
    for index in 0..85 {
        let id = format!("{:03}", index + 1);
        let path = root.join(format!("batch-{id}.pdf"));
        fs::write(&path, &bytes).unwrap();
        let path = path.canonicalize().unwrap();
        let stamp =
            chrono::DateTime::<chrono::Utc>::from(path.metadata().unwrap().modified().unwrap())
                .to_rfc3339();
        let fingerprint = identity::legacy_fingerprint(&path).unwrap();
        sql.execute("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES(?1,?2,?3,?4,2,65549,?5,?5,?5,?6)",params![id,path.to_string_lossy(),path.to_string_lossy().to_lowercase(),format!("batch-{id}.pdf"),stamp,fingerprint]).unwrap();
        match index {
            81 => {
                let mut changed = bytes.clone();
                changed[0] = b'!';
                fs::write(&path, changed).unwrap();
            }
            82 => {
                fs::remove_file(&path).unwrap();
            }
            83 => {
                sql.execute(
                    "UPDATE documents SET file_modified_at='unparseable' WHERE id=?",
                    [&id],
                )
                .unwrap();
            }
            84 => {
                fs::remove_file(&path).unwrap();
                fs::create_dir(&path).unwrap();
            }
            _ => {}
        }
    }
    sql.execute_batch("UPDATE documents SET title_override='Manual study title',author_override='Manual author',is_favorite=1,last_page_index=1,is_modified=1 WHERE id='001'").unwrap();
    assert_eq!(db.file_check_page(None, "085").unwrap().len(), 80);
    let mut cancelled = LibraryCheckJob::start(request(), &db).unwrap();
    for _ in 0..10 {
        if cancelled.progress().counts.checked() > 0 {
            break;
        }
        assert!(!drive(&mut cancelled, &mut db));
    }
    assert_eq!(cancelled.progress().counts.checked(), 1);
    cancelled.cancel().cancel();
    assert!(drive(&mut cancelled, &mut db));
    assert_eq!(cancelled.progress().phase, LibraryCheckPhase::Cancelled);
    assert_eq!(cancelled.progress().counts.checked(), 1);
    drop(cancelled);
    assert_eq!(governor.usage().total, 0);
    let mut job = LibraryCheckJob::start(request(), &db).unwrap();
    let mut edited = false;
    let mut complete = false;
    for _ in 0..400 {
        complete = drive(&mut job, &mut db);
        if !edited && job.progress().counts.checked() == 1 {
            sql.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT 'zz-later',path,'later-normalized','later.pdf',page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint FROM documents WHERE id='001'; DELETE FROM documents WHERE id='040'").unwrap();
            edited = true;
        }
        if complete {
            break;
        }
    }
    assert!(edited && complete);
    assert_eq!(job.progress().phase, LibraryCheckPhase::Completed);
    assert_eq!(
        job.progress().counts,
        CheckCounts {
            available: 80,
            missing: 1,
            changed: 1,
            unverifiable: 1,
            failed: 2
        }
    );
    drop(job);
    assert_eq!(governor.usage().total, 0);
    let modified = db
        .query(&yamantaka_domain::LibraryQuery {
            modified_only: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        modified
            .iter()
            .map(|doc| doc.id.as_str())
            .collect::<Vec<_>>(),
        ["082"]
    );
    let missing = db
        .query(&yamantaka_domain::LibraryQuery {
            missing_only: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        missing
            .iter()
            .map(|doc| doc.id.as_str())
            .collect::<Vec<_>>(),
        ["083"]
    );
    let saved = db.get("001").unwrap().unwrap();
    assert_eq!(saved.display_title(), "Manual study title");
    assert_eq!(saved.author.as_deref(), Some("Manual author"));
    assert!(saved.favorite);
    assert_eq!(saved.last_page_index, 1);
    assert_eq!(fs::read(root.join("batch-001.pdf")).unwrap(), bytes);
    let later: Option<String> = sql
        .query_row(
            "SELECT last_validated_at FROM documents WHERE id='zz-later'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(later.is_none());
    assert!(
        sql.query_row(
            "SELECT last_validated_at IS NOT NULL FROM documents WHERE id='001'",
            [],
            |row| row.get::<_, bool>(0)
        )
        .unwrap()
    );
    assert!(!db.get("085").unwrap().unwrap().missing);
}
