use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum TargetKind {
    Tag,
    Collection,
    Workspace,
}
impl TargetKind {
    pub(super) fn table(self) -> &'static str {
        match self {
            Self::Tag => "tags",
            Self::Collection => "collections",
            Self::Workspace => "workspaces",
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreatedTarget {
    pub kind: TargetKind,
    pub id: String,
    pub revision: [u8; 32],
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AddedRow {
    pub kind: TargetKind,
    pub target: String,
    pub id: String,
    pub revision: [u8; 32],
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Progress {
    pub page: i64,
    pub value: f64,
    pub opened: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DocumentUndo {
    pub document: String,
    pub identity: [u8; 32],
    pub favorite: Option<(bool, bool)>,
    pub progress: Option<(Progress, Progress)>,
    pub rows: Vec<AddedRow>,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(super) enum Undo {
    Targets { entries: Vec<CreatedTarget> },
    Document(DocumentUndo),
    LegacyDocument(legacy::DocumentUndo),
    LegacyJournal(legacy::JournalUndo),
}
pub(super) struct TargetGuard {
    pub kind: TargetKind,
    pub id: String,
    pub revision: [u8; 32],
}
pub(super) fn target(action: &PlannedAction) -> Option<(TargetKind, &PlannedTarget)> {
    match action {
        PlannedAction::AddTag(value) => Some((TargetKind::Tag, value)),
        PlannedAction::AddCollection(value) => Some((TargetKind::Collection, value)),
        PlannedAction::AddWorkspace(value) | PlannedAction::AddReadingQueue(value) => {
            Some((TargetKind::Workspace, value))
        }
        _ => None,
    }
}
pub(super) fn targets(db: &Connection, plan: &RulePlan) -> Result<(Vec<TargetGuard>, Undo)> {
    let mut guards: Vec<TargetGuard> = Vec::new();
    let mut created = Vec::new();
    for (kind, value) in plan.actions().iter().filter_map(target) {
        if guards
            .iter()
            .any(|guard| guard.kind == kind && guard.id == value.id)
        {
            continue;
        }
        if let Some(expected) = value.revision {
            if super::super::plan::record(db, kind.table(), &value.id)? != expected {
                return Err(invalid("A reviewed destination changed; review again"));
            }
        } else {
            if kind == TargetKind::Workspace {
                return Err(invalid("A reviewed workspace must exist"));
            }
            let occupied: bool = db
                .query_row(
                    &format!(
                        "SELECT EXISTS(SELECT 1 FROM {} WHERE id=? OR normalized_name=?)",
                        kind.table()
                    ),
                    params![value.id, value.name.to_lowercase()],
                    |row| row.get(0),
                )
                .map_err(storage)?;
            if occupied {
                return Err(invalid("A new destination now exists; review again"));
            }
            let now = stamp("");
            match kind {
                TargetKind::Tag => {
                    db.execute("INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES(?,?,?,'violet',?,?)",params![value.id,value.name,value.name.to_lowercase(),now,now]).map_err(storage)?;
                }
                TargetKind::Collection => {
                    db.execute("INSERT INTO collections(id,name,normalized_name,sort_order,created_at,updated_at) SELECT ?,?,?,COALESCE(MAX(sort_order),-1)+1,?,? FROM collections",params![value.id,value.name,value.name.to_lowercase(),now,now]).map_err(storage)?;
                }
                TargetKind::Workspace => unreachable!(),
            }
            created.push(CreatedTarget {
                kind,
                id: value.id.clone(),
                revision: super::super::plan::record(db, kind.table(), &value.id)?,
            });
        }
        guards.push(TargetGuard {
            kind,
            id: value.id.clone(),
            revision: super::super::plan::record(db, kind.table(), &value.id)?,
        });
    }
    Ok((guards, Undo::Targets { entries: created }))
}
pub(super) fn verify_targets(db: &Connection, guards: &[TargetGuard]) -> Result<()> {
    for guard in guards {
        if super::super::plan::record(db, guard.kind.table(), &guard.id)? != guard.revision {
            return Err(invalid("A destination changed during execution"));
        }
    }
    Ok(())
}
pub(super) fn refresh_guards(db: &Connection, guards: &mut [TargetGuard]) -> Result<()> {
    for guard in guards {
        guard.revision = super::super::plan::record(db, guard.kind.table(), &guard.id)?;
    }
    Ok(())
}
pub(super) fn membership_signature(
    db: &Connection,
    kind: TargetKind,
    id: &str,
    target: &str,
) -> Result<[u8; 32]> {
    let query = match kind {
        TargetKind::Tag => "SELECT * FROM document_tags WHERE document_id=? AND tag_id=?",
        TargetKind::Collection => {
            "SELECT * FROM collection_documents WHERE document_id=? AND collection_id=?"
        }
        TargetKind::Workspace => "SELECT * FROM workspace_tabs WHERE id=? AND workspace_id=?",
    };
    let mut statement = db.prepare(query).map_err(storage)?;
    let mut rows = statement.query(params![id, target]).map_err(storage)?;
    super::super::plan::signature(
        rows.next()
            .map_err(storage)?
            .ok_or_else(|| invalid("An automation membership no longer exists"))?,
    )
}
pub(super) fn progress(db: &Connection, id: &str) -> Result<Progress> {
    db.query_row(
        "SELECT last_page_index,reading_progress,last_opened_at FROM documents WHERE id=?",
        [id],
        |row| {
            Ok(Progress {
                page: row.get(0)?,
                value: row.get(1)?,
                opened: row.get(2)?,
            })
        },
    )
    .map_err(storage)
}
pub(super) fn document_identity(db: &Connection, id: &str) -> Result<[u8; 32]> {
    let mut statement=db.prepare("SELECT id,fingerprint,page_count,file_size,file_modified_at,normalized_path FROM documents WHERE id=?").map_err(storage)?;
    let mut rows = statement.query([id]).map_err(storage)?;
    super::super::plan::signature(
        rows.next()
            .map_err(storage)?
            .ok_or_else(|| invalid("The document source no longer exists"))?,
    )
}
pub(super) fn document_budget(
    db: &Connection,
    id: &str,
    context: &ExecutionContext,
) -> Result<yamantaka_domain::memory::ByteLease> {
    let mut budget_query = db
        .prepare("SELECT last_opened_at FROM documents WHERE id=?")
        .map_err(storage)?;
    let mut budget_rows = budget_query.query([id]).map_err(storage)?;
    let budget_row = budget_rows
        .next()
        .map_err(storage)?
        .ok_or_else(|| invalid("Document no longer exists"))?;
    let opened = super::super::field(budget_row, 0)?;
    let lease = super::super::plan::reserve(
        &context.governor,
        opened
            .len()
            .checked_mul(64)
            .and_then(|bytes| bytes.checked_add(65536))
            .ok_or(Failure::MemoryBudget)?,
    )?;
    drop(budget_rows);
    drop(budget_query);
    Ok(lease)
}
pub(super) fn apply_document(
    db: &Connection,
    plan: &RulePlan,
    index: usize,
    context: &ExecutionContext,
) -> Result<Undo> {
    let matched = &plan.report().matches[index];
    let id = &matched.document;
    if super::super::plan::record(db, "documents", id)? != plan.document_revisions()[index] {
        return Err(invalid("A reviewed document changed; review again"));
    }
    let old_favorite: bool = db
        .query_row(
            "SELECT is_favorite FROM documents WHERE id=?",
            [id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let old_progress = progress(db, id)?;
    let mut undo = DocumentUndo {
        document: id.clone(),
        identity: document_identity(db, id)?,
        favorite: None,
        progress: None,
        rows: Vec::new(),
    };
    for action in plan.actions() {
        context.cancel.check()?;
        if let Some((kind, target)) = target(action) {
            let row_id = if kind == TargetKind::Workspace {
                uuid::Uuid::new_v4().to_string()
            } else {
                id.clone()
            };
            let changed=match kind {
                TargetKind::Tag=>db.execute("INSERT OR IGNORE INTO document_tags(document_id,tag_id) VALUES(?,?)",params![id,target.id]),
                TargetKind::Collection=>db.execute("INSERT OR IGNORE INTO collection_documents(collection_id,document_id,added_at) VALUES(?,?,?)",params![target.id,id,stamp("")]),
                TargetKind::Workspace=>db.execute("INSERT OR IGNORE INTO workspace_tabs(id,workspace_id,document_id,sort_order,page_index,reading_mode) SELECT ?,?,?,COALESCE(MAX(sort_order),-1)+1,?,(SELECT preferred_view_mode FROM documents WHERE id=?) FROM workspace_tabs WHERE workspace_id=? HAVING COUNT(*)<1000",params![row_id,target.id,id,old_progress.page,id,target.id]),
            }.map_err(storage)?;
            if changed > 0 {
                let revision = membership_signature(db, kind, &row_id, &target.id)?;
                undo.rows.push(AddedRow {
                    kind,
                    target: target.id.clone(),
                    id: row_id,
                    revision,
                });
                if kind == TargetKind::Workspace {
                    db.execute(
                        "UPDATE workspaces SET active_tab_id=COALESCE(active_tab_id,?) WHERE id=?",
                        params![undo.rows.last().unwrap().id, target.id],
                    )
                    .map_err(storage)?;
                    touch_workspace(db, &target.id)?;
                }
            }
        } else {
            match action {
                PlannedAction::SetFavorite(value) => {
                    db.execute(
                        "UPDATE documents SET is_favorite=? WHERE id=?",
                        params![value, id],
                    )
                    .map_err(storage)?;
                }
                PlannedAction::SetReadingState { finished } => {
                    if *finished {
                        db.execute("UPDATE documents SET last_page_index=MAX(page_count-1,0),reading_progress=1,last_opened_at=? WHERE id=?",params![stamp(""),id]).map_err(storage)?;
                    } else {
                        db.execute(
                            "UPDATE documents SET last_page_index=0,reading_progress=0 WHERE id=?",
                            [id],
                        )
                        .map_err(storage)?;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    let new_favorite: bool = db
        .query_row(
            "SELECT is_favorite FROM documents WHERE id=?",
            [id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let new_progress = progress(db, id)?;
    if old_favorite != new_favorite {
        undo.favorite = Some((old_favorite, new_favorite));
    }
    if old_progress != new_progress {
        undo.progress = Some((old_progress, new_progress));
    }
    crate::search::refresh_document_tags(db, id, context.access)?;
    Ok(Undo::Document(undo))
}
pub(super) fn touch_workspace(db: &Connection, id: &str) -> Result<()> {
    let previous: String = db
        .query_row(
            "SELECT updated_at FROM workspaces WHERE id=?",
            [id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    db.execute(
        "UPDATE workspaces SET updated_at=? WHERE id=?",
        params![stamp(&previous), id],
    )
    .map_err(storage)?;
    Ok(())
}
