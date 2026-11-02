use std::{ffi::OsString, path::Path};
mod legacy_recovery;
use yamantaka_domain::LibraryRepository;
use yamantaka_storage::{Database, identity};

pub fn seed(
    db: &Database,
    store: &Path,
    fixture: &Path,
    args: &[OsString],
) -> Result<(), Box<dyn std::error::Error>> {
    if args.iter().any(|arg| arg == "--inherited-journals") {
        use yamantaka_domain::{Cancellation, automation::*, memory::MemoryGovernor};
        let governor = MemoryGovernor::new(Default::default());
        let page =
            db.automation_rules(&RuleQuery::default(), &governor, &Cancellation::default())?;
        let rule = page
            .entries
            .iter()
            .find(|rule| rule.revision.id == "legacy-rule")
            .unwrap();
        let hash = if args.iter().any(|arg| arg == "--inherited-incompatible") {
            String::from("different-rule")
        } else {
            flutter_compatibility_hash(&rule.revision.id, rule.definition().unwrap())?
        };
        db.execute_batch(&format!("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,undo_json,started_at,updated_at,phase,resume_capability,compatibility_hash) VALUES('inherited-0-retry','automation','interrupted','{{\"ruleId\":\"legacy-rule\",\"scanned\":10}}',10,0,NULL,'2026-10-30','2026-10-30','cancelled','retry','{hash}'),('inherited-1-rollback','automation','interrupted','{{\"ruleId\":\"legacy-rule\"}}',10,1,'{{\"demo-0:0:setPriority\":{{\"kind\":\"restoreFavorite\",\"documentId\":\"demo-0\",\"value\":false}}}}','2026-10-30','2026-10-30','applying','rollback','{hash}'),('inherited-2-future','automation','interrupted','{{\"ruleId\":\"future-rule\",\"vendor\":7}}',10,1,'{{\"future\":{{\"kind\":\"futureUndo\",\"keep\":true}}}}','2026-10-30','2026-10-30','applying','rollback',NULL);"))?;
        if args.iter().any(|arg| arg == "--inherited-recovery") {
            legacy_recovery::seed(db, args)?;
        }
        if args.iter().any(|arg| arg == "--inherited-paged") {
            db.execute_batch("WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<78) INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,started_at,updated_at,phase,resume_capability) SELECT printf('older-inherited-%03d',x),'automation','completed','{}',1,1,'2026-10-29','2026-10-29','completed','none' FROM n;")?;
        }
    }
    if args.iter().any(|arg| arg == "--rule-import-context") {
        db.execute_batch("UPDATE automation_rules SET trigger_json='{\"kind\":\"documentImported\",\"value\":null}',actions_json='[{\"kind\":\"setPriority\",\"value\":\"1\"}]' WHERE id='legacy-rule'")?;
    }
    if args.iter().any(|arg| arg == "--rule-review") {
        db.execute_batch("INSERT INTO workspaces(id,name,created_at,updated_at) VALUES('review-workspace','Study desk','','')")?;
        let workspace = if args.iter().any(|arg| arg == "--rule-review-missing") {
            "Missing workspace"
        } else {
            "review-workspace"
        };
        let actions = serde_json::json!([
            {"kind":"addTag","value":"Study notes"},
            {"kind":"addCollection","value":"Reading seminar"},
            {"kind":"addWorkspace","value":workspace},
            {"kind":"addReadingQueue","value":workspace},
            {"kind":"setReadingState","value":"finished"},
            {"kind":"setPriority","value":"normal"},
            {"kind":"addTag","value":"STUDY NOTES"}
        ]);
        db.execute_batch(&format!(
            "UPDATE automation_rules SET actions_json='{}' WHERE id='legacy-rule'",
            serde_json::to_string(&actions)?.replace('\'', "''")
        ))?;
    }
    if args.iter().any(|arg| arg == "--rule-execute-fail") {
        db.execute_batch("CREATE TRIGGER reject_rule_fixture BEFORE INSERT ON workspace_tabs WHEN NEW.document_id='demo-1' BEGIN SELECT RAISE(ABORT,'synthetic queue failure'); END;")?;
    }
    if args
        .iter()
        .any(|arg| arg == "--rule-simulation-duplicates" || arg == "--duplicates")
    {
        db.execute_batch("DELETE FROM documents WHERE id NOT IN ('demo-0','demo-1','demo-2'); UPDATE automation_rules SET trigger_json='{\"kind\":\"exactDuplicate\",\"value\":null}' WHERE id='legacy-rule';")?;
        let mut bytes = std::fs::read(fixture)?;
        bytes.resize(70000, b' ');
        for index in 0..3 {
            *bytes.last_mut().unwrap() = if index == 2 { b'B' } else { b'A' };
            let path = store.join(format!("duplicate-{index}.pdf"));
            std::fs::write(&path, &bytes)?;
            let path = path.canonicalize()?;
            let fingerprint = identity::legacy_fingerprint(&path)?;
            let modified =
                chrono::DateTime::<chrono::Utc>::from(path.metadata()?.modified()?).to_rfc3339();
            let escaped = path.to_string_lossy().replace('\'', "''");
            db.execute_batch(&format!("UPDATE documents SET path='{escaped}',normalized_path='{}',file_size=70000,fingerprint='{fingerprint}',file_modified_at='{modified}',is_missing=0 WHERE id='demo-{index}'", escaped.to_lowercase()))?;
        }
        if args.iter().any(|arg| arg == "--duplicates-empty") {
            db.execute_batch("DELETE FROM documents WHERE id='demo-1'")?;
        }
    } else if args.iter().any(|arg| arg == "--rule-simulation-cancel") {
        db.execute_batch("WITH RECURSIVE n(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM n WHERE x<99999) INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) SELECT printf('scan-%06d',x),printf('scan-%06d.pdf',x),printf('scan-%06d.pdf',x),'scan.pdf',2,70000,'2026-10-31','2026-10-31','2026-10-31','synthetic' FROM n;")?;
    }
    Ok(())
}
