use super::*;
use rusqlite::{Connection, params};
use std::{fs, path::Path};
use yamantaka_domain::LibraryRepository;
use yamantaka_storage::{Database, identity};

fn saved(
    db: &mut Database,
    governor: &MemoryGovernor,
    kind: TriggerKind,
    value: Option<&str>,
) -> Arc<SavedRule> {
    db.execute_batch("DELETE FROM automation_rules").unwrap();
    db.change_automation_rule(
        &RuleChange::Create(RuleDraft {
            name: "Study rule".into(),
            enabled: true,
            definition: RuleDefinition {
                trigger: RuleTrigger {
                    kind,
                    value: value.map(str::to_owned),
                },
                actions: vec![RuleAction {
                    kind: ActionKind::AddTag,
                    value: "Never applied".into(),
                }],
            },
        }),
        &Cancellation::default(),
    )
    .unwrap();
    db.automation_rules(&RuleQuery::default(), governor, &Cancellation::default())
        .unwrap()
        .entries
        .pop()
        .unwrap()
}
fn run(db: &Database, request: SimulationRequest) -> SimulationOutcome {
    let mut job = SimulationJob::start(request, db.automation_reader().unwrap()).unwrap();
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(10)).unwrap();
    drop(select);
    job.poll().unwrap().outcome
}
fn report(outcome: SimulationOutcome) -> SimulationReport {
    match outcome {
        SimulationOutcome::Complete(report) => report,
        SimulationOutcome::Failed { error, .. } => panic!("{error}"),
        _ => panic!("Simulation did not complete"),
    }
}
pub(crate) fn verify(root: &Path, governor: &MemoryGovernor) {
    let path = root.join("rule-simulation.sqlite3");
    let mut db = Database::open(&path).unwrap();
    let sql = Connection::open(&path).unwrap();
    let mut bytes = vec![b'A'; 70000];
    let mut originals = Vec::new();
    for index in 0..3 {
        *bytes.last_mut().unwrap() = if index == 2 { b'B' } else { b'A' };
        let path = root.join(format!("simulation-{index}.pdf"));
        fs::write(&path, &bytes).unwrap();
        let path = path.canonicalize().unwrap();
        let stamp =
            chrono::DateTime::<chrono::Utc>::from(path.metadata().unwrap().modified().unwrap())
                .to_rfc3339();
        let fingerprint = identity::legacy_fingerprint(&path).unwrap();
        sql.execute("INSERT INTO documents(id,path,normalized_path,file_name,title,author,author_override,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,is_missing) VALUES(?1,?2,?3,?4,?5,'Inherited',?6,2,70000,?7,?7,?7,?8,0)", params![format!("book-{index}"),path.to_string_lossy(),if index==2 {"c:/study-other/c.pdf".to_owned()}else{format!("c:/study/book-{index}.pdf")},format!("book-{index}.PDF"),format!("Book {index}"),if index==0 {Some("ÁUTOR")}else{None},stamp,fingerprint]).unwrap();
        originals.push((path, bytes.clone()));
    }
    assert_eq!(
        identity::legacy_fingerprint(&originals[0].0).unwrap(),
        identity::legacy_fingerprint(&originals[2].0).unwrap()
    );
    sql.execute_batch("INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES('tag-id','Study','study','blue','',''),('study','Different','different','blue','',''); INSERT INTO document_tags(document_id,tag_id) VALUES('book-0','tag-id'),('book-1','study'); INSERT INTO collections(id,name,normalized_name,created_at,updated_at) VALUES('collection-id','Reference','reference','',''); INSERT INTO collection_documents(collection_id,document_id,added_at) VALUES('collection-id','book-2','');").unwrap();
    for (kind, value, ids) in [
        (TriggerKind::DocumentImported, None, vec![]),
        (
            TriggerKind::Extension,
            Some(" .PDF "),
            vec!["book-0", "book-1", "book-2"],
        ),
        (
            TriggerKind::SourceFolder,
            Some("C:\\STUDY\\"),
            vec!["book-0", "book-1"],
        ),
        (TriggerKind::Author, Some("áutor"), vec!["book-0"]),
        (TriggerKind::Tag, Some("tag-id"), vec!["book-0"]),
        (TriggerKind::Tag, Some("study"), vec!["book-1"]),
        (TriggerKind::Tag, Some("STUDY"), vec!["book-0"]),
        (TriggerKind::Collection, Some("reference"), vec!["book-2"]),
        (TriggerKind::Inaccessible, None, vec![]),
        (TriggerKind::ExactDuplicate, None, vec!["book-0", "book-1"]),
    ] {
        let rule = saved(&mut db, governor, kind, value);
        let before: i64 = sql
            .query_row("PRAGMA data_version", [], |row| row.get(0))
            .unwrap();
        let result = report(run(&db, SimulationRequest::new(rule, governor.clone())));
        assert_eq!(result.progress.scanned, 3);
        assert_eq!(result.progress.unverified, 0);
        assert_eq!(
            result
                .matches
                .iter()
                .map(|entry| entry.document.as_str())
                .collect::<Vec<_>>(),
            ids,
            "{kind:?}"
        );
        assert_eq!(
            sql.query_row("PRAGMA data_version", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            before,
            "Simulation wrote to the library"
        );
        if kind == TriggerKind::ExactDuplicate {
            assert!(result.matches.iter().all(|entry| entry.proof.is_some()));
            assert_eq!(
                result.matches[0].proof.as_ref().unwrap().digest,
                result.matches[1].proof.as_ref().unwrap().digest
            );
        }
        drop(result);
        assert_eq!(governor.usage().total, 0);
    }
    let rule = saved(&mut db, governor, TriggerKind::Extension, Some(".pdf"));
    let reader = db.automation_reader().unwrap()().unwrap();
    let cancel = Cancellation::default();
    let bound = reader.begin(&rule, governor, &cancel).unwrap();
    sql.execute_batch("UPDATE documents SET title_override='Changed later',is_missing=1 WHERE id='book-0'; UPDATE automation_rules SET name='Changed later'").unwrap();
    let entries = reader
        .page(
            None,
            bound.value.as_deref().unwrap(),
            &rule.definition().unwrap().trigger,
            governor,
            &cancel,
        )
        .unwrap();
    assert_eq!(entries[0].title, "Book 0");
    assert!(!entries[0].missing);
    drop((entries, bound, reader));
    assert!(matches!(
        run(&db, SimulationRequest::new(rule.clone(), governor.clone())),
        SimulationOutcome::Failed {
            error: Failure::Invalid(_),
            ..
        }
    ));
    let request = SimulationRequest::new(rule.clone(), governor.clone());
    request.cancel.cancel();
    assert!(matches!(
        SimulationJob::start(request, db.automation_reader().unwrap()),
        Err(Failure::Cancelled)
    ));
    drop(rule);
    let rule = saved(&mut db, governor, TriggerKind::ExactDuplicate, None);
    let result = report(run(&db, SimulationRequest::new(rule, governor.clone())));
    assert_eq!(result.progress.unverified, 1);
    assert!(result.matches.is_empty());
    drop(result);
    assert_eq!(governor.usage().total, 0);
    for (path, bytes) in originals {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}
