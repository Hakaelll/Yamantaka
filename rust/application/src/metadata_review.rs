use yamantaka_domain::{
    Cancellation, LibraryRepository, Result,
    metadata_review::{MetadataMutation, MetadataReview},
    search::IndexAccess,
};

pub enum MetadataChange {
    Read,
    Apply {
        expected: Box<MetadataReview>,
        mutation: MetadataMutation,
    },
}
pub struct MetadataRequest {
    pub document: String,
    pub change: MetadataChange,
    pub cancel: Cancellation,
}
pub struct MetadataReply {
    pub document: String,
    pub writing: bool,
    pub cancel: Cancellation,
    pub result: Result<MetadataReview>,
}
impl MetadataRequest {
    pub(crate) fn execute(
        self,
        repository: &mut impl LibraryRepository,
        access: IndexAccess,
    ) -> MetadataReply {
        let writing = matches!(&self.change, MetadataChange::Apply { .. });
        let result = self.cancel.check().and_then(|()| match &self.change {
            MetadataChange::Read => repository.metadata_review(&self.document, &self.cancel),
            MetadataChange::Apply { expected, mutation } => {
                if expected.document != self.document {
                    return Err(yamantaka_domain::Failure::Invalid(
                        "Metadata request belongs to another document".into(),
                    ));
                }
                repository.change_metadata(expected, mutation, access, &self.cancel)
            }
        });
        MetadataReply {
            document: self.document,
            writing,
            cancel: self.cancel,
            result,
        }
    }
}
