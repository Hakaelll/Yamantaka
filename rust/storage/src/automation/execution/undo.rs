use super::*;
pub(super) fn rollback(
    db: &mut Connection,
    root: &str,
    context: &ExecutionContext,
    final_status: &str,
    error: Option<&str>,
) -> Result<()> {
    loop {
        if context.cancel.is_cancelled() {
            journal::update(
                db,
                root,
                "interrupted",
                "undo_cancelled",
                None,
                Some("user_cancelled"),
                true,
            )?;
            return Ok(());
        }
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut query=tx.prepare("SELECT id,undo_json FROM operation_journal WHERE operation_kind=? AND id>=? AND id<? AND status='completed' ORDER BY id DESC LIMIT 1").map_err(storage)?;
        let mut rows = query
            .query(params![
                journal::ITEM,
                format!("{root}:"),
                format!("{root};")
            ])
            .map_err(storage)?;
        let Some(row) = rows.next().map_err(storage)? else {
            drop(rows);
            drop(query);
            journal::update(&tx, root, final_status, "rolled_back", None, error, false)?;
            tx.commit().map_err(storage)?;
            return Ok(());
        };
        let id = super::super::field(row, 0)?;
        let json = super::super::field(row, 1)?;
        let bytes = json
            .len()
            .checked_mul(6)
            .and_then(|bytes| bytes.checked_add(65536))
            .ok_or(Failure::MemoryBudget)?;
        let lease = super::super::plan::reserve(&context.governor, bytes)?;
        let undo: Undo = serde_json::from_str(json).map_err(storage)?;
        let id = id.to_owned();
        drop(rows);
        drop(query);
        if revert(&tx, &undo, context).is_err() {
            drop(tx);
            journal::update(
                db,
                root,
                "failed",
                "rollback_conflict",
                None,
                Some("rollback_conflict"),
                true,
            )?;
            return Ok(());
        }
        tx.execute("UPDATE operation_journal SET status='rolledBack',phase='rolled_back',updated_at=? WHERE id=? AND operation_kind=?",params![stamp(""),id,journal::ITEM]).map_err(storage)?;
        journal::update(&tx, root, "interrupted", "undoing", None, error, true)?;
        tx.commit().map_err(storage)?;
        drop((undo, lease));
    }
}
