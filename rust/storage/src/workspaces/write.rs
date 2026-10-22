use super::*;
use yamantaka_domain::reader_tabs::{DocumentTabs, ReaderTab};

fn remap(tabs: &DocumentTabs) -> Result<DocumentTabs> {
    let mut mapped = Vec::new();
    let mut selected = None;
    for tab in tabs.tabs() {
        if mapped
            .iter()
            .any(|other: &ReaderTab| other.document() == tab.document())
        {
            return Err(invalid(
                "A saved workspace can contain each document only once; close duplicate readings before saving",
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        if tabs.active().is_some_and(|active| active.id() == tab.id()) {
            selected = Some(id.clone());
        }
        mapped.push(
            ReaderTab::new(
                id,
                tab.document().into(),
                tab.page(),
                tab.preferences().clone(),
                tab.panel(),
            )?
            .with_pinned(tab.pinned())
            .with_queue(tab.queue().clone())?,
        );
    }
    let mut model = DocumentTabs::default();
    model.restore(mapped, selected.as_deref())?;
    Ok(model)
}
fn readings(db: &Connection, id: &str, tabs: &DocumentTabs, cancel: &Cancellation) -> Result<()> {
    let model = remap(tabs)?;
    let split: Option<String> = db
        .query_row(
            "SELECT split_json FROM workspaces WHERE id=?",
            [id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let mut split: serde_json::Value = split
        .map(|value| serde_json::from_str(&value).map_err(storage))
        .transpose()?
        .unwrap_or_else(|| serde_json::json!({}));
    let object = split.as_object_mut().ok_or_else(|| {
        invalid("Workspace split settings must be an object; stored data is preserved")
    })?;
    if let Some(secondary) = object
        .get("secondaryTabId")
        .and_then(serde_json::Value::as_str)
    {
        let previous_document: Option<String> = db
            .query_row(
                "SELECT document_id FROM workspace_tabs WHERE workspace_id=? AND id=?",
                params![id, secondary],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage)?;
        let document = previous_document.as_deref().or_else(|| {
            tabs.tabs()
                .iter()
                .find(|tab| tab.id() == secondary)
                .map(ReaderTab::document)
        });
        let mapped = model
            .tabs()
            .iter()
            .find(|tab| Some(tab.document()) == document);
        let secondary = mapped
            .map(|tab| serde_json::Value::String(tab.id().into()))
            .unwrap_or(serde_json::Value::Null);
        if secondary.is_null() {
            object.insert("enabled".into(), serde_json::Value::Bool(false));
        }
        object.insert("secondaryTabId".into(), secondary);
    }
    object.insert(
        super::readings::CONTEXT.into(),
        crate::reader_continuity::encode_workspace(&model)?,
    );
    db.execute("DELETE FROM workspace_tabs WHERE workspace_id=?", [id])
        .map_err(storage)?;
    for (index, tab) in model.tabs().iter().enumerate() {
        cancel.check()?;
        let status = serde_json::to_value(tab.queue().status()).map_err(storage)?;
        db.execute("INSERT INTO workspace_tabs(id,workspace_id,document_id,sort_order,is_pinned,page_index,zoom_level,page_layout,reading_mode,scroll_offset,queue_status,blocking_reason,next_action) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)", params![tab.id(),id,tab.document(),index as i64,tab.pinned(),tab.page(),tab.preferences().zoom_level,tab.preferences().persisted_layout_name(),tab.preferences().view_mode.persisted_name(),tab.preferences().scroll_offset,status.as_str().unwrap(),tab.queue().blocking_reason(),tab.queue().next_action()]).map_err(storage)?;
    }
    db.execute(
        "UPDATE workspaces SET active_tab_id=?,split_json=? WHERE id=?",
        params![
            model.active().map(ReaderTab::id),
            serde_json::to_string(&split).map_err(storage)?,
            id
        ],
    )
    .map_err(storage)?;
    Ok(())
}
impl Database {
    pub(crate) fn apply_workspace(
        &mut self,
        change: &WorkspaceChange,
        cancel: &Cancellation,
    ) -> Result<WorkspaceChanged> {
        cancel.check()?;
        let tx = self
            .connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut deleted = None;
        let id = match change {
            WorkspaceChange::Create {
                name: value,
                tabs,
                view: layout,
            } => {
                let clean = name(value)?;
                let id = uuid::Uuid::new_v4().to_string();
                let now = next_revision(None);
                tx.execute(
                    "INSERT INTO workspaces(id,name,created_at,updated_at) VALUES(?,?,?,?)",
                    params![id, clean, now, now],
                )
                .map_err(storage)?;
                readings(&tx, &id, tabs, cancel)?;
                view(&tx, &id, layout)?;
                id
            }
            WorkspaceChange::Replace {
                expected,
                tabs,
                view: layout,
            } => {
                verify(&tx, expected)?;
                readings(&tx, &expected.id, tabs, cancel)?;
                if let Some(layout) = layout {
                    view(&tx, &expected.id, layout)?;
                }
                tx.execute(
                    "UPDATE workspaces SET updated_at=? WHERE id=?",
                    params![next_revision(Some(&expected.updated_at)), expected.id],
                )
                .map_err(storage)?;
                expected.id.clone()
            }
            WorkspaceChange::Comparison { expected, value } => {
                verify(&tx, expected)?;
                if let Some(value) = value {
                    value.validate()?;
                }
                let existing: String = tx
                    .query_row(
                        "SELECT query_json FROM workspaces WHERE id=?",
                        [&expected.id],
                        |row| row.get(0),
                    )
                    .map_err(storage)?;
                let mut query: serde_json::Value =
                    serde_json::from_str(&existing).map_err(storage)?;
                query
                    .as_object_mut()
                    .ok_or_else(|| {
                        invalid("Workspace query must be an object; stored data is preserved")
                    })?
                    .insert(
                        "yamantakaRustComparison".into(),
                        serde_json::to_value(value).map_err(storage)?,
                    );
                tx.execute(
                    "UPDATE workspaces SET query_json=?,updated_at=? WHERE id=?",
                    params![
                        serde_json::to_string(&query).map_err(storage)?,
                        next_revision(Some(&expected.updated_at)),
                        expected.id
                    ],
                )
                .map_err(storage)?;
                expected.id.clone()
            }
            WorkspaceChange::Rename {
                expected,
                name: value,
            } => {
                verify(&tx, expected)?;
                tx.execute(
                    "UPDATE workspaces SET name=?,updated_at=? WHERE id=?",
                    params![
                        name(value)?,
                        next_revision(Some(&expected.updated_at)),
                        expected.id
                    ],
                )
                .map_err(storage)?;
                expected.id.clone()
            }
            WorkspaceChange::SetDefault { expected } => {
                verify(&tx, expected)?;
                let now = next_revision(Some(&expected.updated_at));
                // Other records' reading contexts are unchanged by this flag.
                tx.execute("UPDATE workspaces SET is_default=0 WHERE is_default=1", [])
                    .map_err(storage)?;
                tx.execute(
                    "UPDATE workspaces SET is_default=1,updated_at=? WHERE id=?",
                    params![now, expected.id],
                )
                .map_err(storage)?;
                expected.id.clone()
            }
            WorkspaceChange::Focus { expected, enabled } => {
                verify(&tx, expected)?;
                tx.execute(
                    "UPDATE workspaces SET focus_mode=?,updated_at=? WHERE id=?",
                    params![
                        enabled,
                        next_revision(Some(&expected.updated_at)),
                        expected.id
                    ],
                )
                .map_err(storage)?;
                expected.id.clone()
            }
            WorkspaceChange::Delete { expected } => {
                let workspace = verify(&tx, expected)?;
                deleted = Some(super::undo::WorkspaceUndoData::capture(
                    &tx, &workspace, cancel,
                )?);
                tx.execute("DELETE FROM workspaces WHERE id=?", [&expected.id])
                    .map_err(storage)?;
                expected.id.clone()
            }
            WorkspaceChange::Duplicate { .. } => {
                return Err(invalid(
                    "Workspace duplication must load its source before the write transaction",
                ));
            }
            WorkspaceChange::UndoDelete { .. } => {
                return Err(invalid(
                    "Workspace undo requires its retained deletion receipt",
                ));
            }
        };
        cancel.check()?;
        let changed = if let Some(deleted) = &deleted {
            WorkspaceChanged::Deleted(deleted.receipt())
        } else {
            WorkspaceChanged::Saved(header(&tx, &id)?)
        };
        tx.commit().map_err(storage)?;
        if let Some(deleted) = deleted {
            self.workspace_undo = Some(deleted);
        }
        Ok(changed)
    }
    pub(crate) fn duplicate_workspace(
        &mut self,
        expected: &WorkspaceRevision,
        value: &str,
        cancel: &Cancellation,
    ) -> Result<WorkspaceChanged> {
        let clean = name(value)?;
        // The transaction freezes metadata and reader rows together.
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let source = super::readings::load(self, expected, cancel)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = next_revision(None);
        tx.execute("INSERT INTO workspaces(id,name,is_default,active_tab_id,panel_section,panel_width,split_json,collection_id,created_at,updated_at,query_json,grouping,sort_mode,density,visible_columns_json,focus_mode) SELECT ?,?,0,active_tab_id,panel_section,panel_width,split_json,collection_id,?,?,query_json,grouping,sort_mode,density,visible_columns_json,focus_mode FROM workspaces WHERE id=?",params![id,clean,now,now,expected.id]).map_err(storage)?;
        readings(&tx, &id, &source.tabs, cancel)?;
        cancel.check()?;
        let changed = WorkspaceChanged::Saved(header(&tx, &id)?);
        tx.commit().map_err(storage)?;
        Ok(changed)
    }
}
fn view(db: &Connection, id: &str, view: &WorkspaceView) -> Result<()> {
    view.validate()?;
    let existing: String = db
        .query_row(
            "SELECT query_json FROM workspaces WHERE id=?",
            [id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let mut query: serde_json::Value = serde_json::from_str(&existing).map_err(storage)?;
    let incoming = view
        .query
        .as_object()
        .ok_or_else(|| invalid("Workspace query must be an object"))?;
    query
        .as_object_mut()
        .ok_or_else(|| invalid("Workspace query must be an object; stored data is preserved"))?
        .extend(incoming.clone());
    WorkspaceView {
        query: query.clone(),
        ..view.clone()
    }
    .validate()?;
    let grouping = serde_json::to_value(view.grouping).map_err(storage)?;
    let sort = serde_json::to_value(view.sort).map_err(storage)?;
    let density = serde_json::to_value(view.density).map_err(storage)?;
    db.execute("UPDATE workspaces SET query_json=?,collection_id=?,grouping=?,sort_mode=?,density=?,focus_mode=? WHERE id=?",params![serde_json::to_string(&query).map_err(storage)?,view.collection_id,grouping.as_str().unwrap(),sort.as_str().unwrap(),density.as_str().unwrap(),view.focus,id]).map_err(storage)?;
    Ok(())
}
