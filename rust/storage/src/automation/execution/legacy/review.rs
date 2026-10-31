use super::*;
use yamantaka_domain::{
    memory::ByteLease,
    reader_preferences::{ReaderMode, ReaderTheme},
};
fn target(
    db: &Connection,
    kind: LegacyTargetKind,
    id: &str,
    targets: &mut Vec<LegacyTarget>,
    leases: &mut Vec<ByteLease>,
    governor: &MemoryGovernor,
) -> Result<u8> {
    if let Some(index) = targets
        .iter()
        .position(|target| target.kind == kind && target.id == id)
    {
        return Ok(index as u8);
    }
    if targets.len() >= 8 {
        return Err(invalid(
            "Inherited receipts exceed eight distinct destinations",
        ));
    }
    let mut statement = db
        .prepare(&format!("SELECT * FROM {} WHERE id=?", table(kind)))
        .map_err(storage)?;
    let mut rows = statement.query([id]).map_err(storage)?;
    let row = rows.next().map_err(storage)?;
    let name = row
        .map(|row| super::super::super::field(row, 1))
        .transpose()?
        .unwrap_or(id);
    let bytes = id
        .len()
        .checked_add(name.len())
        .and_then(|bytes| bytes.checked_mul(2))
        .and_then(|bytes| bytes.checked_add(256))
        .ok_or(Failure::MemoryBudget)?;
    leases.push(super::super::super::plan::reserve(governor, bytes)?);
    let revision = row.map(super::super::super::plan::signature).transpose()?;
    let index = targets.len() as u8;
    targets.push(LegacyTarget {
        kind,
        id: id.into(),
        name: name.into(),
        revision,
    });
    Ok(index)
}
fn document(
    db: &Connection,
    group: parser::Group,
    targets: &mut Vec<LegacyTarget>,
    leases: &mut Vec<ByteLease>,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<Arc<LegacyDocument>> {
    let mut statement = db
        .prepare("SELECT * FROM documents WHERE id=?")
        .map_err(storage)?;
    let mut rows = statement.query([&group.id]).map_err(storage)?;
    let Some(row) = rows.next().map_err(storage)? else {
        let bytes = group
            .id
            .len()
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(256))
            .ok_or(Failure::MemoryBudget)?;
        let lease = super::super::super::plan::reserve(governor, bytes)?;
        drop(rows);
        drop(statement);
        return Ok(Arc::new(LegacyDocument::new(
            group.id.clone(),
            group.id,
            None,
            Vec::new(),
            lease,
        )));
    };
    let title = optional_text(row, "title_override")?
        .or(optional_text(row, "title")?)
        .or(optional_text(row, "source_title")?)
        .or(optional_text(row, "file_name")?)
        .unwrap_or(&group.id);
    let mode = optional_text(row, "preferred_view_mode")?
        .ok_or_else(|| invalid("Current reading mode is invalid"))?;
    let theme = optional_text(row, "preferred_theme")?
        .ok_or_else(|| invalid("Current reading theme is invalid"))?;
    let opened = optional_text(row, "last_opened_at")?
        .ok_or_else(|| invalid("Current reading timestamp is invalid"))?;
    let bytes = group
        .id
        .len()
        .checked_add(title.len())
        .and_then(|bytes| bytes.checked_add(mode.len()))
        .and_then(|bytes| bytes.checked_add(theme.len()))
        .and_then(|bytes| bytes.checked_add(opened.len()))
        .and_then(|bytes| bytes.checked_mul(2))
        .and_then(|bytes| {
            bytes.checked_add(
                std::mem::size_of::<LegacyDocument>()
                    + std::mem::size_of::<LegacyReadingState>()
                    + group.receipts.len() * std::mem::size_of::<LegacyChange>()
                    + 256,
            )
        })
        .ok_or(Failure::MemoryBudget)?;
    let lease = super::super::super::plan::reserve(governor, bytes)?;
    let revision = super::super::super::plan::signature(row)?;
    let count: u32 = row.get("page_count").map_err(storage)?;
    let current = Arc::new(LegacyReadingState {
        page: row.get("last_page_index").map_err(storage)?,
        progress: row.get("reading_progress").map_err(storage)?,
        mode: mode.into(),
        theme: theme.into(),
        opened: opened.into(),
    });
    let favorite: bool = row.get("is_favorite").map_err(storage)?;
    let title = title.to_owned();
    drop(rows);
    drop(statement);
    let mut changes = Vec::with_capacity(group.receipts.len());
    for receipt in group.receipts {
        cancel.check()?;
        changes.push(match receipt {
            Receipt::RestoreFavorite { value, .. } => LegacyChange::RestoreFavorite {
                current: favorite,
                value,
            },
            Receipt::RestoreProgress {
                page_index,
                page_count,
                view_mode,
                theme,
                ..
            } => {
                if count != page_count || page_index > page_count.saturating_sub(1) {
                    return Err(invalid(
                        "An inherited progress receipt no longer matches the document page count",
                    ));
                }
                LegacyChange::RestoreProgress {
                    current: current.clone(),
                    page: page_index,
                    page_count,
                    mode: ReaderMode::from_persisted(&view_mode)?,
                    theme: ReaderTheme::from_persisted(&theme)?,
                }
            }
            receipt => {
                let (kind, id) = match receipt {
                    Receipt::RemoveTag { tag_id, .. } => (LegacyTargetKind::Tag, tag_id),
                    Receipt::RemoveCollection { collection_id, .. } => {
                        (LegacyTargetKind::Collection, collection_id)
                    }
                    Receipt::RemoveQueue { workspace_id, .. } => {
                        (LegacyTargetKind::Workspace, workspace_id)
                    }
                    _ => unreachable!(),
                };
                let index = target(db, kind, &id, targets, leases, governor)?;
                LegacyChange::RemoveMembership {
                    target: index,
                    revision: membership(db, kind, &group.id, &id)?,
                }
            }
        });
    }
    Ok(Arc::new(LegacyDocument::new(
        group.id,
        title,
        Some(revision),
        changes,
        lease,
    )))
}
pub(crate) fn prepare(
    db: &Connection,
    expected: &JournalRevision,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<LegacyRecoveryPlan> {
    cancel.check()?;
    let tx = db.unchecked_transaction().map_err(storage)?;
    available(&tx, expected)?;
    let journal = journal::entry(&tx, &expected.id, governor)?;
    let mut statement = tx
        .prepare("SELECT * FROM operation_journal WHERE id=?")
        .map_err(storage)?;
    let mut rows = statement.query([&expected.id]).map_err(storage)?;
    let row = rows
        .next()
        .map_err(storage)?
        .ok_or_else(|| invalid("Inherited journal no longer exists"))?;
    let json =
        optional_text(row, "undo_json")?.ok_or_else(|| invalid("Inherited receipts are absent"))?;
    let raw_bytes = json
        .len()
        .checked_mul(8)
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or(Failure::MemoryBudget)?;
    let _raw = super::super::super::plan::reserve(governor, raw_bytes)?;
    let mut metadata = 0usize;
    for column in [
        "status",
        "phase",
        "resume_capability",
        "error_code",
        "interruption_reason",
    ] {
        metadata = metadata
            .checked_add(optional_text(row, column)?.map_or(0, str::len))
            .ok_or(Failure::MemoryBudget)?;
    }
    let lease = super::super::super::plan::reserve(
        governor,
        metadata
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or(Failure::MemoryBudget)?,
    )?;
    let before = LegacyJournalState {
        status: optional_text(row, "status")?.unwrap().into(),
        phase: optional_text(row, "phase")?.unwrap().into(),
        capability: optional_text(row, "resume_capability")?.unwrap().into(),
        error: optional_text(row, "error_code")?.map(str::to_owned),
        interruption: optional_text(row, "interruption_reason")?.map(str::to_owned),
    };
    let (groups, skipped) = parser::parse(json, cancel)?;
    drop(rows);
    drop(statement);
    let vector = super::super::super::plan::reserve(
        governor,
        groups
            .len()
            .checked_mul(std::mem::size_of::<Arc<LegacyDocument>>())
            .ok_or(Failure::MemoryBudget)?,
    )?;
    let mut documents = Vec::new();
    documents
        .try_reserve_exact(groups.len())
        .map_err(|_| Failure::MemoryBudget)?;
    let mut targets = Vec::new();
    let mut leases = vec![lease, vector];
    for group in groups {
        cancel.check()?;
        documents.push(document(
            &tx,
            group,
            &mut targets,
            &mut leases,
            governor,
            cancel,
        )?);
    }
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(LegacyRecoveryPlan::new(
        journal, before, documents, targets, skipped, leases,
    ))
}

impl Database {
    pub(crate) fn legacy_recovery_plan(
        &self,
        journal: &JournalRevision,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<LegacyRecoveryPlan> {
        prepare(&self.connection, journal, governor, cancel)
    }
}
