use super::*;
use rusqlite::types::Value;

struct Rows {
    columns: Vec<String>,
    rows: Vec<Vec<Value>>,
}
pub(crate) struct WorkspaceUndoData {
    receipt: WorkspaceUndo,
    header: Rows,
    tabs: Rows,
}
fn rows(
    db: &Connection,
    table: &str,
    column: &str,
    id: &str,
    budget: &mut usize,
    cancel: &Cancellation,
) -> Result<Rows> {
    let mut statement = db
        .prepare(&format!("SELECT * FROM {table} WHERE {column}=?"))
        .map_err(storage)?;
    let columns = statement
        .column_names()
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    let mut query = statement.query([id]).map_err(storage)?;
    let mut rows = Vec::new();
    while let Some(row) = query.next().map_err(storage)? {
        cancel.check()?;
        let values = (0..columns.len())
            .map(|index| row.get::<_, Value>(index).map_err(storage))
            .collect::<Result<Vec<_>>>()?;
        let bytes = values
            .iter()
            .map(|value| match value {
                Value::Text(text) => text.len(),
                Value::Blob(bytes) => bytes.len(),
                _ => 8,
            })
            .sum::<usize>();
        *budget = budget.checked_sub(bytes).ok_or_else(|| {
            invalid("Workspace is too large for deletion undo; nothing was deleted")
        })?;
        if rows.len() >= 100_000 {
            return Err(invalid(
                "Workspace exceeds the deletion undo row limit; nothing was deleted",
            ));
        }
        rows.push(values);
    }
    Ok(Rows { columns, rows })
}
fn insert(db: &Connection, table: &str, rows: &Rows, cancel: &Cancellation) -> Result<()> {
    let columns = rows
        .columns
        .iter()
        .map(|name| format!("\"{}\"", name.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(",");
    let placeholders = vec!["?"; rows.columns.len()].join(",");
    let mut statement = db
        .prepare(&format!(
            "INSERT INTO {table}({columns}) VALUES({placeholders})"
        ))
        .map_err(storage)?;
    for row in &rows.rows {
        cancel.check()?;
        statement
            .execute(rusqlite::params_from_iter(row))
            .map_err(storage)?;
    }
    Ok(())
}
impl WorkspaceUndoData {
    pub(super) fn capture(
        db: &Connection,
        workspace: &Workspace,
        cancel: &Cancellation,
    ) -> Result<Self> {
        let mut budget = 8 * 1024 * 1024;
        Ok(Self {
            receipt: WorkspaceUndo {
                id: workspace.id.clone(),
                token: uuid::Uuid::new_v4().to_string(),
                name: workspace.name.clone(),
            },
            header: rows(db, "workspaces", "id", &workspace.id, &mut budget, cancel)?,
            tabs: rows(
                db,
                "workspace_tabs",
                "workspace_id",
                &workspace.id,
                &mut budget,
                cancel,
            )?,
        })
    }
    pub(super) fn receipt(&self) -> WorkspaceUndo {
        self.receipt.clone()
    }
}
impl Database {
    pub(crate) fn undo_workspace(
        &mut self,
        token: &str,
        cancel: &Cancellation,
    ) -> Result<WorkspaceChanged> {
        cancel.check()?;
        let undo = self
            .workspace_undo
            .as_ref()
            .filter(|undo| undo.receipt.token == token)
            .ok_or_else(|| invalid("The last workspace deletion can no longer be undone"))?;
        let tx = self
            .connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage)?;
        if tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM workspaces WHERE id=?)",
                [&undo.receipt.id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(storage)?
        {
            return Err(invalid(
                "A workspace with this identity already exists; nothing was replaced",
            ));
        }
        insert(&tx, "workspaces", &undo.header, cancel)?;
        insert(&tx, "workspace_tabs", &undo.tabs, cancel)?;
        let original = header(&tx, &undo.receipt.id)?;
        if original.is_default {
            tx.execute(
                "UPDATE workspaces SET is_default=0 WHERE is_default=1 AND id<>?",
                [&original.id],
            )
            .map_err(storage)?;
        }
        tx.execute(
            "UPDATE workspaces SET updated_at=? WHERE id=?",
            params![next_revision(Some(&original.updated_at)), original.id],
        )
        .map_err(storage)?;
        let restored = header(&tx, &original.id)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        self.workspace_undo = None;
        Ok(WorkspaceChanged::Saved(restored))
    }
}
