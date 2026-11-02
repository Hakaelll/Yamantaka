use super::*;
use yamantaka_domain::LibraryRepository;
use yamantaka_storage::Database;
pub(super) fn run(db: &Database, governor: &MemoryGovernor) -> Arc<Report> {
    let mut job = Job::start(
        Request {
            governor: governor.clone(),
            cancel: Cancellation::default(),
            watch: Arc::new(Watch::default()),
        },
        db.duplicate_reader().unwrap(),
    )
    .unwrap();
    let mut select = Select::new();
    job.register(&mut select);
    select
        .ready_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    drop(select);
    job.poll().unwrap().result.unwrap()
}
pub(crate) fn verify(db: &Database, governor: &MemoryGovernor) {
    let report = run(db, governor);
    assert_eq!(
        report.progress,
        Progress {
            checked: 3,
            matched: 2,
            unverified: 0
        }
    );
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].size, 70000);
    assert_eq!(
        report.groups[0]
            .sources
            .iter()
            .map(|source| source.catalogue.document.as_str())
            .collect::<Vec<_>>(),
        ["book-0", "book-1"]
    );
    assert!(
        report.groups[0]
            .sources
            .iter()
            .all(|source| source.proof.digest == report.groups[0].digest)
    );
    drop(report);
    assert_eq!(governor.usage().total, 0);
    let request = Request {
        governor: governor.clone(),
        cancel: Cancellation::default(),
        watch: Arc::new(Watch::default()),
    };
    request.cancel.cancel();
    assert!(matches!(
        Job::start(request, db.duplicate_reader().unwrap()),
        Err(Failure::Cancelled)
    ));
    db.execute_batch("WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<81) INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT printf('duplicate-page-%03d',x),d.path,printf('duplicate-page-%03d.pdf',x),'alias.pdf','Page alias',d.page_count,d.file_size,d.file_modified_at,d.added_at,d.last_opened_at,d.fingerprint FROM n CROSS JOIN documents d WHERE d.id='book-0';").unwrap();
    let reader = db.duplicate_reader().unwrap()().unwrap();
    let cancel = Cancellation::default();
    let first = reader.page(None, governor, &cancel).unwrap();
    assert_eq!(first.len(), 80);
    db.execute_batch(
        "UPDATE documents SET title='Changed after snapshot' WHERE id LIKE 'duplicate-page-%'",
    )
    .unwrap();
    let next = reader
        .page(Some(&first.last().unwrap().document), governor, &cancel)
        .unwrap();
    assert_eq!(next.len(), 5);
    assert!(next.iter().all(|source| source.title == "Page alias"));
    drop((first, next, reader));
    let report = run(db, governor);
    assert_eq!(
        report.progress,
        Progress {
            checked: 85,
            matched: 84,
            unverified: 0
        }
    );
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].sources.len(), 84);
    drop(report);
    db.execute_batch("DELETE FROM documents WHERE id LIKE 'duplicate-page-%'")
        .unwrap();
    assert_eq!(governor.usage().total, 0);
    super::merge_verification::verify(db, governor);
}
