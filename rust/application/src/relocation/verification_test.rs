use super::*;
use std::{fs, path::Path, time::Duration};
use yamantaka_storage::Database;
fn finish(mut job: RelocationJob, db: &mut Database) -> Result<RelocationOutcome> {
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(5)).unwrap();
    drop(select);
    let reply = job.poll(db).unwrap();
    drop(job);
    reply.result
}
fn request(action: RelocationAction, governor: &MemoryGovernor) -> RelocationRequest {
    RelocationRequest {
        action,
        governor: governor.clone(),
        cancel: Cancellation::default(),
    }
}
fn inspect(
    db: &mut Database,
    candidate: &Path,
    governor: &MemoryGovernor,
) -> Result<Box<RelocationPlan>> {
    let document = db.get("book").unwrap().unwrap();
    let job = RelocationJob::start(
        request(
            RelocationAction::Inspect {
                document: Box::new(document),
                candidate: candidate.into(),
            },
            governor,
        ),
        db,
    )?;
    match finish(job, db)? {
        RelocationOutcome::Review(plan) => Ok(plan),
        _ => panic!("Inspection must not write a relocation"),
    }
}
fn confirm(
    db: &mut Database,
    plan: Box<RelocationPlan>,
    governor: &MemoryGovernor,
) -> Result<RelocationOutcome> {
    let job = RelocationJob::start(request(RelocationAction::Confirm(plan), governor), db)?;
    finish(job, db)
}
pub(crate) fn verify(db: &mut Database, root: &Path, governor: &MemoryGovernor) {
    let document = db.get("book").unwrap().unwrap();
    let before = fs::read(&document.path).unwrap();
    let candidate = root.join("Lectura recuperada 日本語.pdf");
    fs::rename(&document.path, &candidate).unwrap();
    db.execute_batch("UPDATE documents SET is_missing=1 WHERE id='book'; INSERT INTO bookmarks VALUES('kept-bookmark','book',1,'Saved passage','2026-10-19','2026-10-19'); INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) SELECT 'kept-note',id,1,'note','violet','My original note',fingerprint,'2026-10-19','2026-10-19' FROM documents WHERE id='book'; INSERT INTO document_covers VALUES('book',1,'','','2026-10-19','2026-10-19',1,'manual',4,'contain',0.7);").unwrap();
    db.execute_batch("INSERT INTO search_index_state(document_id,fingerprint,status,indexed_pages,updated_at) SELECT id,fingerprint,'indexed',page_count,'2026-10-19T00:00:00Z' FROM documents WHERE id='book'; INSERT INTO library_search(document_id,page_index,kind,title,body,tags) VALUES('book',1,'pageText','A book','Previous searchable text','')").unwrap();
    let old = db.file_snapshot("book").unwrap();
    assert!(old.digest.is_some());
    let mut collision = before.clone();
    *collision.last_mut().unwrap() = b'?';
    fs::write(&candidate, &collision).unwrap();
    assert_eq!(
        yamantaka_storage::identity::legacy_fingerprint(&candidate).unwrap(),
        document.fingerprint
    );
    assert!(matches!(
        inspect(db, &candidate, governor),
        Err(Failure::Invalid(_))
    ));
    fs::write(&candidate, &before).unwrap();
    let plan = inspect(db, &candidate, governor).unwrap();
    assert!(plan.exact());
    assert_eq!(db.file_snapshot("book").unwrap(), old);
    // A change after the review is caught by a fresh full read at confirmation.
    fs::write(&candidate, &collision).unwrap();
    assert!(matches!(
        confirm(db, plan, governor),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(db.file_snapshot("book").unwrap(), old);
    fs::write(&candidate, &before).unwrap();
    let plan = inspect(db, &candidate, governor).unwrap();
    let encoded = plan.candidate.path.to_string_lossy().replace('\'', "''");
    let normalized = plan
        .candidate
        .path
        .to_string_lossy()
        .to_lowercase()
        .replace('\'', "''");
    db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT 'other','{encoded}','{normalized}','other.pdf',page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint FROM documents WHERE id='book'")).unwrap();
    assert!(matches!(
        confirm(db, plan.clone(), governor),
        Err(Failure::DestinationOccupied)
    ));
    assert_eq!(db.file_snapshot("book").unwrap(), old);
    db.execute_batch("DELETE FROM documents WHERE id='other'; CREATE TRIGGER reject_relocation BEFORE UPDATE OF path ON documents BEGIN SELECT RAISE(ABORT,'synthetic relocation interruption'); END;").unwrap();
    assert!(matches!(
        confirm(db, plan.clone(), governor),
        Err(Failure::Storage(_))
    ));
    assert_eq!(db.file_snapshot("book").unwrap(), old);
    db.execute_batch("DROP TRIGGER reject_relocation; UPDATE documents SET title_override='Latest manual title' WHERE id='book'").unwrap();
    let cancelled = request(RelocationAction::Confirm(plan.clone()), governor);
    let token = cancelled.cancel.clone();
    let job = RelocationJob::start(cancelled, db).unwrap();
    token.cancel();
    assert!(matches!(finish(job, db), Err(Failure::Cancelled)));
    assert_eq!(db.file_snapshot("book").unwrap(), old);
    assert!(matches!(
        confirm(db, plan, governor).unwrap(),
        RelocationOutcome::Applied
    ));
    let saved = db.get("book").unwrap().unwrap();
    assert_eq!(saved.path, candidate.canonicalize().unwrap());
    assert!(!saved.missing);
    assert_eq!(saved.id, document.id);
    assert_eq!(saved.fingerprint, document.fingerprint);
    assert_eq!(saved.display_title(), "Latest manual title");
    assert_eq!(saved.author, document.author);
    assert!(saved.favorite);
    assert_eq!(saved.last_page_index, 1);
    assert_eq!(fs::read(&candidate).unwrap(), before);
    assert!(!document.path.exists());
    let reopened = Database::open(&root.join("library.sqlite3")).unwrap();
    assert_eq!(reopened.file_snapshot("book").unwrap().digest, old.digest);
    drop(reopened);
    let sql = rusqlite::Connection::open(root.join("library.sqlite3")).unwrap();
    let retained:(String,String,u32)=sql.query_row("SELECT (SELECT label FROM bookmarks WHERE id='kept-bookmark'),(SELECT note FROM annotations WHERE id='kept-note'),(SELECT source_page_index FROM document_covers WHERE document_id='book')",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
    assert_eq!(
        retained,
        ("Saved passage".into(), "My original note".into(), 1)
    );
    let index:(String,u32,u32)=sql.query_row("SELECT status,indexed_pages,(SELECT count(*) FROM library_search WHERE document_id='book') FROM search_index_state WHERE document_id='book'",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
    assert_eq!(index, ("pending".into(), 0, 0));
    drop(sql);
    // Imported legacy libraries may have no full reference. Never call that exact.
    let mut cache_name = root.join("library.sqlite3").into_os_string();
    cache_name.push(".file-hashes.sqlite3");
    rusqlite::Connection::open(PathBuf::from(cache_name))
        .unwrap()
        .execute("DELETE FROM file_hashes WHERE document_id='book'", [])
        .unwrap();
    db.execute_batch("UPDATE documents SET is_missing=1 WHERE id='book'")
        .unwrap();
    db.execute_batch("UPDATE search_index_state SET status='excluded' WHERE document_id='book'")
        .unwrap();
    let legacy = inspect(db, &candidate, governor).unwrap();
    assert!(!legacy.exact());
    assert!(db.get("book").unwrap().unwrap().missing);
    let mut stale_plan = legacy.clone();
    stale_plan.expected.file.modified = "stale".into();
    assert!(matches!(
        confirm(db, stale_plan, governor),
        Err(Failure::Invalid(_))
    ));
    assert!(matches!(
        confirm(db, legacy, governor).unwrap(),
        RelocationOutcome::Applied
    ));
    assert_eq!(db.file_snapshot("book").unwrap().digest, old.digest);
    let sql = rusqlite::Connection::open(root.join("library.sqlite3")).unwrap();
    assert_eq!(
        sql.query_row(
            "SELECT status FROM search_index_state WHERE document_id='book'",
            [],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        "excluded"
    );
    assert_eq!(governor.usage().total, 0);
}
