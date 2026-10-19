use super::*;
use yamantaka_domain::collections::{
    Collection, CollectionChange, CollectionPage, CollectionQuery,
};
mod preview;
mod view;

#[derive(Default)]
pub(super) struct CollectionState {
    entries: Vec<Collection>,
    query: CollectionQuery,
    total: u64,
    pending: Option<Cancellation>,
    writing: Option<(Cancellation, CollectionChange)>,
    filter_check: Option<(String, Cancellation)>,
    filter_checked: bool,
    loaded: bool,
    failed: bool,
    write_failed: bool,
    draft: String,
    editing: Option<(String, String)>,
    deleting: Option<Collection>,
    context: Option<Document>,
    pub(super) filter: Option<(String, String)>,
    preview: Option<(u8, u8)>,
}
impl CollectionState {
    pub(super) fn new(preferences: &Preferences) -> Self {
        let filter = preferences
            .get("libraryCollection")
            .and_then(|value| value.as_object())
            .and_then(|value| {
                Some((
                    value.get("id")?.as_str()?.to_owned(),
                    value.get("name")?.as_str()?.to_owned(),
                ))
            })
            .filter(|(id, name)| !id.is_empty() && id.len() <= 4096 && name.len() <= 4096);
        let mut state = Self::default();
        state.filter = filter;
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
        if let Some((_, cancel)) = &self.filter_check {
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
        serde_json::json!({"loaded":self.loaded,"failed":self.failed || self.write_failed,"total":self.total,"visible":self.entries.len(),"assigned":self.entries.iter().filter(|item|item.contains_document).count(),"pinned":self.entries.iter().filter(|item|item.pinned).count(),"filter":self.filter.as_ref().map(|(_,name)|name),"filterChecked":self.filter_checked,"confirmDelete":self.deleting.is_some(),"busy":!self.ready()})
    }
}
impl Drop for CollectionState {
    fn drop(&mut self) {
        self.cancel();
    }
}
impl YamantakaApp {
    pub(super) fn bulk_collections_changed(&mut self) {
        self.collections.loaded = false;
        if matches!(self.surface, Surface::Collections) {
            self.load_collections();
        }
    }

