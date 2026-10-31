use super::*;
use yamantaka_domain::revisions::*;
mod preview;
mod view;
#[derive(Default)]
pub(super) struct RevisionState {
    groups: Option<Arc<RevisionPage>>,
    members: Option<Arc<RevisionMemberPage>>,
    selected: Option<RevisionGroup>,
    select_after: Option<String>,
    group_pending: Option<Cancellation>,
    member_pending: Option<Cancellation>,
    writing: Option<(Cancellation, bool)>,
    offset: usize,
    member_offset: usize,
    name: String,
    creation_open: bool,
    error: Option<String>,
    saved: bool,
    preview: u8,
    preview_mode: u8,
}
impl RevisionState {
    pub(super) fn leave(&mut self) {
        if let Some(cancel) = self.group_pending.take() {
            cancel.cancel();
        }
        if let Some(cancel) = self.member_pending.take() {
            cancel.cancel();
        }
        self.groups = None;
        self.members = None;
        self.selected = None;
        self.select_after = None;
        self.offset = 0;
        self.member_offset = 0;
    }
    pub(super) fn writing(&self) -> bool {
        self.writing.is_some()
    }
    fn busy(&self) -> bool {
        self.writing() || self.group_pending.is_some() || self.member_pending.is_some()
    }
    pub(super) fn ready(&self) -> bool {
        !self.busy() && matches!(self.preview, 0 | 10)
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"phase":self.preview,"pending":self.busy(),"failed":self.error.is_some(),"total":self.groups.as_ref().map(|page|page.total),"selected":self.selected.as_ref().map(|group|&group.id),"current":self.members.as_ref().and_then(|page|page.current.as_ref()),"memberTotal":self.members.as_ref().map(|page|page.total),"members":self.members.as_ref().map(|page|page.entries.iter().map(|member|serde_json::json!({"id":member.document,"title":member.title,"missing":member.missing})).collect::<Vec<_>>()),"groups":self.groups.as_ref().map(|page|page.entries.iter().map(|group|serde_json::json!({"id":group.id,"name":group.name,"current":group.current,"members":group.member_count,"created":group.created_at,"updated":group.updated_at})).collect::<Vec<_>>())})
    }
}
impl Drop for RevisionState {
    fn drop(&mut self) {
        self.leave();
        if let Some((cancel, _)) = &self.writing {
            cancel.cancel();
        }
    }
}
impl YamantakaApp {
    pub(in crate::app) fn load_revision_groups(&mut self) {
        let Some(document) = self.relations.context.as_ref().map(|doc| doc.id.clone()) else {
            return;
        };
        if let Some(cancel) = self.relations.revisions.group_pending.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        let request = Action::RevisionGroups(
            RevisionQuery {
                document,
                offset: self.relations.revisions.offset,
            },
            cancel.clone(),
        );
        match self.runtime.submit(request) {
            Ok(()) => {
                self.relations.revisions.group_pending = Some(cancel);
                self.relations.revisions.error = None;
            }
            Err(error) => self.relations.revisions.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_revision_groups(
        &mut self,
        result: yamantaka_domain::Result<RevisionPage>,
        cancel: Cancellation,
    ) {
        let state = &mut self.relations.revisions;
        if !state
            .group_pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(&cancel))
        {
            return;
        }
        state.group_pending = None;
        match result {
            Ok(page) if state.offset > 0 && state.offset as u64 >= page.total => {
                state.offset = usize::try_from(page.total.saturating_sub(1) / 80 * 80).unwrap_or(0);
                self.load_revision_groups();
            }
            Ok(page) => {
                let selected = state
                    .select_after
                    .take()
                    .or_else(|| state.selected.as_ref().map(|group| group.id.clone()));
                state.selected = selected
                    .and_then(|id| page.entries.iter().find(|group| group.id == id).cloned());
                state.groups = Some(Arc::new(page));
                if state.selected.is_some() {
                    self.load_revision_members();
                } else {
                    state.members = None;
                }
            }
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    fn select_revision_group(&mut self, group: RevisionGroup) {
        if self.relations.revisions.writing() {
            return;
        }
        self.relations.revisions.selected = Some(group);
        self.relations.revisions.member_offset = 0;
        self.load_revision_members();
    }
    fn load_revision_members(&mut self) {
        let state = &mut self.relations.revisions;
        let Some(group) = state.selected.as_ref().map(|group| group.id.clone()) else {
            return;
        };
        if let Some(cancel) = state.member_pending.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::RevisionMembers(
            RevisionMemberQuery {
                group,
                offset: state.member_offset,
            },
            cancel.clone(),
        )) {
            Ok(()) => {
                state.member_pending = Some(cancel);
                state.error = None;
            }
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_revision_members(
        &mut self,
        result: yamantaka_domain::Result<RevisionMemberPage>,
        cancel: Cancellation,
    ) {
        let state = &mut self.relations.revisions;
        if !state
            .member_pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(&cancel))
        {
            return;
        }
        state.member_pending = None;
        match result {
            Ok(page)
                if state.selected.as_ref().is_none_or(|group| {
                    group.id != page.group || group.updated_at != page.updated_at
                }) =>
            {
                self.load_revision_groups();
            }
            Ok(page) if state.member_offset > 0 && state.member_offset as u64 >= page.total => {
                state.member_offset =
                    usize::try_from(page.total.saturating_sub(1) / 80 * 80).unwrap_or(0);
                self.load_revision_members();
            }
            Ok(page) => state.members = Some(Arc::new(page)),
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    fn change_revision_group(&mut self, change: RevisionChange) {
        if self.relations.writing() || self.relations.context_lookup.is_some() {
            return;
        }
        let state = &mut self.relations.revisions;
        if let Some(cancel) = state.group_pending.take() {
            cancel.cancel();
        }
        if let Some(cancel) = state.member_pending.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        state.saved = false;
        state.error = None;
        let creating = matches!(change, RevisionChange::Create { .. });
        match self
            .runtime
            .submit(Action::ChangeRevision(Box::new(change), cancel.clone()))
        {
            Ok(()) => state.writing = Some((cancel, creating)),
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_revision_change(
        &mut self,
        result: yamantaka_domain::Result<String>,
        cancel: Cancellation,
    ) {
        let state = &mut self.relations.revisions;
        if !state
            .writing
            .as_ref()
            .is_some_and(|(pending, _)| pending.same_request(&cancel))
        {
            return;
        }
        let (_, creating) = state.writing.take().unwrap();
        match result {
            Ok(id) => {
                state.saved = true;
                if creating {
                    state.name.clear();
                    state.creation_open = false;
                    self.relations.target = None;
                }
                state.select_after = Some(id);
                state.offset = 0;
                if matches!(self.surface, Surface::Relations)
                    && self.relations.section == RelationSection::Revisions
                {
                    self.load_revision_groups();
                }
            }
            Err(error) => {
                state.error = Some(error.to_string());
                self.error = Some(error.to_string());
            }
        }
    }
}
