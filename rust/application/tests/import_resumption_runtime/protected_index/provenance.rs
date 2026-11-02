use super::*;

pub(super) fn verify() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("provenance.sqlite3");
    let mut cache_path = path.clone().into_os_string();
    cache_path.push(".file-hashes.sqlite3");
    let cache = rusqlite::Connection::open(std::path::PathBuf::from(cache_path)).unwrap();
    cache.execute_batch("CREATE TABLE file_hashes(document_id TEXT NOT NULL,source_key BLOB NOT NULL,digest BLOB NOT NULL,PRIMARY KEY(document_id,source_key)) WITHOUT ROWID; INSERT INTO file_hashes VALUES('sentinel',zeroblob(32),zeroblob(32)); PRAGMA user_version=1;").unwrap();
    let cancel = Cancellation::default();
    let mut db = Database::open(&path).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('proof','synthetic.pdf','synthetic.pdf','synthetic.pdf',2,1,'now','now','now','synthetic');").unwrap();
    let source = db.file_snapshot("proof").unwrap();
    let version: u32 = cache
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 2);
    let preserved: u32 = cache
        .query_row(
            "SELECT count(*) FROM file_hashes WHERE document_id='sentinel'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(preserved, 1);
    let begin = |db: &mut Database, digest| {
        db.begin_index(&source, digest, true, IndexAccess::Enabled, &cancel)
            .unwrap()
    };
    let mut session = begin(&mut db, [1; 32]);
    assert_eq!(session.cursor, 0);
    db.index_page(
        &mut session,
        0,
        "First source passage",
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    let mut stale = session.clone();
    drop(db);
    let mut db = Database::open(&path).unwrap();
    let resumed = begin(&mut db, [1; 32]);
    assert_eq!(
        resumed.cursor, 1,
        "A matching durable proof must survive a database restart"
    );
    assert!(
        db.index_page(&mut stale, 1, "Late result", IndexAccess::Enabled, &cancel)
            .is_err()
    );
    let mut changed = begin(&mut db, [2; 32]);
    assert_eq!(
        changed.cursor, 0,
        "Different full content must not share a cursor despite equal legacy identity"
    );
    db.index_page(
        &mut changed,
        0,
        "Second source passage",
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    cache
        .execute(
            "UPDATE index_sources SET revision='obsolete' WHERE document_id='proof'",
            [],
        )
        .unwrap();
    let mut reset = begin(&mut db, [2; 32]);
    assert_eq!(
        reset.cursor, 0,
        "A proof for an older committed revision must not authorize resume"
    );
    cache.execute_batch("CREATE TRIGGER reject_index_proof BEFORE INSERT ON index_sources BEGIN SELECT RAISE(ABORT,'Synthetic cache failure'); END;").unwrap();
    assert!(
        db.index_page(
            &mut reset,
            0,
            "Committed before cache failure",
            IndexAccess::Enabled,
            &cancel
        )
        .is_err()
    );
    cache
        .execute_batch("DROP TRIGGER reject_index_proof;")
        .unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    let committed: u32 = sql
        .query_row(
            "SELECT indexed_pages FROM search_index_state WHERE document_id='proof'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(committed, 1);
    let mut reset = begin(&mut db, [2; 32]);
    assert_eq!(
        reset.cursor, 0,
        "A failed post-commit proof write must restart instead of accepting unproven pages"
    );
    db.index_page(
        &mut reset,
        0,
        "Verified current source",
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    cache
        .execute("DELETE FROM index_sources WHERE document_id='proof'", [])
        .unwrap();
    let mut reset = begin(&mut db, [2; 32]);
    assert_eq!(
        reset.cursor, 0,
        "A missing or cleaned cache must restart safely"
    );
    db.index_page(
        &mut reset,
        0,
        "Preserved before relocation",
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    db.execute_batch("UPDATE documents SET path='relocated.pdf' WHERE id='proof';")
        .unwrap();
    assert!(
        db.index_page(
            &mut reset,
            1,
            "Wrong path result",
            IndexAccess::Enabled,
            &cancel
        )
        .is_err()
    );
    assert!(
        db.begin_index(&source, [2; 32], true, IndexAccess::Enabled, &cancel)
            .is_err()
    );
    let pages: Vec<String> = sql
        .prepare("SELECT body FROM library_search WHERE document_id='proof' AND kind='pageText'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(pages, vec!["Preserved before relocation"]);
    db.clear_index(&cancel).unwrap();
    let proofs: u32 = cache
        .query_row("SELECT count(*) FROM index_sources", [], |r| r.get(0))
        .unwrap();
    assert_eq!(proofs, 0);
}

pub(super) fn verify_tail_change(source: &Path, root: &Path) {
    use yamantaka_domain::file_check::FileCheckResult;
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("tail-change.pdf");
    let mut bytes = std::fs::read(source).unwrap();
    bytes.extend_from_slice(b"\n% ");
    bytes.extend(std::iter::repeat_n(b'A', 70000));
    std::fs::write(&target, &bytes).unwrap();
    let target = target.canonicalize().unwrap();
    let modified = target.metadata().unwrap().modified().unwrap();
    let fingerprint = yamantaka_storage::identity::legacy_fingerprint(&target).unwrap();
    let hash = yamantaka_storage::identity::full_sha256(&target, &Cancellation::default()).unwrap();
    let digest = std::array::from_fn(|i| u8::from_str_radix(&hash[i * 2..i * 2 + 2], 16).unwrap());
    let mut db = Database::open(&dir.path().join("guard.sqlite3")).unwrap();
    let path = target.to_string_lossy().replace('\'', "''");
    let date = chrono::DateTime::<chrono::Utc>::from(modified).to_rfc3339();
    db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('guard','{path}','{path}','tail-change.pdf',2,{},'{date}','now','now','{fingerprint}');",bytes.len())).unwrap();
    let cancel = Cancellation::default();
    let expected = db.file_snapshot("guard").unwrap();
    db.apply_file_check(
        &expected,
        FileCheckResult::Available {
            digest,
            modified: date,
            baseline_created: true,
        },
        &cancel,
    )
    .unwrap();
    let expected = db.file_snapshot("guard").unwrap();
    let mut session = db
        .begin_index(&expected, digest, true, IndexAccess::Enabled, &cancel)
        .unwrap();
    db.index_page(
        &mut session,
        0,
        "Prior source remains intact",
        IndexAccess::Enabled,
        &cancel,
    )
    .unwrap();
    *bytes.last_mut().unwrap() = b'B';
    std::fs::write(&target, &bytes).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&target)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert_eq!(
        yamantaka_storage::identity::legacy_fingerprint(&target).unwrap(),
        fingerprint
    );
    assert_ne!(
        yamantaka_storage::identity::full_sha256(&target, &cancel).unwrap(),
        hash
    );
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let (prompts, receive) = bounded(4);
    let (wake, _wakeups) = bounded(4);
    let mut controller = SearchIndexController::default().with_passwords(prompts, wake);
    controller
        .command(IndexCommand::Enable(true), &mut db)
        .unwrap();
    controller
        .command(IndexCommand::Start { rebuild: false }, &mut db)
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while controller.progress().phase != IndexPhase::Completed {
        assert!(Instant::now() < deadline, "Changed-source indexing stalled");
        controller.step(&mut db, &worker).unwrap();
        assert!(
            receive.try_recv().is_err(),
            "A changed source must be rejected before opening or asking for a password"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(controller.progress().failed_documents, 1);
    assert_eq!(controller.progress().completed_documents, 0);
    assert_eq!(controller.progress().last_error_code, Some("sourceChanged"));
    assert!(controller.native_document_id().is_none());
    let sql = rusqlite::Connection::open(dir.path().join("guard.sqlite3")).unwrap();
    let rows: Vec<String> = sql
        .prepare("SELECT body FROM library_search WHERE document_id='guard' AND kind='pageText'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(rows, vec!["Prior source remains intact"]);
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(&target)
            .is_ok()
    );
}
