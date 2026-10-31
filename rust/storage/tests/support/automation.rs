use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, LibraryRepository,
    automation::*,
    memory::{GlobalMemoryLimits, MemoryGovernor},
};
use yamantaka_storage::Database;
fn saved(db: &Database, governor: &MemoryGovernor, id: &str) -> Arc<SavedRule> {
    db.automation_rules(&RuleQuery::default(), governor, &Cancellation::default())
        .unwrap()
        .entries
        .into_iter()
        .find(|rule| rule.revision.id == id)
        .unwrap()
}
pub fn verify() {
    let mut db = Database::in_memory_at(11).unwrap();
    let governor = MemoryGovernor::new(Default::default());
    let cancel = Cancellation::default();
    let draft = RuleDraft {
        name: "  Reading PDFs  ".into(),
        enabled: true,
        definition: RuleDefinition {
            trigger: RuleTrigger {
                kind: TriggerKind::Extension,
                value: Some(".pdf".into()),
            },
            actions: vec![RuleAction {
                kind: ActionKind::AddTag,
                value: "Study".into(),
            }],
        },
    };
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(
        db.change_automation_rule(&RuleChange::Create(draft.clone()), &cancelled)
            .is_err()
    );
    let mut invalid = draft.clone();
    invalid.definition.actions.clear();
    assert!(
        db.change_automation_rule(&RuleChange::Create(invalid), &cancel)
            .is_err()
    );
    let id = db
        .change_automation_rule(&RuleChange::Create(draft.clone()), &cancel)
        .unwrap();
    let first = saved(&db, &governor, &id);
    assert_eq!(first.name, "Reading PDFs");
    assert_eq!(
        first.definition().unwrap().trigger.kind,
        TriggerKind::Extension
    );
    db.change_automation_rule(
        &RuleChange::SetEnabled {
            expected: first.clone(),
            enabled: false,
        },
        &cancel,
    )
    .unwrap();
    assert!(
        db.change_automation_rule(
            &RuleChange::SetEnabled {
                expected: first,
                enabled: true
            },
            &cancel
        )
        .is_err()
    );
    let second = saved(&db, &governor, &id);
    assert!(!second.enabled);
    db.execute_batch(r#"UPDATE automation_rules SET actions_json='[{"kind":"addTag","value":"Changed"}]' WHERE name='Reading PDFs'"#).unwrap();
    assert!(
        db.change_automation_rule(
            &RuleChange::SetEnabled {
                expected: second,
                enabled: true
            },
            &cancel
        )
        .is_err()
    );
    let current = saved(&db, &governor, &id);
    assert_eq!(current.definition().unwrap().actions[0].value, "Changed");
    db.execute_batch(r#"CREATE TEMP TRIGGER reject_rule_delete BEFORE DELETE ON automation_rules BEGIN SELECT RAISE(ABORT,'synthetic rule failure'); END;"#).unwrap();
    assert!(
        db.change_automation_rule(
            &RuleChange::Delete {
                expected: current.clone()
            },
            &cancel
        )
        .is_err()
    );
    assert_eq!(saved(&db, &governor, &id).name, "Reading PDFs");
    db.execute_batch(r#"DROP TRIGGER reject_rule_delete; INSERT INTO automation_rules(id,name,enabled,trigger_json,actions_json,created_at,updated_at) VALUES('future-rule','Future rule',1,' {"kind":"futureTrigger","vendor":7} ',' [ {"kind":"futureAction","value":"Keep bytes"} ] ','2026-10-31','2026-10-31'); CREATE TEMP TRIGGER keep_future_payload BEFORE UPDATE ON automation_rules WHEN OLD.id='future-rule' AND (NEW.trigger_json<>OLD.trigger_json OR NEW.actions_json<>OLD.actions_json OR NEW.created_at<>OLD.created_at) BEGIN SELECT RAISE(ABORT,'future data was rewritten'); END;"#).unwrap();
    let future = saved(&db, &governor, "future-rule");
    assert!(future.definition().is_none());
    db.change_automation_rule(
        &RuleChange::SetEnabled {
            expected: future,
            enabled: false,
        },
        &cancel,
    )
    .unwrap();
    let future = saved(&db, &governor, "future-rule");
    assert!(!future.enabled);
    assert!(future.definition().is_none());
    assert!(
        db.change_automation_rule(
            &RuleChange::SetEnabled {
                expected: future.clone(),
                enabled: true
            },
            &cancel
        )
        .is_err()
    );
    assert!(
        db.change_automation_rule(&RuleChange::Delete { expected: future }, &cancelled)
            .is_err()
    );
    db.change_automation_rule(&RuleChange::Delete { expected: current }, &cancel)
        .unwrap();
    assert_eq!(
        db.automation_rules(&RuleQuery::default(), &governor, &cancel)
            .unwrap()
            .total,
        1
    );
    let tiny = MemoryGovernor::new(GlobalMemoryLimits {
        total: 1,
        ..Default::default()
    });
    assert!(
        db.automation_rules(&RuleQuery::default(), &tiny, &cancel)
            .is_err()
    );
    assert_eq!(tiny.usage().total, 0);
    let lifetime = MemoryGovernor::new(Default::default());
    let page = db
        .automation_rules(&RuleQuery::default(), &lifetime, &cancel)
        .unwrap();
    assert!(lifetime.usage().other > 0);
    let retained = page.entries[0].clone();
    drop(page);
    assert!(lifetime.usage().other > 0);
    drop(retained);
    assert_eq!(lifetime.usage().total, 0);
    let mut many = draft.clone();
    many.definition.actions = (0..8)
        .map(|index| RuleAction {
            kind: ActionKind::AddTag,
            value: format!("Study {index}"),
        })
        .collect();
    let many_id = db
        .change_automation_rule(&RuleChange::Create(many), &cancel)
        .unwrap();
    assert_eq!(
        saved(&db, &governor, &many_id)
            .definition()
            .unwrap()
            .actions
            .len(),
        8
    );
    assert!(
        RuleDefinition::decode(
            r#"{"kind":"extension","value":".pdf","futureCondition":true}"#,
            r#"[{"kind":"addTag","value":"Study"}]"#
        )
        .is_none()
    );
    let mut long = draft.clone();
    long.name = "Long inherited values".into();
    long.definition.trigger.kind = TriggerKind::SourceFolder;
    long.definition.trigger.value = Some("folder/".repeat(12000));
    long.definition.actions[0].value = "Study ".repeat(12000);
    let long_id = db
        .change_automation_rule(&RuleChange::Create(long), &cancel)
        .unwrap();
    let long = saved(&db, &governor, &long_id);
    assert_eq!(
        long.definition()
            .unwrap()
            .trigger
            .value
            .as_ref()
            .unwrap()
            .len(),
        84000
    );
    assert_eq!(long.definition().unwrap().actions[0].value.len(), 72000);
    db.change_automation_rule(&RuleChange::Delete { expected: long }, &cancel)
        .unwrap();
    assert!(
        RuleDefinition::decode(
            r#"{"kind":"author","value":null}"#,
            r#"[{"kind":"addTag","value":"Study"}]"#
        )
        .is_some()
    );
    db.execute_batch(r#"WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<79) INSERT INTO automation_rules(id,name,enabled,trigger_json,actions_json,created_at,updated_at) SELECT printf('page-rule-%03d',x),'Paged rule',1,'{"kind":"inaccessible","value":null}','[{"kind":"setPriority","value":"true"}]','2026-10-31','2026-10-31' FROM n;"#).unwrap();
    let page = db
        .automation_rules(&RuleQuery::default(), &governor, &cancel)
        .unwrap();
    assert_eq!(page.total, 82);
    assert_eq!(page.entries.len(), 80);
    let tail = db
        .automation_rules(&RuleQuery { offset: 80 }, &governor, &cancel)
        .unwrap();
    assert_eq!(tail.total, 82);
    assert_eq!(tail.entries.len(), 2);
    db.integrity().unwrap();
}
