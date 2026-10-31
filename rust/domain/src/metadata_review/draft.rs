use crate::{Failure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetadataDraft {
    pub document: String,
    pub revision: String,
    pub title: String,
    pub author: String,
}
impl MetadataDraft {
    pub fn validate(&self) -> Result<()> {
        if self.document.is_empty()
            || self.document.len() > 1024
            || self.revision.len() != 64
            || !self.revision.bytes().all(|byte| byte.is_ascii_hexdigit())
            || self.title.len() > 65536
            || self.author.len() > 65536
        {
            return Err(Failure::Invalid(
                "Invalid metadata recovery draft; original session is preserved".into(),
            ));
        }
        Ok(())
    }
}
