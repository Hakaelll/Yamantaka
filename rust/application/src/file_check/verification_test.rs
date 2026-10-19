use super::*;
use std::{
    fs::{self, File, FileTimes},
    path::Path,
    time::Duration,
};
use yamantaka_storage::Database;
fn check(
    db: &mut Database,
    document: &Document,
    governor: &MemoryGovernor,
) -> Result<FileCheckResult> {
    let mut job = FileCheckJob::start(
        FileCheckRequest {
            document: document.clone(),
            governor: governor.clone(),
            cancel: Cancellation::default(),
        },
        db,
    )?;
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(5)).unwrap();
    let reply = job.poll(db).unwrap();
    assert!(!reply.cancel.is_cancelled());
    drop(job);
    assert_eq!(governor.usage().total, 0);
    reply.result
}
pub(crate) fn verify(
    db: &mut Database,
    document: &Document,
    root: &Path,
    governor: &MemoryGovernor,
) {
    let original = fs::read(&document.path).unwrap();
    let modified = document.path.metadata().unwrap().modified().unwrap();
    db.execute_batch("UPDATE documents SET title_override='My saved title',author_override='My saved author',is_favorite=1,last_page_index=1 WHERE id='book'").unwrap();
    let first = check(db, document, governor).unwrap();
    assert!(matches!(
        first,
        FileCheckResult::Available {
            baseline_created: true,
            ..
        }
    ));
    let digest = db.file_snapshot(&document.id).unwrap().digest.unwrap();
    assert_eq!(
        Database::open(&root.join("library.sqlite3"))
            .unwrap()
            .file_snapshot(&document.id)
            .unwrap()
            .digest,
        Some(digest)
    );
    assert!(matches!(
        check(db, document, governor).unwrap(),
        FileCheckResult::Available {
            baseline_created: false,
            ..
        }
    ));
    for seconds in 1..=4 {
        File::options()
            .write(true)
            .open(&document.path)
            .unwrap()
            .set_times(FileTimes::new().set_modified(modified + Duration::from_secs(seconds)))
            .unwrap();
        assert!(matches!(
            check(db, document, governor).unwrap(),
            FileCheckResult::Available {
                baseline_created: false,
                ..
            }
        ));
    }
    let mut cache_path = root.join("library.sqlite3").into_os_string();
    cache_path.push(".file-hashes.sqlite3");
    let cache = rusqlite::Connection::open(std::path::PathBuf::from(cache_path)).unwrap();
    let versions: u32 = cache
        .query_row(
            "SELECT count(*) FROM file_hashes WHERE document_id='book'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(versions, 2);
    drop(cache);
    let mut altered = original.clone();
    *altered.last_mut().unwrap() = b'!';
    fs::write(&document.path, &altered).unwrap();
    File::options()
        .write(true)
        .open(&document.path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(modified))
        .unwrap();
    assert_eq!(
        yamantaka_storage::identity::legacy_fingerprint(&document.path).unwrap(),
        document.fingerprint
    );
    assert_eq!(
        check(db, document, governor).unwrap(),
        FileCheckResult::Changed
    );
    assert_eq!(db.file_snapshot(&document.id).unwrap().digest, Some(digest));
    let moved = root.join("temporarily-moved.pdf");
    fs::rename(&document.path, &moved).unwrap();
    assert_eq!(
        check(db, document, governor).unwrap(),
        FileCheckResult::Missing
    );
    let missing = db.get(&document.id).unwrap().unwrap();
    assert!(missing.missing);
    fs::create_dir(&document.path).unwrap();
    assert!(matches!(
        check(db, &missing, governor),
        Err(Failure::Invalid(_))
    ));
    assert!(db.get(&document.id).unwrap().unwrap().missing);
    fs::remove_dir(&document.path).unwrap();
    fs::rename(&moved, &document.path).unwrap();
    fs::write(&document.path, &original).unwrap();
    File::options()
        .write(true)
        .open(&document.path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(matches!(
        check(db, &missing, governor).unwrap(),
        FileCheckResult::Available {
            baseline_created: false,
            ..
        }
    ));
    let saved = db.get(&document.id).unwrap().unwrap();
    assert!(!saved.missing);
    assert_eq!(saved.display_title(), "My saved title");
    assert_eq!(saved.author.as_deref(), Some("My saved author"));
    assert!(saved.favorite);
    assert_eq!(saved.last_page_index, 1);
    // A late catalogue change cannot be overwritten by the checking worker.
    let mut job = FileCheckJob::start(
        FileCheckRequest {
            document: saved.clone(),
            governor: governor.clone(),
            cancel: Cancellation::default(),
        },
        db,
    )
    .unwrap();
    db.execute_batch("UPDATE documents SET fingerprint='newer' WHERE id='book'")
        .unwrap();
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(
        job.poll(db).unwrap().result,
        Err(Failure::Invalid(_))
    ));
    drop(job);
    db.execute_batch(&format!(
        "UPDATE documents SET fingerprint='{}' WHERE id='book'",
        document.fingerprint
    ))
    .unwrap();
    // Disappearance followed by restoration must not publish a stale missing flag.
    fs::rename(&document.path, &moved).unwrap();
    let mut job = FileCheckJob::start(
        FileCheckRequest {
            document: saved.clone(),
            governor: governor.clone(),
            cancel: Cancellation::default(),
        },
        db,
    )
    .unwrap();
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(5)).unwrap();
    fs::rename(&moved, &document.path).unwrap();
    assert!(matches!(
        job.poll(db).unwrap().result,
        Err(Failure::Invalid(_))
    ));
    drop(job);
    assert!(!db.get(&document.id).unwrap().unwrap().missing);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        FileCheckJob::start(
            FileCheckRequest {
                document: saved,
                governor: governor.clone(),
                cancel
            },
            db
        ),
        Err(Failure::Cancelled)
    ));
    assert_eq!(governor.usage().total, 0);
    assert_eq!(fs::read(&document.path).unwrap(), original);
}
