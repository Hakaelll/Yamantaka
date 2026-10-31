use super::*;
use crate::automation_simulation::{SimulationJob, SimulationRequest};
use rusqlite::Connection;
use std::{path::PathBuf, time::Duration};
use yamantaka_domain::{
    LibraryRepository,
    automation::{RuleQuery, plan::PlannedAction},
};
use yamantaka_storage::Database;

fn simulate(db: &Database, governor: &MemoryGovernor) -> Arc<SimulationReport> {
    let rule = db
        .automation_rules(&RuleQuery::default(), governor, &Cancellation::default())
        .unwrap()
        .entries
        .pop()
        .unwrap();
    let mut job = SimulationJob::start(
        SimulationRequest::new(rule, governor.clone()),
        db.automation_reader().unwrap(),
    )
    .unwrap();
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(10)).unwrap();
    drop(select);
    match job.poll().unwrap().outcome {
        SimulationOutcome::Complete(report) => report,
        _ => panic!("Simulation failed"),
    }
}
fn review(db: &Database, request: PlanRequest) -> Result<Arc<RulePlan>> {
    let mut job = PlanJob::start(request, db.automation_reader().unwrap())?;
    let mut select = Select::new();
    job.register(&mut select);
    select.ready_timeout(Duration::from_secs(10)).unwrap();
    drop(select);
    job.poll().unwrap().result
}
pub(crate) fn verify(
    db: &Database,
    sql: &Connection,
    governor: &MemoryGovernor,
    originals: &[(PathBuf, Vec<u8>)],
) {
    sql.execute_batch("INSERT INTO workspaces(id,name,created_at,updated_at) VALUES('plan-space','Mesa de estúdio','',''); UPDATE automation_rules SET trigger_json='{\"kind\":\"extension\",\"value\":\".pdf\"}',actions_json='[{\"kind\":\"addTag\",\"value\":\"tag-id\"},{\"kind\":\"addTag\",\"value\":\"New study\"},{\"kind\":\"addCollection\",\"value\":\"New reading\"},{\"kind\":\"addCollection\",\"value\":\"NEW READING\"},{\"kind\":\"addWorkspace\",\"value\":\"MESA DE ESTÚDIO\"},{\"kind\":\"addReadingQueue\",\"value\":\"plan-space\"},{\"kind\":\"setReadingState\",\"value\":\"terminado\"},{\"kind\":\"setPriority\",\"value\":\"normal\"}]'").unwrap();
    let report = simulate(db, governor);
    let before: i64 = sql
        .query_row("PRAGMA data_version", [], |row| row.get(0))
        .unwrap();
    let plan = review(db, PlanRequest::new(report.clone(), governor.clone())).unwrap();
    assert_eq!(plan.document_revisions().len(), 3);
    assert_eq!(plan.actions().len(), 8);
    let PlannedAction::AddTag(tag) = &plan.actions()[0] else {
        panic!("Wrong action")
    };
    assert_eq!(tag.id, "tag-id");
    assert!(!tag.will_create());
    let PlannedAction::AddTag(tag) = &plan.actions()[1] else {
        panic!("Wrong action")
    };
    assert!(tag.will_create());
    let PlannedAction::AddCollection(first) = &plan.actions()[2] else {
        panic!("Wrong action")
    };
    let PlannedAction::AddCollection(second) = &plan.actions()[3] else {
        panic!("Wrong action")
    };
    assert_eq!(first.id, second.id);
    assert!(first.will_create());
    let PlannedAction::AddWorkspace(workspace) = &plan.actions()[4] else {
        panic!("Wrong action")
    };
    assert_eq!(workspace.id, "plan-space");
    assert!(matches!(
        plan.actions()[6],
        PlannedAction::SetReadingState { finished: true }
    ));
    assert!(matches!(
        plan.actions()[7],
        PlannedAction::SetFavorite(false)
    ));
    assert_eq!(
        sql.query_row("PRAGMA data_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        before
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
    crate::automation_execution::verification_test::verify(db, plan.clone(), governor);
    let request = PlanRequest::new(report.clone(), governor.clone());
    request.cancel.cancel();
    assert!(matches!(review(db, request), Err(Failure::Cancelled)));
    let tiny = MemoryGovernor::new(yamantaka_domain::memory::GlobalMemoryLimits {
        total: 1,
        ..Default::default()
    });
    assert!(matches!(
        review(db, PlanRequest::new(report.clone(), tiny.clone())),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(tiny.usage().total, 0);
    sql.execute_batch(
        "UPDATE documents SET author_override='Changed since simulation' WHERE id='book-1'",
    )
    .unwrap();
    assert!(matches!(
        review(db, PlanRequest::new(report.clone(), governor.clone())),
        Err(Failure::Invalid(_))
    ));
    assert_eq!(plan.report().matches[0].title, "Book 0");
    drop((plan, report));
    assert_eq!(governor.usage().total, 0);
    sql.execute_batch("UPDATE documents SET author_override=NULL WHERE id='book-1'; UPDATE automation_rules SET actions_json='[{\"kind\":\"addWorkspace\",\"value\":\"Missing space\"}]'").unwrap();
    assert!(matches!(
        review(
            db,
            PlanRequest::new(simulate(db, governor), governor.clone())
        ),
        Err(Failure::Invalid(_))
    ));
    sql.execute_batch("UPDATE automation_rules SET actions_json='[{\"kind\":\"setReadingState\",\"value\":\"unsupported\"}]'").unwrap();
    assert!(matches!(
        review(
            db,
            PlanRequest::new(simulate(db, governor), governor.clone())
        ),
        Err(Failure::Invalid(_))
    ));
    sql.execute_batch("UPDATE automation_rules SET trigger_json='{\"kind\":\"exactDuplicate\",\"value\":null}',actions_json='[{\"kind\":\"addTag\",\"value\":\"Study\"}]'").unwrap();
    let report = simulate(db, governor);
    let mut changed = originals[1].1.clone();
    *changed.last_mut().unwrap() = b'C';
    std::fs::write(&originals[1].0, &changed).unwrap();
    assert!(matches!(
        review(db, PlanRequest::new(report, governor.clone())),
        Err(Failure::Invalid(_))
    ));
    std::fs::write(&originals[1].0, &originals[1].1).unwrap();
    let stamp = chrono::DateTime::<chrono::Utc>::from(
        originals[1].0.metadata().unwrap().modified().unwrap(),
    )
    .to_rfc3339();
    sql.execute(
        "UPDATE documents SET file_modified_at=? WHERE id='book-1'",
        [stamp],
    )
    .unwrap();
    assert_eq!(governor.usage().total, 0);
}
