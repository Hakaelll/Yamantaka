use super::*;
use yamantaka_domain::{
    reader_preferences::{ReaderMode, ReaderPreferences, ReaderTheme, ZoomMode},
    reader_tabs::{DocumentTabs, QueueEntry, ReaderTab},
    reading_sessions::restoration::SessionPanel,
};
pub(super) const CONTEXT: &str = "yamantakaRustReadingContext";
pub(super) fn panel(value: Option<&str>) -> Result<Option<SessionPanel>> {
    Ok(match value {
        None => None,
        Some("outline") => Some(SessionPanel::Outline),
        Some("thumbnails") => Some(SessionPanel::Thumbnails),
        Some("search") => Some(SessionPanel::Search),
        Some("bookmarks") => Some(SessionPanel::Bookmarks),
        Some("annotations") => Some(SessionPanel::Annotations),
        Some("text") => Some(SessionPanel::Text),
        Some(_) => {
            return Err(invalid(
                "Unknown saved workspace panel; stored data is preserved",
            ));
        }
    })
}
pub(super) fn load(
    database: &Database,
    expected: &WorkspaceRevision,
    cancel: &Cancellation,
) -> Result<WorkspaceReadings> {
    cancel.check()?;
    let db = &database.connection;
    let workspace = verify(db, expected)?;
    let view = db.query_row("SELECT query_json,collection_id,grouping,sort_mode,density,focus_mode FROM workspaces WHERE id=?",[&expected.id],|row|Ok((row.get::<_,String>(0)?,row.get::<_,Option<String>>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,String>(4)?,row.get::<_,bool>(5)?))).map_err(storage)?;
    let mut view = WorkspaceView {
        query: serde_json::from_str(&view.0).map_err(storage)?,
        collection_id: view.1,
        grouping: serde_json::from_value(serde_json::Value::String(view.2)).map_err(storage)?,
        sort: serde_json::from_value(serde_json::Value::String(view.3)).map_err(storage)?,
        density: serde_json::from_value(serde_json::Value::String(view.4)).map_err(storage)?,
        focus: view.5,
    };
    view.validate()?;
    if workspace.readings > 12 {
        return Err(invalid(
            "This workspace exceeds the twelve-reading limit; stored data is preserved",
        ));
    }
    let (active, section, split): (Option<String>, Option<String>, Option<String>) = db
        .query_row(
            "SELECT active_tab_id,panel_section,split_json FROM workspaces WHERE id=?",
            [&expected.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(storage)?;
    let panel = panel(section.as_deref())?;
    let mut statement = db.prepare("SELECT t.id,t.document_id,t.page_index,t.zoom_level,t.page_layout,t.reading_mode,t.scroll_offset,t.is_pinned,t.queue_status,t.blocking_reason,t.next_action,COALESCE(d.title_override,d.title,d.file_name),d.preferred_theme FROM workspace_tabs t JOIN documents d ON d.id=t.document_id WHERE t.workspace_id=? ORDER BY t.sort_order,t.id").map_err(storage)?;
    let mut rows = statement.query([&expected.id]).map_err(storage)?;
    let mut tabs = Vec::new();
    let mut captions = Vec::new();
    while let Some(row) = rows.next().map_err(storage)? {
        cancel.check()?;
        let id: String = row.get(0).map_err(storage)?;
        let document: String = row.get(1).map_err(storage)?;
        let mut preferences = crate::reader_preferences::load(database, &document)?
            .unwrap_or_else(|| ReaderPreferences::new(&document));
        preferences.zoom_level = row.get(3).map_err(storage)?;
        preferences.zoom_mode = ZoomMode::Custom;
        preferences.load_layout_name(&row.get::<_, String>(4).map_err(storage)?)?;
        preferences.view_mode =
            ReaderMode::from_persisted(&row.get::<_, String>(5).map_err(storage)?)?;
        preferences.scroll_offset = row.get(6).map_err(storage)?;
        preferences.reading_theme =
            ReaderTheme::from_persisted(&row.get::<_, String>(12).map_err(storage)?)?;
        let status =
            serde_json::from_value(serde_json::Value::String(row.get(8).map_err(storage)?))
                .map_err(storage)?;
        let reason: Option<String> = row.get(9).map_err(storage)?;
        let action: Option<String> = row.get(10).map_err(storage)?;
        let page: i64 = row.get(2).map_err(storage)?;
        let page =
            u32::try_from(page).map_err(|_| invalid("Saved workspace page is out of range"))?;
        let tab = ReaderTab::new(id, document.clone(), page, preferences, panel)?
            .with_pinned(row.get(7).map_err(storage)?)
            .with_queue(QueueEntry::new(
                status,
                reason.as_deref().unwrap_or_default(),
                action.as_deref().unwrap_or_default(),
            ))?;
        captions.push((document, row.get(11).map_err(storage)?));
        tabs.push(tab);
    }
    let selected = active
        .as_deref()
        .filter(|id| tabs.iter().any(|tab| tab.id() == *id))
        .or_else(|| tabs.first().map(ReaderTab::id));
    let mut model = DocumentTabs::default();
    model.restore(tabs.clone(), selected)?;
    let split: serde_json::Value = split
        .map(|split| serde_json::from_str(&split).map_err(storage))
        .transpose()?
        .unwrap_or_else(|| serde_json::json!({}));
    if let Some(context) = split.get(CONTEXT) {
        let rich = crate::reader_continuity::decode_workspace(context.clone())?;
        // Flutter may edit the v11 rows without updating the Rust extension.
        if rich.tabs().len() == tabs.len()
            && rich.active().map(ReaderTab::id) == model.active().map(ReaderTab::id)
            && rich.tabs().iter().zip(&tabs).all(|(a, b)| {
                a.id() == b.id()
                    && a.document() == b.document()
                    && a.page() == b.page()
                    && a.pinned() == b.pinned()
                    && a.queue() == b.queue()
                    && a.preferences().zoom_level == b.preferences().zoom_level
                    && a.preferences().persisted_layout_name()
                        == b.preferences().persisted_layout_name()
                    && a.preferences().view_mode == b.preferences().view_mode
                    && a.preferences().scroll_offset == b.preferences().scroll_offset
            })
        {
            model = rich;
        }
    }
    let mut comparison_warning = None;
    if view.query.get("yamantakaRustComparison").is_none() {
        match super::legacy_comparison::decode(&split, &model) {
            Ok(Some(comparison)) => {
                view.query
                    .as_object_mut()
                    .ok_or_else(|| {
                        invalid("Workspace query must be an object; stored data is preserved")
                    })?
                    .insert(
                        "yamantakaRustComparison".into(),
                        serde_json::to_value(comparison).map_err(storage)?,
                    );
            }
            Ok(None) => {}
            Err(error) => comparison_warning = Some(error.to_string()),
        }
    }
    let reference_document = view
        .parsed_query()?
        .comparison
        .map(|comparison| yamantaka_domain::LibraryRepository::get(database, &comparison.document))
        .transpose()?
        .flatten();
    cancel.check()?;
    Ok(WorkspaceReadings {
        workspace,
        tabs: model,
        captions,
        view,
        reference_document,
        comparison_warning,
    })
}
