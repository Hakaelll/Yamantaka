use super::*;
use yamantaka_domain::relations::*;
mod location_edit;
mod preview;
mod revisions;
mod view;
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum RelationSection {
    #[default]
    Connections,
    Revisions,
}
#[derive(Default)]
pub(super) struct RelationState {
    revisions: revisions::RevisionState,
    section: RelationSection,
    pub(super) context_lookup: Option<(String, Cancellation)>,
    context: Option<Document>,
    page: Option<Arc<RelationPage>>,
    offset: usize,
    pending: Option<Cancellation>,
    writing: Option<Cancellation>,
    pub(super) chooser: catalogue_chooser::CatalogueChooser,
    target: Option<Document>,
    kind: RelationKind,
    source_page: Option<u32>,
    target_page: Option<u32>,
    preview_locations: u8,
    removing: Option<Relation>,
    editing: Option<location_edit::Editor>,
    pub(super) error: Option<String>,
    saved: bool,
    preview: u8,
    preview_review: bool,
}
impl RelationState {
    fn writing(&self) -> bool {
        self.writing.is_some() || self.revisions.writing()
    }
    pub(super) fn leave(&mut self) {
        self.revisions.leave();
        if let Some((_, cancel)) = self.context_lookup.take() {
            cancel.cancel();
        }
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
        self.page = None;
        self.editing = None;
        self.chooser = Default::default();
    }
    pub(super) fn ready(&self) -> bool {
        self.context_lookup.is_none()
            && self.pending.is_none()
            && !self.writing()
            && self.revisions.ready()
            && (self.preview == 0 || self.preview == 10)
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"document":self.context.as_ref().map(|doc|&doc.id),"pending":self.pending.is_some(),"writing":self.writing.is_some(),"saved":self.saved,"failed":self.error.is_some(),"phase":self.preview,"offset":self.offset,"total":self.page.as_ref().map(|page|page.total),"confirmRemove":self.removing.is_some(),"editingLocation":self.editing.is_some(),"entries":self.page.as_ref().map(|page|page.entries.iter().map(|entry|serde_json::json!({"id":entry.relation.id,"other":entry.other_document,"kind":entry.relation.kind.stored(),"backlink":entry.backlink,"sourceAnchor":entry.relation.source_anchor_id,"targetAnchor":entry.relation.target_anchor_id,"page":match entry.destination {RelationDestination::Page(page)=>Some(page),_=>None}})).collect::<Vec<_>>())})
    }
}
impl Drop for RelationState {
    fn drop(&mut self) {
        self.leave();
        if let Some(cancel) = &self.writing {
            cancel.cancel();
        }
    }
}
impl YamantakaApp {
    fn load_relation_context(&mut self, id: String) {
        if self.relations.writing() {
            return;
        }
        let cancel = Cancellation::default();
        if let Some((_, previous)) = self.relations.context_lookup.take() {
            previous.cancel();
        }
        match self
            .runtime
            .submit(Action::GetDocument(id.clone(), cancel.clone()))
        {
            Ok(()) => self.relations.context_lookup = Some((id, cancel)),
            Err(error) => self.relations.error = Some(error.to_string()),
        }
    }
    pub(super) fn accept_relation_context(
        &mut self,
        id: &str,
        document: Option<&Document>,
        cancel: &Cancellation,
    ) -> bool {
        if !self
            .relations
            .context_lookup
            .as_ref()
            .is_some_and(|(expected, token)| expected == id && token.same_request(cancel))
        {
            return false;
        }
        self.relations.context_lookup = None;
        if let Some(document) = document {
            self.show_relations(document.clone());
        } else {
            self.relations.error = Some(
                self.tr(
                    "El documento ya no está en la biblioteca.",
                    "The document is no longer in the library.",
                )
                .into(),
            );
        }
        true
    }
    pub(super) fn show_relations(&mut self, document: Document) {
        if self.relations.writing() {
            self.error = Some(
                self.tr(
                    "Espera a que termine el cambio de relación.",
                    "Wait for the relation change to finish.",
                )
                .into(),
            );
            return;
        }
        self.show_surface(Surface::Relations);
        if !matches!(self.surface, Surface::Relations) {
            return;
        }
        self.relations.leave();
        self.relations.context = Some(document);
        self.relations.offset = 0;
        self.relations.target = None;
        self.relations.source_page = None;
        self.relations.target_page = None;
        self.relations.removing = None;
        self.relations.error = None;
        self.relations.saved = false;
        match self.relations.section {
            RelationSection::Connections => self.load_relations(),
            RelationSection::Revisions => self.load_revision_groups(),
        }
    }
    fn load_relations(&mut self) {
        if let Some(previous) = self.relations.pending.take() {
            previous.cancel();
        }
        let Some(document) = self.relations.context.as_ref().map(|doc| doc.id.clone()) else {
            return;
        };
        let cancel = Cancellation::default();
        self.relations.error = None;
        match self.runtime.submit(Action::Relations(
            RelationQuery {
                document,
                offset: self.relations.offset,
            },
            cancel.clone(),
        )) {
            Ok(()) => self.relations.pending = Some(cancel),
            Err(error) => self.relations.error = Some(error.to_string()),
        }
    }
    pub(super) fn accept_relations(
        &mut self,
        result: yamantaka_domain::Result<RelationPage>,
        cancel: Cancellation,
    ) {
        if !self
            .relations
            .pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(&cancel))
        {
            return;
        }
        self.relations.pending = None;
        match result {
            Ok(page) if self.relations.offset > 0 && self.relations.offset as u64 >= page.total => {
                self.relations.offset =
                    usize::try_from(page.total.saturating_sub(1) / 80 * 80).unwrap_or(0);
                self.load_relations();
            }
            Ok(page) => self.relations.page = Some(Arc::new(page)),
            Err(error) => self.relations.error = Some(error.to_string()),
        }
    }
    fn open_relation(&mut self, entry: &RelationEntry) {
        let page = match entry.destination {
            RelationDestination::Page(page) => Some(page),
            _ => None,
        };
        let id = entry.other_document.clone();
        let cancel = self.open_intent.lookup(id.clone(), page);
        self.send(Action::GetDocument(id, cancel));
    }
    fn change_relation(&mut self, change: RelationChange) {
        if self.relations.writing() || self.relations.context_lookup.is_some() {
            return;
        }
        if let Some(cancel) = self.relations.pending.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        self.relations.saved = false;
        self.relations.error = None;
        match self
            .runtime
            .submit(Action::ChangeRelation(Box::new(change), cancel.clone()))
        {
            Ok(()) => self.relations.writing = Some(cancel),
            Err(error) => self.relations.error = Some(error.to_string()),
        }
    }
    pub(super) fn accept_relation_change(
        &mut self,
        result: yamantaka_domain::Result<String>,
        cancel: Cancellation,
    ) {
        if !self
            .relations
            .writing
            .as_ref()
            .is_some_and(|pending| pending.same_request(&cancel))
        {
            return;
        }
        self.relations.writing = None;
        match result {
            Ok(_) => {
                self.relations.saved = true;
                self.relations.editing = None;
                self.relations.removing = None;
                self.relations.target = None;
                if matches!(self.surface, Surface::Relations) {
                    self.load_relations();
                }
            }
            Err(error) => {
                self.relations.error = Some(error.to_string());
                self.error = Some(error.to_string());
            }
        }
    }
}
