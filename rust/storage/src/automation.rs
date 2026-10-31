mod execution;
mod plan;
mod simulation;
use crate::{Database, database::storage};
use rusqlite::{Connection, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Failure, MAX_LIBRARY_PAGE, Result,
    automation::*,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
};
const SELECT: &str =
    "SELECT id,name,enabled,trigger_json,actions_json,created_at,updated_at FROM automation_rules";
fn field<'a>(row: &'a rusqlite::Row<'_>, index: usize) -> Result<&'a str> {
    row.get_ref(index)
        .map_err(storage)?
        .as_str()
        .map_err(storage)
}
fn hash(row: &rusqlite::Row<'_>) -> Result<[u8; 32]> {
    let mut hash = Sha256::new();
    for index in [0, 1, 3, 4, 5, 6] {
        let value = field(row, index)?;
        hash.update((value.len() as u64).to_le_bytes());
        hash.update(value.as_bytes());
    }
    hash.update([u8::from(row.get::<_, bool>(2).map_err(storage)?)]);
    Ok(hash.finalize().into())
}
fn verify(db: &Connection, expected: &SavedRule) -> Result<()> {
    let mut statement = db
        .prepare(&format!("{SELECT} WHERE id=?"))
        .map_err(storage)?;
    let mut rows = statement.query([&expected.revision.id]).map_err(storage)?;
    let current = rows.next().map_err(storage)?.map(hash).transpose()?;
    if current != Some(expected.revision.hash) {
        return Err(Failure::Invalid(
            "Rule changed; reload before retrying".into(),
        ));
    }
    Ok(())
}
fn next_revision(previous: &str) -> String {
    let mut now = chrono::Utc::now();
    if let Ok(previous) = chrono::DateTime::parse_from_rfc3339(previous)
        && now <= previous
    {
        now = previous.with_timezone(&chrono::Utc) + chrono::Duration::nanoseconds(1);
    }
    now.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}
fn decode(row: &rusqlite::Row<'_>, governor: &MemoryGovernor) -> Result<Arc<SavedRule>> {
    let trigger = field(row, 3)?;
    let actions = field(row, 4)?;
    let mut metadata = 0usize;
    for index in [0, 1, 5, 6] {
        metadata = metadata
            .checked_add(field(row, index)?.len())
            .ok_or(Failure::MemoryBudget)?;
    }
    let parse_bytes = trigger
        .len()
        .checked_add(actions.len())
        .and_then(|size| size.checked_mul(3))
        .ok_or(Failure::MemoryBudget)?;
    let bytes = metadata
        .checked_add(parse_bytes)
        .and_then(|size| {
            size.checked_add(
                std::mem::size_of::<SavedRule>() + 8 * std::mem::size_of::<RuleAction>() + 128,
            )
        })
        .ok_or(Failure::MemoryBudget)?;
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Other,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let content = RuleDefinition::decode(trigger, actions)
        .map_or(RuleContent::Unavailable, RuleContent::Supported);
    let saved = SavedRule::new(
        RuleRevision {
            id: field(row, 0)?.into(),
            hash: hash(row)?,
        },
        field(row, 1)?.into(),
        row.get(2).map_err(storage)?,
        field(row, 5)?.into(),
        field(row, 6)?.into(),
        content,
        lease,
    );
    Ok(Arc::new(saved))
}
impl Database {
    pub(crate) fn rule_page(
        &self,
        query: &RuleQuery,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<RulePage> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let total = tx
            .query_row("SELECT count(*) FROM automation_rules", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(storage)?;
        let entries = {
            let mut statement = tx
                .prepare(&format!(
                    "{SELECT} ORDER BY updated_at DESC,id LIMIT ? OFFSET ?"
                ))
                .map_err(storage)?;
            let offset = i64::try_from(query.offset)
                .map_err(|_| Failure::Invalid("Rule offset is too large".into()))?;
            let mut rows = statement
                .query(params![MAX_LIBRARY_PAGE as i64, offset])
                .map_err(storage)?;
            let mut entries = Vec::new();
            while let Some(row) = rows.next().map_err(storage)? {
                cancel.check()?;
                let saved = decode(row, governor)?;
                entries.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
                entries.push(saved);
            }
            entries
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(RulePage {
            entries,
            total: total.max(0) as u64,
        })
    }
    pub(crate) fn apply_rule(
        &mut self,
        change: &RuleChange,
        cancel: &Cancellation,
    ) -> Result<String> {
        cancel.check()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let id = match change {
            RuleChange::Create(draft) => {
                draft.validate()?;
                let id = uuid::Uuid::new_v4().to_string();
                let now = next_revision("");
                let trigger = serde_json::to_string(&draft.definition.trigger).map_err(storage)?;
                let actions = serde_json::to_string(&draft.definition.actions).map_err(storage)?;
                tx.execute("INSERT INTO automation_rules(id,name,enabled,trigger_json,actions_json,created_at,updated_at) VALUES(?,?,?,?,?,?,?)",params![id,draft.name.trim(),draft.enabled,trigger,actions,now,now]).map_err(storage)?;
                id
            }
            RuleChange::SetEnabled { expected, enabled } => {
                verify(&tx, expected)?;
                if *enabled && expected.definition().is_none() {
                    return Err(Failure::Invalid(
                        "This saved rule format is unavailable; keep it disabled".into(),
                    ));
                }
                tx.execute(
                    "UPDATE automation_rules SET enabled=?,updated_at=? WHERE id=?",
                    params![
                        enabled,
                        next_revision(&expected.updated_at),
                        expected.revision.id
                    ],
                )
                .map_err(storage)?;
                expected.revision.id.clone()
            }
            RuleChange::Delete { expected } => {
                verify(&tx, expected)?;
                tx.execute(
                    "DELETE FROM automation_rules WHERE id=?",
                    [&expected.revision.id],
                )
                .map_err(storage)?;
                expected.revision.id.clone()
            }
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(id)
    }
}
