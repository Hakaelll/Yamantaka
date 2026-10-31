use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Summary {
    #[serde(rename = "ruleId")]
    rule_id: String,
}
impl Database {
    pub(crate) fn inherited_rule(
        &self,
        expected: &JournalRevision,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Arc<SavedRule>> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        journal::verify_kind(&tx, expected, journal::LEGACY)?;
        let saved = {
            let mut query=tx.prepare("SELECT summary_json,compatibility_hash,resume_capability,status FROM operation_journal WHERE id=? AND operation_kind=?").map_err(storage)?;
            let mut rows = query
                .query(params![expected.id, journal::LEGACY])
                .map_err(storage)?;
            let row = rows
                .next()
                .map_err(storage)?
                .ok_or_else(|| invalid("Inherited execution no longer exists"))?;
            let field = super::super::field;
            if !matches!(field(row, 2)?, "retry" | "resume")
                || matches!(field(row, 3)?, "completed" | "rolledBack")
            {
                return Err(invalid(
                    "This inherited execution requires receipt recovery instead of retry",
                ));
            }
            let summary = field(row, 0)?;
            let expected_hash = field(row, 1)
                .map_err(|_| invalid("The inherited execution has no compatibility proof"))?;
            let bytes = summary
                .len()
                .checked_mul(3)
                .and_then(|bytes| bytes.checked_add(256))
                .ok_or(Failure::MemoryBudget)?;
            let _lease = super::super::plan::reserve(governor, bytes)?;
            let summary: Summary = serde_json::from_str(summary)
                .map_err(|_| invalid("The inherited journal summary needs review"))?;
            cancel.check()?;
            let mut rules = tx
                .prepare(&format!("{} WHERE id=?", super::super::SELECT))
                .map_err(storage)?;
            let mut found = rules.query([&summary.rule_id]).map_err(storage)?;
            let rule = super::super::decode(
                found
                    .next()
                    .map_err(storage)?
                    .ok_or_else(|| invalid("The linked rule no longer exists"))?,
                governor,
            )?;
            let definition = rule
                .definition()
                .ok_or_else(|| invalid("The linked rule format needs review"))?;
            definition.validate()?;
            if flutter_compatibility_hash(&rule.revision.id, definition)? != expected_hash {
                return Err(invalid(
                    "The linked rule changed since this journal; review the current rule separately",
                ));
            }
            rule
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(saved)
    }
}