    pub(super) fn show_collections(&mut self, document: Option<Document>) {
        self.show_surface(Surface::Collections);
        if !matches!(self.surface, Surface::Collections) {
            return;
        }
        self.collections.context = document;
        self.collections.query = CollectionQuery {
            document: self.collections.context.as_ref().map(|doc| doc.id.clone()),
            ..Default::default()
        };
        self.collections.entries.clear();
        self.collections.editing = None;
        self.collections.deleting = None;
        self.load_collections();
    }
    fn load_collections(&mut self) {
        if let Some(previous) = self.collections.pending.take() {
            previous.cancel();
        }
        self.collections.loaded = false;
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::Collections(
            self.collections.query.clone(),
            cancel.clone(),
        )) {
            Ok(()) => {
                if let Some(previous) = self.collections.pending.replace(cancel) {
                    previous.cancel();
                }
                self.collections.loaded = false;
            }
            Err(error) => {
                self.collections.failed = true;
                self.error = Some(error.to_string());
            }
        }
    }
    pub(super) fn check_collection_filter(&mut self) {
        let Some((id, _)) = &self.collections.filter else {
            return;
        };
        let id = id.clone();
        self.collections.filter_checked = false;
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::Collections(
            CollectionQuery {
                id: Some(id.clone()),
                ..Default::default()
            },
            cancel.clone(),
        )) {
            Ok(()) => self.collections.filter_check = Some((id, cancel)),
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn accept_collections(
        &mut self,
        result: yamantaka_domain::Result<CollectionPage>,
        cancel: Cancellation,
    ) {
        if self
            .collections
            .filter_check
            .as_ref()
            .is_some_and(|(_, scope)| scope.same_request(&cancel))
        {
            let (id, _) = self.collections.filter_check.take().unwrap();
            if self
                .collections
                .filter
                .as_ref()
                .is_some_and(|(current, _)| *current == id)
                && let Ok(page) = result
            {
                self.collections.filter_checked = true;
                self.collections.filter = page
                    .entries
                    .into_iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| (entry.id, entry.name));
                self.refresh();
                self.persist_library_state();
            }
            return;
        }
        if !self
            .collections
            .pending
            .as_ref()
            .is_some_and(|scope| scope.same_request(&cancel) && !scope.is_cancelled())
        {
            return;
        }
        self.collections.pending = None;
        self.collections.loaded = true;
        match result {
            Ok(page) => {
                self.collections.failed = false;
                if page.total <= self.collections.query.offset as u64
                    && self.collections.query.offset > 0
                {
                    self.collections.query.offset =
                        (page.total.saturating_sub(1) as usize / 80) * 80;
                    self.load_collections();
                    return;
                }
                self.collections.entries = page.entries;
                self.collections.total = page.total;
            }
            Err(_) => self.collections.failed = true,
        }
    }
    fn change_collection(&mut self, change: CollectionChange) {
        if self.collections.writing.is_some() {
            return;
        }
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::ChangeCollection(
            Box::new(change.clone()),
            cancel.clone(),
        )) {
            Ok(()) => {
                self.collections.writing = Some((cancel, change));
                self.collections.write_failed = false;
            }
            Err(error) => {
                self.collections.failed = true;
                self.error = Some(error.to_string());
            }
        }
    }
    pub(super) fn accept_collection_change(
        &mut self,
        result: yamantaka_domain::Result<String>,
        cancel: Cancellation,
    ) {
        if !self
            .collections
            .writing
            .as_ref()
            .is_some_and(|(scope, _)| scope.same_request(&cancel))
        {
            return;
        }
        let (_, change) = self.collections.writing.take().unwrap();
        match result {
            Ok(id) => {
                match change {
                    CollectionChange::Create { .. } => self.collections.draft.clear(),
                    CollectionChange::Rename { name, .. } => {
                        if let Some((current, value)) = &mut self.collections.filter
                            && *current == id
                        {
                            *value = name.trim().into();
                        }
                    }
                    CollectionChange::Delete { .. }
                        if self
                            .collections
                            .filter
                            .as_ref()
                            .is_some_and(|(current, _)| *current == id) =>
                    {
                        self.collections.filter = None;
                    }
                    _ => {}
                }
                self.collections.failed = false;
                self.collections.editing = None;
                self.collections.deleting = None;
                self.refresh();
                self.persist_library_state();
            }
            Err(_) => {
                self.collections.write_failed = true;
                self.collections.deleting = None;
            }
        }
        if matches!(self.surface, Surface::Collections) {
            self.load_collections();
        } else {
            self.collections.loaded = false;
        }
    }
    fn apply_collection_filter(&mut self, collection: &Collection) {
        self.clear_collection_filter();
        self.collections.filter = Some((collection.id.clone(), collection.name.clone()));
        self.collections.filter_checked = true;
        self.offset = 0;
        self.inspector = false;
        self.show_surface(Surface::Library);
        self.refresh();
        self.persist_library_state();
    }
    pub(super) fn clear_collection_filter(&mut self) {
        if let Some((_, cancel)) = self.collections.filter_check.take() {
            cancel.cancel();
        }
        self.collections.filter = None;
    }
    pub(super) fn collection_filter(&mut self, ui: &mut egui::Ui, p: Palette) {
        let Some((_, name)) = self.collections.filter.clone() else {
            return;
        };
        let short = if name.chars().count() > 24 {
            format!("{}…", name.chars().take(24).collect::<String>())
        } else {
            name.clone()
        };
        if theme::navigation(ui, &short, true, p)
            .on_hover_text(&name)
            .clicked()
        {
            self.show_collections(None);
        }
        if theme::icon_button(
            ui,
            self.tr("Quitar filtro de colección", "Clear collection filter"),
            theme::Icon::Minus,
            p,
        )
        .clicked()
        {
            self.clear_collection_filter();
            self.offset = 0;
            self.refresh();
            self.persist_library_state();
        }
    }
}
