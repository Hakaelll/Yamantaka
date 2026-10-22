use crate::{Database, database::storage};
use rusqlite::{Connection, OptionalExtension, params};
use yamantaka_domain::{Cancellation, Failure, MAX_LIBRARY_PAGE, Result, workspaces::*};
mod legacy_comparison;
mod readings;
pub(crate) mod undo;
mod write;

fn invalid(message: &str) -> Failure {
    Failure::Invalid(message.into())
}
fn name(value: &str) -> Result<&str> {
    let value = value.trim();
    if value.is_empty() || value.encode_utf16().count() > 80 {
        return Err(invalid("Use a workspace name between 1 and 80 characters"));
    }
    Ok(value)
}
fn summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
        updated_at: row.get(2)?,
        is_default: row.get(3)?,
        readings: row.get::<_, i64>(4)? as u64,
    })
}
const SUMMARY: &str = "SELECT w.id,w.name,w.updated_at,w.is_default,(SELECT count(*) FROM workspace_tabs t WHERE t.workspace_id=w.id) FROM workspaces w";
fn verify(db: &Connection, expected: &WorkspaceRevision) -> Result<Workspace> {
    let workspace = db
        .query_row(&format!("{SUMMARY} WHERE w.id=?"), [&expected.id], summary)
        .optional()
        .map_err(storage)?;
    workspace
        .filter(|row| row.updated_at == expected.updated_at)
        .ok_or_else(|| invalid("Workspace changed; refresh before retrying"))
}
fn header(db: &Connection, id: &str) -> Result<Workspace> {
    db.query_row(&format!("{SUMMARY} WHERE w.id=?"), [id], summary)
        .map_err(storage)
}
fn next_revision(previous: Option<&str>) -> String {
    let mut now = chrono::Utc::now();
    if let Some(previous) =
        previous.and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        && now <= previous
    {
        now = previous.with_timezone(&chrono::Utc) + chrono::Duration::nanoseconds(1);
    }
    now.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}
impl Database {
    pub(crate) fn recover_workspace(
        &self,
        expected: Option<&WorkspaceRevision>,
        cancel: &Cancellation,
    ) -> Result<Option<WorkspaceReadings>> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let target = if let Some(expected) = expected {
            Some(verify(&tx, expected)?)
        } else {
            tx.query_row(
                &format!("{SUMMARY} WHERE w.is_default=1 ORDER BY w.updated_at DESC,w.id LIMIT 1"),
                [],
                summary,
            )
            .optional()
            .map_err(storage)?
        };
        target
            .map(|target| readings::load(self, &(&target).into(), cancel))
            .transpose()
    }
    pub(crate) fn workspace_page(
        &self,
        offset: usize,
        cancel: &Cancellation,
    ) -> Result<WorkspacePage> {
        cancel.check()?;
        let offset = i64::try_from(offset).map_err(|_| invalid("Workspace offset is too large"))?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let total = tx
            .query_row("SELECT count(*) FROM workspaces", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(storage)? as u64;
        let mut statement = tx
            .prepare(&format!(
                "{SUMMARY} ORDER BY w.is_default DESC,w.updated_at DESC,w.id LIMIT ? OFFSET ?"
            ))
            .map_err(storage)?;
        let rows = statement
            .query_map(params![MAX_LIBRARY_PAGE as i64, offset], summary)
            .map_err(storage)?;
        let mut entries = Vec::new();
        for row in rows {
            cancel.check()?;
            entries.push(row.map_err(storage)?);
        }
        Ok(WorkspacePage { entries, total })
    }
    pub(crate) fn workspace_readings(
        &self,
        expected: &WorkspaceRevision,
        cancel: &Cancellation,
    ) -> Result<WorkspaceReadings> {
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let result = readings::load(self, expected, cancel)?;
        drop(tx);
        Ok(result)
    }
}
