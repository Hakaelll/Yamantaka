use super::*;
use rusqlite::OptionalExtension;
use std::time::{Duration, Instant};
use yamantaka_domain::automation::{execution::*, plan::*};
mod changes;
mod inherited;
mod journal;
mod undo;
mod undo_changes;
use changes::*;
use undo_changes::revert;
fn invalid(message: &str) -> Failure {
    Failure::Invalid(message.into())
}
fn stamp(previous: &str) -> String {
    super::next_revision(previous)
}
fn item_budget(
    plan: &RulePlan,
    context: &ExecutionContext,
) -> Result<yamantaka_domain::memory::ByteLease> {
    let longest = plan
        .report()
        .matches
        .iter()
        .map(|matched| matched.document.len())
        .max()
        .unwrap_or(0);
    let destinations = plan
        .actions()
        .iter()
        .filter_map(target)
        .try_fold(0usize, |sum, (_, target)| {
            sum.checked_add(target.id.len())
                .and_then(|sum| sum.checked_add(target.name.len()))
        })
        .ok_or(Failure::MemoryBudget)?;
    let rule_bytes = plan
        .report()
        .rule
        .revision
        .id
        .len()
        .checked_add(plan.report().rule.name.len())
        .and_then(|bytes| bytes.checked_mul(16))
        .ok_or(Failure::MemoryBudget)?;
    let bytes = longest
        .checked_add(destinations)
        .and_then(|bytes| bytes.checked_mul(128))
        .and_then(|bytes| bytes.checked_add(rule_bytes))
        .and_then(|bytes| bytes.checked_add(65536))
        .ok_or(Failure::MemoryBudget)?;
    super::plan::reserve(&context.governor, bytes)
}
impl RuleExecutionStore for Database {
    fn apply(
        &mut self,
        plan: &RulePlan,
        context: &ExecutionContext,
        verify_source: &mut SourceVerifier<'_>,
    ) -> Result<Arc<JournalEntry>> {
        context.cancel.check()?;
        if plan.document_revisions().len() != plan.report().matches.len()
            || plan.report().matches.is_empty()
        {
            return Err(invalid("The reviewed plan is empty or incomplete"));
        }
        journal::indexes(&self.connection)?;
        let _lease = item_budget(plan, context)?;
        let start = Instant::now();
        let (root, mut guards) = {
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage)?;
            verify(&tx, &plan.report().rule)?;
            let (guards, undo) = targets(&tx, plan)?;
            let root = journal::start(&tx, plan)?;
            journal::child(&tx, &root, 0, &undo)?;
            context.cancel.check()?;
            tx.commit().map_err(storage)?;
            (root, guards)
        };
        let apply = (|| -> Result<()> {
            for index in 0..plan.report().matches.len() {
                context.cancel.check()?;
                if start.elapsed()
                    >= Duration::from_secs(
                        yamantaka_domain::automation::simulation::MAX_SIMULATION_SECONDS,
                    )
                {
                    return Err(invalid("Rule execution time limit reached"));
                }
                let _source_guard = verify_source(&plan.report().matches[index])?;
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)
                    .map_err(storage)?;
                let active:bool=tx.query_row("SELECT status='running' FROM operation_journal WHERE id=? AND operation_kind=?",params![root,journal::ROOT],|row|row.get(0)).map_err(storage)?;
                if !active {
                    return Err(invalid("The execution was changed by another request"));
                }
                verify(&tx, &plan.report().rule)?;
                verify_targets(&tx, &guards)?;
                let _document_lease =
                    document_budget(&tx, &plan.report().matches[index].document, context)?;
                let undo = apply_document(&tx, plan, index, context)?;
                journal::child(&tx, &root, index + 1, &undo)?;
                journal::update(
                    &tx,
                    &root,
                    "running",
                    "applying",
                    Some(index + 1),
                    None,
                    true,
                )?;
                context.cancel.check()?;
                refresh_guards(&tx, &mut guards)?;
                tx.commit().map_err(storage)?;
                context.watch.publish((index + 1) as u64);
            }
            context.cancel.check()?;
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage)?;
            journal::update(&tx, &root, "completed", "completed", None, None, true)?;
            context.cancel.check()?;
            tx.commit().map_err(storage)?;
            Ok(())
        })();
        if let Err(error) = apply {
            let cancelled = matches!(error, Failure::Cancelled);
            let recovery = ExecutionContext {
                governor: context.governor.clone(),
                cancel: Cancellation::default(),
                watch: context.watch.clone(),
                access: context.access,
            };
            undo::rollback(
                &mut self.connection,
                &root,
                &recovery,
                if cancelled { "cancelled" } else { "failed" },
                Some(if cancelled {
                    "user_cancelled"
                } else {
                    "apply_failed"
                }),
            )?;
        }
        journal::entry(&self.connection, &root, &context.governor)
    }
    fn undo(
        &mut self,
        revision: &JournalRevision,
        context: &ExecutionContext,
    ) -> Result<Arc<JournalEntry>> {
        context.cancel.check()?;
        journal::indexes(&self.connection)?;
        {
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage)?;
            journal::verify_revision(&tx, revision)?;
            let compatible:bool=tx.query_row("SELECT compatibility_hash='yamantaka-rules-v1' AND resume_capability='rollback' FROM operation_journal WHERE id=? AND operation_kind=?",params![revision.id,journal::ROOT],|row|row.get(0)).map_err(storage)?;
            if !compatible || uuid::Uuid::parse_str(&revision.id).is_err() {
                return Err(invalid("This execution cannot be undone by this version"));
            }
            journal::update(
                &tx,
                &revision.id,
                "interrupted",
                "undoing",
                None,
                None,
                true,
            )?;
            tx.commit().map_err(storage)?;
        }
        undo::rollback(
            &mut self.connection,
            &revision.id,
            context,
            "rolledBack",
            None,
        )?;
        journal::entry(&self.connection, &revision.id, &context.governor)
    }
}
