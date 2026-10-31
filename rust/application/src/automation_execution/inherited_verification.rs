use super::*;
use rusqlite::Connection;
use yamantaka_domain::{LibraryRepository, automation::*};
use yamantaka_storage::Database;

pub(crate) fn verify(original: &Database) {
    let path = original
        .file_path()
        .unwrap()
        .with_file_name("inherited-rule-journals.sqlite3");
    let db = Database::open(&path).unwrap();
    let sql = Connection::open(&path).unwrap();
    let governor = MemoryGovernor::new(Default::default());
    let cancel = Cancellation::default();
    let expected = "9ccf9b292f1aaea0691218c3486473ba8b80e7606a1cfec88fc1c9dbe8ef358d";
    let definition = RuleDefinition {
        trigger: RuleTrigger {
            kind: TriggerKind::Extension,
            value: Some(".pdf".into()),
        },
        actions: vec![RuleAction {
            kind: ActionKind::AddTag,
            value: "Study Ω".into(),
        }],
    };
    assert_eq!(
        flutter_compatibility_hash("inherited-rule", &definition).unwrap(),
        expected
    );
    sql.execute_batch("INSERT INTO automation_rules(id,name,enabled,trigger_json,actions_json,created_at,updated_at) VALUES('inherited-rule','Inherited study',1,' { \"value\": \".pdf\", \"kind\": \"extension\" } ','[ {\"value\":\"Study Ω\",\"kind\":\"addTag\"} ]','','');").unwrap();
    sql.execute("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,started_at,updated_at,phase,resume_capability,compatibility_hash) VALUES('legacy-retry','automation','interrupted','{\"ruleId\":\"inherited-rule\",\"scanned\":3}',3,0,'','now','cancelled','retry',?)",[expected]).unwrap();
    sql.execute_batch("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,undo_json,started_at,updated_at,phase,resume_capability) VALUES('legacy-future','automation','interrupted','{\"ruleId\":\"future\"}',3,1,' {\"old\":{\"kind\":\"futureUndo\",\"payload\":[1,2,3]}} ','','now','applying','rollback'),('native-root','rustAutomation.v1','completed','{\"ruleName\":\"Native history\"}',1,1,NULL,'','now','completed','none'),('native-item','rustAutomationItem.v1','completed','{}',1,1,'{}','','now','committed','none');").unwrap();
    let page = db.automation_journals(0, &governor, &cancel).unwrap();
    assert_eq!(page.total, 3);
    let retry = page
        .entries
        .iter()
        .find(|entry| entry.revision.id == "legacy-retry")
        .unwrap();
    assert_eq!(retry.rule_name, "Inherited study");
    assert_eq!(
        retry.origin,
        JournalOrigin::Flutter {
            retry_candidate: true,
            has_undo: false
        }
    );
    assert!(!retry.reversible);
    let future = page
        .entries
        .iter()
        .find(|entry| entry.revision.id == "legacy-future")
        .unwrap();
    assert_eq!(
        future.origin,
        JournalOrigin::Flutter {
            retry_candidate: false,
            has_undo: true
        }
    );
    assert!(!future.reversible);
    let native = page
        .entries
        .iter()
        .find(|entry| entry.revision.id == "native-root")
        .unwrap();
    assert_eq!(native.origin, JournalOrigin::Native);
    assert!(
        db.inherited_automation_rule(&native.revision, &governor, &cancel)
            .is_err()
    );
    let before: i64 = sql
        .query_row("PRAGMA data_version", [], |row| row.get(0))
        .unwrap();
    let linked = db
        .inherited_automation_rule(&retry.revision, &governor, &cancel)
        .unwrap();
    assert_eq!(linked.revision.id, "inherited-rule");
    assert_eq!(linked.definition().unwrap().actions[0].value, "Study Ω");
    assert_eq!(
        sql.query_row("PRAGMA data_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        before
    );
    drop(linked);
    let tiny = MemoryGovernor::new(yamantaka_domain::memory::GlobalMemoryLimits {
        total: 1,
        ..Default::default()
    });
    assert!(matches!(
        db.inherited_automation_rule(&retry.revision, &tiny, &cancel),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(tiny.usage().total, 0);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        db.inherited_automation_rule(&retry.revision, &governor, &cancelled),
        Err(Failure::Cancelled)
    ));
    sql.execute_batch("UPDATE automation_rules SET name='Renamed study',enabled=0,updated_at='later' WHERE id='inherited-rule'").unwrap();
    let linked = db
        .inherited_automation_rule(&retry.revision, &governor, &cancel)
        .unwrap();
    assert!(!linked.enabled);
    assert_eq!(linked.name, "Renamed study");
    drop(linked);
    sql.execute_batch("UPDATE automation_rules SET actions_json='[{\"kind\":\"addTag\",\"value\":\"Different\"}]' WHERE id='inherited-rule'").unwrap();
    assert!(matches!(
        db.inherited_automation_rule(&retry.revision, &governor, &cancel),
        Err(Failure::Invalid(_))
    ));
    sql.execute_batch("UPDATE automation_rules SET actions_json='[{\"kind\":\"addTag\",\"value\":\"Study Ω\"}]'; UPDATE operation_journal SET phase='changed-without-timestamp' WHERE id='legacy-retry'").unwrap();
    assert!(
        db.inherited_automation_rule(&retry.revision, &governor, &cancel)
            .is_err()
    );
    sql.execute_batch("UPDATE operation_journal SET phase='cancelled' WHERE id='legacy-retry'; UPDATE operation_journal SET resume_capability='rollback' WHERE id='legacy-retry'").unwrap();
    let current_page = db.automation_journals(0, &governor, &cancel).unwrap();
    let current = current_page
        .entries
        .iter()
        .find(|entry| entry.revision.id == "legacy-retry")
        .unwrap();
    assert!(
        db.inherited_automation_rule(&current.revision, &governor, &cancel)
            .is_err()
    );
    drop(current_page);
    sql.execute_batch("UPDATE operation_journal SET resume_capability='retry' WHERE id='legacy-retry'; UPDATE automation_rules SET trigger_json='{\"kind\":\"futureTrigger\",\"value\":null}'").unwrap();
    assert!(
        db.inherited_automation_rule(&retry.revision, &governor, &cancel)
            .is_err()
    );
    sql.execute_batch("DELETE FROM automation_rules").unwrap();
    assert!(
        db.inherited_automation_rule(&retry.revision, &governor, &cancel)
            .is_err()
    );
    assert_eq!(
        sql.query_row(
            "SELECT undo_json FROM operation_journal WHERE id='legacy-future'",
            [],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        " {\"old\":{\"kind\":\"futureUndo\",\"payload\":[1,2,3]}} "
    );
    sql.execute_batch("WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<78) INSERT INTO operation_journal(id,operation_kind,status,total_items,completed_items,started_at,updated_at,resume_capability) SELECT printf('older-%03d',x),'automation','completed',1,1,'','older','none' FROM n;").unwrap();
    let first = db.automation_journals(0, &governor, &cancel).unwrap();
    let second = db.automation_journals(80, &governor, &cancel).unwrap();
    assert_eq!(
        (
            first.total,
            first.entries.len(),
            second.total,
            second.entries.len()
        ),
        (82, 80, 82, 2)
    );
    drop((first, second, page));
    assert_eq!(governor.usage().total, 0);
    assert_eq!(
        sql.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}
