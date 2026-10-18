use yamantaka_domain::{
    Cancellation, LibraryRepository, Result,
    metadata_review::{MetadataField, MetadataLocation, MetadataReview},
};

pub struct MetadataSourceRequest {
    pub expected: Box<MetadataReview>,
    pub field: MetadataField,
    pub cancel: Cancellation,
}
pub struct MetadataSourceReply {
    pub document: String,
    pub result: Result<MetadataLocation>,
    pub cancel: Cancellation,
}
impl MetadataSourceRequest {
    pub(crate) fn execute(self, repository: &impl LibraryRepository) -> MetadataSourceReply {
        let result = self
            .cancel
            .check()
            .and_then(|()| repository.metadata_source(&self.expected, self.field, &self.cancel));
        MetadataSourceReply {
            document: self.expected.document.clone(),
            result,
            cancel: self.cancel,
        }
    }
}
