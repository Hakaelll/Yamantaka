//! One catalogue item per actor iteration, with a fixed target snapshot.
use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, MAX_LIBRARY_PAGE, Result,
    collections::{CollectionChange, CollectionRevision},
    search::IndexAccess,
    tags::TagChange,
};
#[derive(Clone)]
pub enum BulkAction {
    Favorite(bool),
    Tag(String),
    Collection {
        expected: CollectionRevision,
        included: bool,
    },
    Archive,
}
pub struct BulkRequest {
    pub ids: Vec<String>,
    pub action: BulkAction,
    pub cancel: Cancellation,
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BulkReport {
    pub succeeded: Vec<String>,
    pub failed: Vec<String>,
    pub remaining: Vec<String>,
}
pub struct BulkReply {
    pub cancel: Cancellation,
    pub result: Result<BulkReport>,
}
pub(crate) struct BulkJob {
    request: BulkRequest,
    next: usize,
    succeeded: Vec<String>,
    failed: Vec<String>,
}
pub(crate) fn normalize_ids(ids: Vec<String>) -> Result<Vec<String>> {
    if ids.is_empty()
        || ids.len() > MAX_LIBRARY_PAGE
        || ids.iter().any(|id| id.trim().is_empty() || id.len() > 4096)
    {
        return Err(Failure::Invalid(
            "Select between one and 80 catalogue books".into(),
        ));
    }
    let mut unique = Vec::with_capacity(ids.len());
    for id in ids {
        if !unique.contains(&id) {
            unique.push(id);
        }
    }
    Ok(unique)
}
impl BulkJob {
    pub(crate) fn start(mut request: BulkRequest) -> Result<Self> {
        let valid = |id: &str| !id.trim().is_empty() && id.len() <= 4096;
        let target_valid = match &request.action {
            BulkAction::Tag(id) => valid(id),
            BulkAction::Collection { expected, .. } => {
                valid(&expected.id) && valid(&expected.updated_at)
            }
            _ => true,
        };
        if !target_valid {
            return Err(Failure::Invalid(
                "Choose a valid organization target".into(),
            ));
        }
        request.ids = normalize_ids(request.ids)?;
        Ok(Self {
            request,
            next: 0,
            succeeded: Vec::new(),
            failed: Vec::new(),
        })
    }
    pub(crate) fn step(
        &mut self,
        repository: &mut impl LibraryRepository,
        access: IndexAccess,
    ) -> bool {
        if self.request.cancel.is_cancelled() || self.next == self.request.ids.len() {
            return true;
        }
        let id = &self.request.ids[self.next];
        let result = match &self.request.action {
            BulkAction::Favorite(value) => repository.set_favorite(id, *value),
            BulkAction::Tag(tag) => repository
                .change_tag(
                    &TagChange::SetMembership {
                        tag: tag.clone(),
                        document: id.clone(),
                        included: true,
                    },
                    access,
                    &self.request.cancel,
                )
                .map(|_| ()),
            BulkAction::Collection { expected, included } => repository
                .change_collection(
                    &CollectionChange::SetMembership {
                        expected: expected.clone(),
                        document: id.clone(),
                        included: *included,
                    },
                    &self.request.cancel,
                )
                .map(|_| ()),
            BulkAction::Archive => repository
                .change_collection(
                    &CollectionChange::EnsureForDocument {
                        name: "Archivado".into(),
                        icon: "archive".into(),
                        document: id.clone(),
                    },
                    &self.request.cancel,
                )
                .map(|_| ()),
        };
        if result.is_ok() {
            self.succeeded.push(id.clone());
        } else {
            self.failed.push(id.clone());
        }
        self.next += 1;
        self.next == self.request.ids.len()
    }
    pub(crate) fn reply(self) -> BulkReply {
        BulkReply {
            cancel: self.request.cancel,
            result: Ok(BulkReport {
                succeeded: self.succeeded,
                failed: self.failed,
                remaining: self.request.ids[self.next..].to_vec(),
            }),
        }
    }
}

#[cfg(test)]
mod tests;
