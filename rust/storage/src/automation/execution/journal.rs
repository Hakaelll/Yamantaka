use super::*;

pub(super) const ROOT: &str = "rustAutomation.v1";
pub(super) const LEGACY: &str = "automation";
pub(super) const ITEM: &str = "rustAutomationItem.v1";
pub(super) fn entry(
    db: &Connection,
    id: &str,
    governor: &MemoryGovernor,
) -> Result<Arc<JournalEntry>> {
    let mut statement=db.prepare("SELECT id,updated_at,status,completed_items,total_items,resume_capability,error_code,COALESCE(CASE WHEN json_valid(summary_json) THEN json_extract(summary_json,'$.ruleName') END,(SELECT name FROM automation_rules WHERE id=CASE WHEN json_valid(summary_json) THEN json_extract(summary_json,'$.ruleId') END),'Inherited rule execution'),operation_kind,undo_json IS NOT NULL,compatibility_hash IS NOT NULL FROM operation_journal WHERE id=? AND operation_kind IN (?,?)").map_err(storage)?;
    let mut rows = statement
        .query(params![id, ROOT, LEGACY])
        .map_err(storage)?;
    let row = rows
        .next()
        .map_err(storage)?
        .ok_or_else(|| invalid("Rule execution history no longer exists"))?;
    let name = super::super::field(row, 7)?;
    let updated = super::super::field(row, 1)?;
    let error = match row.get_ref(6).map_err(storage)? {
        rusqlite::types::ValueRef::Null => None,
        value => Some(value.as_str().map_err(storage)?),
    };
    let bytes = id
        .len()
        .checked_add(name.len())
        .and_then(|bytes| bytes.checked_add(updated.len()))
        .and_then(|bytes| bytes.checked_add(error.map_or(0, str::len)))
        .and_then(|bytes| bytes.checked_mul(2))
        .and_then(|bytes| bytes.checked_add(2048))
        .ok_or(Failure::MemoryBudget)?;
    let lease = super::super::plan::reserve(governor, bytes)?;
    let capability = super::super::field(row, 5)?;
    let native = super::super::field(row, 8)? == ROOT;
    let origin = if native {
        JournalOrigin::Native
    } else {
        JournalOrigin::Flutter {
            retry_candidate: matches!(capability, "retry" | "resume")
                && row.get::<_, bool>(10).map_err(storage)?
                && !matches!(super::super::field(row, 2)?, "completed" | "rolledBack"),
            has_undo: capability == "rollback" && row.get::<_, bool>(9).map_err(storage)?,
        }
    };
    let status = match super::super::field(row, 2)? {
        "completed" => ExecutionStatus::Completed,
        "cancelled" => ExecutionStatus::Cancelled,
        "rolledBack" => ExecutionStatus::RolledBack,
        "running" | "planned" | "interrupted" => ExecutionStatus::Interrupted,
        _ => ExecutionStatus::Failed,
    };
    Ok(Arc::new(
        JournalEntry::new(
            JournalRevision {
                id: id.into(),
                updated_at: updated.into(),
                hash: super::super::plan::record(db, "operation_journal", id)?,
            },
            name.into(),
            status,
            (
                u64::try_from(row.get::<_, i64>(3).map_err(storage)?)
                    .map_err(|_| invalid("Invalid journal count"))?,
                u64::try_from(row.get::<_, i64>(4).map_err(storage)?)
                    .map_err(|_| invalid("Invalid journal count"))?,
            ),
            native && capability == "rollback",
            error.map(str::to_owned),
            lease,
        )
        .with_origin(origin),
    ))
}
pub(super) fn verify_revision(db: &Connection, expected: &JournalRevision) -> Result<()> {
    verify_kind(db, expected, ROOT)
}
pub(super) fn verify_kind(db: &Connection, expected: &JournalRevision, kind: &str) -> Result<()> {
    let current: Option<String> = db
        .query_row(
            "SELECT updated_at FROM operation_journal WHERE id=? AND operation_kind=?",
            params![expected.id, kind],
            |row| row.get(0),
        )
        .optional()
        .map_err(storage)?;
    if current.as_deref() != Some(expected.updated_at.as_str())
        || super::super::plan::record(db, "operation_journal", &expected.id)? != expected.hash
    {
        return Err(invalid("Execution history changed; reload before retrying"));
    }
    Ok(())
}
pub(super) fn start(db: &Connection, plan: &RulePlan) -> Result<String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = stamp("");
    let summary = serde_json::json!({"version":1,"ruleId":plan.report().rule.revision.id,"ruleName":plan.report().rule.name});
    db.execute("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,undo_json,started_at,updated_at,phase,resume_capability,compatibility_hash) VALUES(?,?,'running',?,?,0,'{\"version\":1}',?,?,'applying','rollback','yamantaka-rules-v1')",params![id,ROOT,summary.to_string(),plan.report().matches.len() as i64,now,now]).map_err(storage)?;
    Ok(id)
}
pub(super) fn child(db: &Connection, root: &str, index: usize, undo: &Undo) -> Result<()> {
    let now = stamp("");
    let json = serde_json::to_string(undo).map_err(storage)?;
    db.execute("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,undo_json,started_at,updated_at,phase,resume_capability) VALUES(?,?,'completed','{}',1,1,?,?,?,'committed','none')",params![format!("{root}:{index:08}"),ITEM,json,now,now]).map_err(storage)?;
    Ok(())
}
pub(super) fn update(
    db: &Connection,
    id: &str,
    status: &str,
    phase: &str,
    completed: Option<usize>,
    error: Option<&str>,
    reversible: bool,
) -> Result<()> {
    let previous: String = db
        .query_row(
            "SELECT updated_at FROM operation_journal WHERE id=? AND operation_kind=?",
            params![id, ROOT],
            |row| row.get(0),
        )
        .map_err(storage)?;
    db.execute("UPDATE operation_journal SET status=?,phase=?,completed_items=COALESCE(?,completed_items),error_code=?,resume_capability=?,updated_at=? WHERE id=? AND operation_kind=?",params![status,phase,completed.map(|value|value as i64),error,if reversible {"rollback"}else{"none"},stamp(&previous),id,ROOT]).map_err(storage)?;
    Ok(())
}
impl Database {
    pub(crate) fn rule_journals(
        &self,
        offset: usize,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<JournalPage> {
        cancel.check()?;
        let offset =
            i64::try_from(offset).map_err(|_| invalid("Execution history offset is too large"))?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let total: i64 = tx
            .query_row(
                "SELECT count(*) FROM operation_journal WHERE operation_kind IN (?,?)",
                params![ROOT, LEGACY],
                |row| row.get(0),
            )
            .map_err(storage)?;
        let mut entries = Vec::new();
        {
            let mut statement=tx.prepare("SELECT id FROM operation_journal WHERE operation_kind IN (?,?) ORDER BY updated_at DESC,id LIMIT 80 OFFSET ?").map_err(storage)?;
            let mut rows = statement
                .query(params![ROOT, LEGACY, offset])
                .map_err(storage)?;
            while let Some(row) = rows.next().map_err(storage)? {
                cancel.check()?;
                entries.push(entry(&tx, super::super::field(row, 0)?, governor)?);
            }
        }
        tx.commit().map_err(storage)?;
        Ok(JournalPage {
            entries,
            total: u64::try_from(total).map_err(|_| invalid("Invalid journal count"))?,
        })
    }
}

pub(super) fn indexes(db: &Connection) -> Result<()> {
    db.execute_batch("CREATE INDEX IF NOT EXISTS idx_rust_rule_receipts ON operation_journal(operation_kind,status,id); CREATE INDEX IF NOT EXISTS idx_rust_rule_history ON operation_journal(operation_kind,updated_at DESC,id);").map_err(storage)
}
