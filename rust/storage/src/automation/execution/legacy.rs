use super::*;
use yamantaka_domain::automation::legacy::*;
mod apply;
mod parser;
mod revert;
mod review;
mod rows;
use parser::Receipt;
pub(super) const FORMAT: &str = "yamantaka-legacy-recovery-v1";
fn table(kind: LegacyTargetKind) -> &'static str {
    match kind {
        LegacyTargetKind::Tag => "tags",
        LegacyTargetKind::Collection => "collections",
        LegacyTargetKind::Workspace => "workspaces",
    }
}
fn optional_text<'a>(row: &'a rusqlite::Row<'_>, column: &str) -> Result<Option<&'a str>> {
    match row.get_ref(column).map_err(storage)? {
        rusqlite::types::ValueRef::Null => Ok(None),
        value => value.as_str().map(Some).map_err(storage),
    }
}
fn record(db: &Connection, table: &str, id: &str) -> Result<Option<[u8; 32]>> {
    let mut query = db
        .prepare(&format!("SELECT * FROM {table} WHERE id=?"))
        .map_err(storage)?;
    let mut rows = query.query([id]).map_err(storage)?;
    rows.next()
        .map_err(storage)?
        .map(super::super::plan::signature)
        .transpose()
}
fn membership_sql(kind: LegacyTargetKind) -> &'static str {
    match kind {
        LegacyTargetKind::Tag => "SELECT * FROM document_tags WHERE document_id=? AND tag_id=?",
        LegacyTargetKind::Collection => {
            "SELECT * FROM collection_documents WHERE document_id=? AND collection_id=?"
        }
        LegacyTargetKind::Workspace => {
            "SELECT * FROM workspace_tabs WHERE document_id=? AND workspace_id=?"
        }
    }
}
fn membership(
    db: &Connection,
    kind: LegacyTargetKind,
    document: &str,
    target: &str,
) -> Result<Option<[u8; 32]>> {
    let mut query = db.prepare(membership_sql(kind)).map_err(storage)?;
    let mut rows = query.query(params![document, target]).map_err(storage)?;
    rows.next()
        .map_err(storage)?
        .map(super::super::plan::signature)
        .transpose()
}
pub(super) fn available(db: &Connection, expected: &JournalRevision) -> Result<()> {
    journal::verify_kind(db, expected, journal::LEGACY)?;
    let eligible:bool=db.query_row("SELECT resume_capability='rollback' AND undo_json IS NOT NULL AND status<>'rolledBack' FROM operation_journal WHERE id=? AND operation_kind=?",params![expected.id,journal::LEGACY],|row|row.get(0)).map_err(storage)?;
    if !eligible {
        return Err(invalid(
            "This inherited execution has no pending rollback receipts",
        ));
    }
    let active:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM operation_journal WHERE operation_kind=? AND resume_capability='rollback' AND CASE WHEN json_valid(summary_json) THEN json_extract(summary_json,'$.legacyJournalId') END=?)",params![journal::ROOT,expected.id],|row|row.get(0)).map_err(storage)?;
    if active {
        return Err(invalid(
            "Review or undo the existing recovery before preparing another one",
        ));
    }
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DocumentUndo {
    document: String,
    identity: [u8; 32],
    favorite: Option<(bool, bool)>,
    reading: Option<(LegacyReadingState, LegacyReadingState)>,
    removed: Vec<rows::RemovedRow>,
    workspaces: Vec<rows::WorkspaceUndo>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalUndo {
    id: String,
    before: LegacyJournalState,
    after: Option<[u8; 32]>,
}
pub(super) fn revert_document(
    db: &Connection,
    undo: &DocumentUndo,
    context: &ExecutionContext,
) -> Result<()> {
    revert::document(db, undo, context)
}
pub(super) fn revert_journal(db: &Connection, undo: &JournalUndo) -> Result<()> {
    revert::journal(db, undo)
}
pub(super) fn recover(
    db: &mut Database,
    plan: &LegacyRecoveryPlan,
    context: &ExecutionContext,
) -> Result<Arc<JournalEntry>> {
    apply::recover(db, plan, context)
}
