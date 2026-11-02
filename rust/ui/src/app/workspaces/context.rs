use super::super::library_grouping::Grouping;
use super::*;
use yamantaka_domain::workspaces::{
    WorkspaceDensity, WorkspaceGrouping, WorkspaceQuery, WorkspaceView,
};

impl YamantakaApp {
    pub(super) fn capture_workspace_view(&self) -> WorkspaceView {
        let query = WorkspaceQuery {
            text: self.query.clone(),
            favorites_only: self.favorites,
            missing_only: self.missing_only,
            modified_only: self.modified_only,
            reading_state: self.reading_state,
            sort_direction: self.sort_direction,
            tag_ids: self
                .library_tags
                .filters
                .iter()
                .map(|(id, _)| id.clone())
                .collect(),
            list: self.list,
            inspector: self.inspector,
            smart_view: self.smart_view.map(|view| view.key().into()),
            comparison: self.capture_comparison(),
            ..Default::default()
        };
        WorkspaceView {
            query: serde_json::to_value(query)
                .expect("Workspace query contains serializable values"),
            collection_id: self.collections.filter.as_ref().map(|(id, _)| id.clone()),
            grouping: match self.grouping.mode {
                Grouping::None => WorkspaceGrouping::None,
                Grouping::Author => WorkspaceGrouping::Author,
                Grouping::Collection => WorkspaceGrouping::Collection,
                Grouping::Tag => WorkspaceGrouping::Tag,
                Grouping::Reading => WorkspaceGrouping::ReadingState,
            },
            sort: self.sort,
            density: self.library_density,
            focus: self.reader_focus,
            panel_width: f64::from(self.reader_panel_width),
            visible_columns: self.library_columns.clone(),
        }
    }
    pub(super) fn apply_workspace_view(
        &mut self,
        view: &WorkspaceView,
    ) -> yamantaka_domain::Result<()> {
        view.validate()?;
        let query = view.parsed_query()?;
        self.query = query.text;
        self.favorites = query.favorites_only;
        self.missing_only = query.missing_only;
        self.modified_only = query.modified_only;
        self.reading_state = query.reading_state;
        self.sort_direction = query.sort_direction;
        self.list = query.list;
        self.inspector = query.inspector;
        self.smart_view = query
            .smart_view
            .as_deref()
            .and_then(yamantaka_domain::LibrarySmartView::from_key);
        self.library_tags.filters = query
            .tag_ids
            .into_iter()
            .map(|id| {
                let label = self
                    .library_tags
                    .filters
                    .iter()
                    .find(|(old, _)| *old == id)
                    .map(|(_, label)| label.clone())
                    .unwrap_or_else(|| self.tr("Etiqueta", "Tag").into());
                (id, label)
            })
            .collect();
        self.collections.filter = view.collection_id.as_ref().map(|id| {
            let label = self
                .collections
                .filter
                .as_ref()
                .filter(|(old, _)| old == id)
                .map(|(_, label)| label.clone())
                .unwrap_or_else(|| self.tr("Colección", "Collection").into());
            (id.clone(), label)
        });
        self.grouping.mode = match view.grouping {
            WorkspaceGrouping::None => Grouping::None,
            WorkspaceGrouping::Author => Grouping::Author,
            WorkspaceGrouping::Collection => Grouping::Collection,
            WorkspaceGrouping::Tag => Grouping::Tag,
            WorkspaceGrouping::ReadingState => Grouping::Reading,
        };
        self.grouping.collapsed.clear();
        self.sort = view.sort;
        self.library_density = view.density;
        self.reader_focus = view.focus;
        self.reader_panel_width = view.panel_width as f32;
        self.reader_panel_restore = true;
        self.library_columns = view.visible_columns.clone();
        self.offset = 0;
        self.check_collection_filter();
        self.check_tag_filters();
        self.refresh();
        self.persist_library_state();
        Ok(())
    }
    pub(in crate::app) fn workspace_density_options(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.separator();
        ui.label(self.tr("Tamaño de las portadas", "Cover size"));
        for (density, es, en) in [
            (WorkspaceDensity::Comfortable, "Amplias", "Comfortable"),
            (WorkspaceDensity::Compact, "Compactas", "Compact"),
        ] {
            if theme::navigation(ui, self.tr(es, en), self.library_density == density, p).clicked()
            {
                self.library_density = density;
                self.persist_library_state();
                ui.close();
            }
        }
    }
}
