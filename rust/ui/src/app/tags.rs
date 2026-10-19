use super::*;
use yamantaka_domain::tags::{MAX_TAG_FILTERS, Tag, TagChange, TagPage, TagQuery};
mod preview;
mod view;

#[derive(Default)]
pub(super) struct TagState {
    entries: Vec<Tag>,
    query: TagQuery,
    total: u64,
    pending: Option<Cancellation>,
    writing: Option<(Cancellation, TagChange)>,
    filter_check: Option<Cancellation>,
    filters_checked: bool,
    loaded: bool,
    failed: bool,
    write_failed: bool,
    draft: String,
    context: Option<Document>,
    pub(super) filters: Vec<(String, String)>,
    preview: Option<(u8, u8)>,
}
impl TagState {
    pub(super) fn new(preferences: &Preferences) -> Self {
        let mut state = Self::default();
        if let Some(values) = preferences
            .get("libraryTags")
            .and_then(serde_json::Value::as_array)
        {
            for value in values.iter().take(MAX_TAG_FILTERS) {
                let Some(id) = value
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .filter(|id| !id.is_empty() && id.len() <= 4096)
                else {
                    continue;
                };
                let Some(name) = value
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .filter(|name| name.len() <= 4096)
                else {
                    continue;
                };
                if !state.filters.iter().any(|(existing, _)| existing == id) {
                    state.filters.push((id.into(), name.into()));
                }
            }
        }
        state
    }
    pub(super) fn leave(&mut self) {
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
        self.loaded = false;
    }
    pub(super) fn cancel(&mut self) {
        self.leave();
        if let Some((cancel, _)) = &self.writing {
            cancel.cancel();
        }
        if let Some(cancel) = &self.filter_check {
            cancel.cancel();
        }
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none()
            && self.writing.is_none()
            && self.filter_check.is_none()
            && self.preview.is_none()
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"loaded":self.loaded,"failed":self.failed||self.write_failed,"total":self.total,"visible":self.entries.len(),"assigned":self.entries.iter().filter(|tag|tag.contains_document).count(),"annotationUses":self.entries.iter().map(|tag|tag.annotation_count).sum::<u64>(),"filters":self.filters.iter().map(|(_,name)|name).collect::<Vec<_>>(),"filtersChecked":self.filters_checked,"busy":!self.ready()})
    }
}
impl Drop for TagState {
    fn drop(&mut self) {
        self.cancel();
    }
}
impl YamantakaApp {
    pub(super) fn show_tags(&mut self, document: Option<Document>) {
        self.show_surface(Surface::Tags);
        if !matches!(self.surface, Surface::Tags) {
            return;
        }
        self.library_tags.context = document;
        self.library_tags.query = TagQuery {
            document: self.library_tags.context.as_ref().map(|doc| doc.id.clone()),
            ..Default::default()
        };
        self.library_tags.entries.clear();
        self.load_tags();
    }
    fn load_tags(&mut self) {
        if let Some(cancel) = self.library_tags.pending.take() {
            cancel.cancel();
        }
        self.library_tags.loaded = false;
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::Tags(
            self.library_tags.query.clone(),
            cancel.clone(),
        )) {
            Ok(()) => self.library_tags.pending = Some(cancel),
            Err(error) => {
                self.library_tags.failed = true;
                self.error = Some(error.to_string());
            }
        }
    }
    pub(super) fn check_tag_filters(&mut self) {
        if self.library_tags.filters.is_empty() {
            return;
        }
        let ids = self
            .library_tags
            .filters
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        let cancel = Cancellation::default();
        self.library_tags.filters_checked = false;
        if let Some(previous) = self.library_tags.filter_check.take() {
            previous.cancel();
        }
        match self.runtime.submit(Action::Tags(
            TagQuery {
                ids,
                ..Default::default()
            },
            cancel.clone(),
        )) {
            Ok(()) => self.library_tags.filter_check = Some(cancel),
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn accept_tags(
        &mut self,
        result: yamantaka_domain::Result<TagPage>,
        cancel: Cancellation,
    ) {
        if self
            .library_tags
            .filter_check
            .as_ref()
            .is_some_and(|scope| scope.same_request(&cancel))
        {
            self.library_tags.filter_check = None;
            if let Ok(page) = result {
                self.library_tags.filters = self
                    .library_tags
                    .filters
                    .iter()
                    .filter_map(|(id, _)| {
                        page.entries
                            .iter()
                            .find(|tag| tag.id == *id)
                            .map(|tag| (tag.id.clone(), tag.name.clone()))
                    })
                    .collect();
                self.library_tags.filters_checked = true;
                self.offset = 0;
                self.refresh();
                self.persist_tag_filters();
            }
            return;
        }
        if !self
            .library_tags
            .pending
            .as_ref()
            .is_some_and(|scope| scope.same_request(&cancel) && !scope.is_cancelled())
        {
            return;
        }
        self.library_tags.pending = None;
        self.library_tags.loaded = true;
        match result {
            Ok(page) => {
                self.library_tags.failed = false;
                if page.total <= self.library_tags.query.offset as u64
                    && self.library_tags.query.offset > 0
                {
                    self.library_tags.query.offset =
                        (page.total.saturating_sub(1) as usize / 80) * 80;
                    self.load_tags();
                    return;
                }
                self.library_tags.entries = page.entries;
                self.library_tags.total = page.total;
            }
            Err(_) => self.library_tags.failed = true,
        }
    }
    fn change_tag(&mut self, change: TagChange) {
        if self.library_tags.writing.is_some() {
            return;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::ChangeTag(Box::new(change.clone()), cancel.clone()))
        {
            Ok(()) => {
                self.library_tags.writing = Some((cancel, change));
                self.library_tags.write_failed = false;
            }
            Err(error) => {
                self.library_tags.write_failed = true;
                self.error = Some(error.to_string());
            }
        }
    }
    pub(super) fn accept_tag_change(
        &mut self,
        result: yamantaka_domain::Result<String>,
        cancel: Cancellation,
    ) {
        if !self
            .library_tags
            .writing
            .as_ref()
            .is_some_and(|(scope, _)| scope.same_request(&cancel))
        {
            return;
        }
        let (_, change) = self.library_tags.writing.take().unwrap();
        match result {
            Ok(id) => {
                if matches!(
                    change,
                    TagChange::Ensure { .. } | TagChange::EnsureForDocument { .. }
                ) {
                    self.library_tags.draft.clear();
                    self.library_tags.query.ids = vec![id];
                    self.library_tags.query.text.clear();
                    self.library_tags.query.offset = 0;
                }
                self.library_tags.write_failed = false;
                self.refresh();
                if self.search.enabled {
                    self.send(Action::IndexStatistics);
                }
            }
            Err(_) => self.library_tags.write_failed = true,
        }
        if matches!(self.surface, Surface::Tags) {
            self.load_tags();
        } else {
            self.library_tags.loaded = false;
        }
    }
    fn toggle_tag_filter(&mut self, tag: &Tag) {
        let index = self
            .library_tags
            .filters
            .iter()
            .position(|(id, _)| *id == tag.id);
        if index.is_none() && self.library_tags.filters.len() >= MAX_TAG_FILTERS {
            self.error = Some(
                self.tr(
                    "Puedes seleccionar hasta 80 etiquetas para filtrar.",
                    "You can select up to 80 tag filters.",
                )
                .into(),
            );
            return;
        }
        if let Some(cancel) = self.library_tags.filter_check.take() {
            cancel.cancel();
        }
        if let Some(index) = index {
            self.library_tags.filters.remove(index);
        } else {
            self.library_tags
                .filters
                .push((tag.id.clone(), tag.name.clone()));
        }
        self.library_tags.filters_checked = true;
        self.offset = 0;
        self.refresh();
        self.persist_tag_filters();
    }
    pub(super) fn clear_tag_filters(&mut self) {
        if let Some(cancel) = self.library_tags.filter_check.take() {
            cancel.cancel();
        }
        self.library_tags.filters.clear();
        self.library_tags.filters_checked = true;
        self.persist_tag_filters();
    }
    fn persist_tag_filters(&mut self) {
        if self.demo {
            return;
        }
        let value = serde_json::json!(
            self.library_tags
                .filters
                .iter()
                .map(|(id, name)| serde_json::json!({"id":id,"name":name}))
                .collect::<Vec<_>>()
        );
        if self.preferences.get("libraryTags") != Some(&value) {
            self.preferences.rust.insert("libraryTags".into(), value);
            if let Err(error) = self.preferences.save(&self.store.join("preferences.json")) {
                self.error = Some(error.to_string());
            }
        }
    }
    pub(super) fn tag_filter_control(&mut self, ui: &mut egui::Ui, p: Palette) {
        let count = self.library_tags.filters.len();
        if count == 0 {
            return;
        }
        let label = if count == 1 {
            let name = &self.library_tags.filters[0].1;
            if name.chars().count() > 24 {
                format!("{}…", name.chars().take(24).collect::<String>())
            } else {
                name.clone()
            }
        } else {
            format!("{} ({count})", self.tr("Etiquetas", "Tags"))
        };
        if theme::navigation(ui, &label, true, p)
            .on_hover_text(self.tr("Gestionar filtros de etiquetas", "Manage tag filters"))
            .clicked()
        {
            self.show_tags(None);
        }
        if theme::icon_button(
            ui,
            self.tr("Quitar filtros de etiquetas", "Clear tag filters"),
            theme::Icon::Minus,
            p,
        )
        .clicked()
        {
            self.clear_tag_filters();
            self.offset = 0;
            self.refresh();
        }
    }
}
