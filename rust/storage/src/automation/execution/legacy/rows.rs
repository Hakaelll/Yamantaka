use super::*;
use rusqlite::types::{Value, ValueRef};
use serde::{Deserialize, Serialize};
use yamantaka_domain::memory::ByteLease;
#[derive(Serialize, Deserialize)]
pub(super) enum Cell {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}
impl Cell {
    fn value(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Integer(value) => Value::Integer(*value),
            Self::Real(value) => Value::Real(*value),
            Self::Text(value) => Value::Text(value.clone()),
            Self::Blob(value) => Value::Blob(value.clone()),
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RemovedRow {
    pub kind: LegacyTargetKind,
    pub columns: Vec<String>,
    pub cells: Vec<Cell>,
}
fn membership_table(kind: LegacyTargetKind) -> &'static str {
    match kind {
        LegacyTargetKind::Tag => "document_tags",
        LegacyTargetKind::Collection => "collection_documents",
        LegacyTargetKind::Workspace => "workspace_tabs",
    }
}
fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
pub(super) fn capture(
    db: &Connection,
    kind: LegacyTargetKind,
    document: &str,
    target: &str,
    governor: &MemoryGovernor,
) -> Result<Option<(RemovedRow, ByteLease)>> {
    let mut query = db.prepare(membership_sql(kind)).map_err(storage)?;
    let mut rows = query.query(params![document, target]).map_err(storage)?;
    let Some(row) = rows.next().map_err(storage)? else {
        return Ok(None);
    };
    let mut bytes = 1024usize;
    for index in 0..row.as_ref().column_count() {
        let value = row.get_ref(index).map_err(storage)?;
        let payload = match value {
            ValueRef::Text(bytes) | ValueRef::Blob(bytes) => bytes.len(),
            _ => 16,
        };
        bytes = bytes
            .checked_add(payload.checked_mul(16).ok_or(Failure::MemoryBudget)?)
            .and_then(|bytes| {
                bytes.checked_add(128 + row.as_ref().column_name(index).ok()?.len() * 16)
            })
            .ok_or(Failure::MemoryBudget)?;
    }
    let lease = super::super::super::plan::reserve(governor, bytes)?;
    let mut cells = Vec::new();
    let mut columns = Vec::new();
    for index in 0..row.as_ref().column_count() {
        columns.push(row.as_ref().column_name(index).map_err(storage)?.into());
        cells.push(match row.get_ref(index).map_err(storage)? {
            ValueRef::Null => Cell::Null,
            ValueRef::Integer(value) => Cell::Integer(value),
            ValueRef::Real(value) if value.is_finite() => Cell::Real(value),
            ValueRef::Real(_) => {
                return Err(invalid(
                    "A queue entry contains a non-finite value; it was preserved",
                ));
            }
            ValueRef::Text(value) => {
                Cell::Text(std::str::from_utf8(value).map_err(storage)?.into())
            }
            ValueRef::Blob(value) => Cell::Blob(value.into()),
        });
    }
    Ok(Some((
        RemovedRow {
            kind,
            columns,
            cells,
        },
        lease,
    )))
}
pub(super) fn restore(db: &Connection, row: &RemovedRow) -> Result<()> {
    let table = membership_table(row.kind);
    let statement = db
        .prepare(&format!("SELECT * FROM {table} LIMIT 0"))
        .map_err(storage)?;
    if statement.column_names() != row.columns.iter().map(String::as_str).collect::<Vec<_>>()
        || row.cells.len() != row.columns.len()
    {
        return Err(invalid(
            "A removed membership receipt no longer matches this schema",
        ));
    }
    let values = row.cells.iter().map(Cell::value).collect::<Vec<_>>();
    let columns = row
        .columns
        .iter()
        .map(|name| quote(name))
        .collect::<Vec<_>>()
        .join(",");
    let placeholders = vec!["?"; row.cells.len()].join(",");
    db.execute(
        &format!("INSERT INTO {table}({columns}) VALUES({placeholders})"),
        rusqlite::params_from_iter(values),
    )
    .map_err(storage)?;
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorkspaceState {
    pub stable: [u8; 32],
    pub queue: [u8; 32],
    pub active: Option<String>,
    pub split: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorkspaceUndo {
    pub id: String,
    pub before: WorkspaceState,
    pub after: WorkspaceState,
}
pub(super) fn workspace(
    db: &Connection,
    id: &str,
    cancel: &Cancellation,
) -> Result<WorkspaceState> {
    let mut query = db
        .prepare("SELECT * FROM workspaces WHERE id=?")
        .map_err(storage)?;
    let mut rows = query.query([id]).map_err(storage)?;
    let row = rows
        .next()
        .map_err(storage)?
        .ok_or_else(|| invalid("The queue workspace no longer exists"))?;
    let stable = super::super::super::plan::signature_excluding(row, Some("updated_at"))?;
    let active = optional_text(row, "active_tab_id")?.map(str::to_owned);
    let split = optional_text(row, "split_json")?.map(str::to_owned);
    let mut query = db
        .prepare("SELECT * FROM workspace_tabs WHERE workspace_id=? ORDER BY id")
        .map_err(storage)?;
    let mut rows = query.query([id]).map_err(storage)?;
    let mut digest = Sha256::new();
    while let Some(row) = rows.next().map_err(storage)? {
        cancel.check()?;
        digest.update(super::super::super::plan::signature(row)?);
    }
    Ok(WorkspaceState {
        stable,
        queue: digest.finalize().into(),
        active,
        split,
    })
}
pub(super) fn workspace_budget(
    db: &Connection,
    id: &str,
    governor: &MemoryGovernor,
) -> Result<ByteLease> {
    let mut query = db
        .prepare("SELECT * FROM workspaces WHERE id=?")
        .map_err(storage)?;
    let mut rows = query.query([id]).map_err(storage)?;
    let row = rows
        .next()
        .map_err(storage)?
        .ok_or_else(|| invalid("The queue workspace no longer exists"))?;
    let bytes = optional_text(row, "split_json")?
        .map_or(0, str::len)
        .checked_add(optional_text(row, "active_tab_id")?.map_or(0, str::len))
        .and_then(|bytes| bytes.checked_mul(32))
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or(Failure::MemoryBudget)?;
    super::super::super::plan::reserve(governor, bytes)
}
