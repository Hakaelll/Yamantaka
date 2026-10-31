use super::repository::{AnnotationColumnValue, AnnotationRecordData};
use crate::{Failure, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnnotationRevision([u8; 32]);

struct DigestWriter(Sha256);
impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl DigestWriter {
    fn field(&mut self, bytes: &[u8]) {
        self.0.update((bytes.len() as u64).to_le_bytes());
        self.0.update(bytes);
    }
}
impl AnnotationRecordData {
    /// Catalogue titles are presentation data; annotation values and inherited columns are authoritative.
    pub(crate) fn calculate_revision(&self) -> Result<AnnotationRevision> {
        let mut digest = DigestWriter(Sha256::new());
        serde_json::to_writer(
            &mut digest,
            &(
                &self.id,
                &self.values,
                &self.tags,
                &self.created_at,
                &self.updated_at,
                &self.origin_document_id,
                &self.origin_fingerprint,
            ),
        )
        .map_err(|_| Failure::Invalid("Annotation revision could not be captured".into()))?;
        digest
            .0
            .update((self.extensions.len() as u64).to_le_bytes());
        for (name, value) in &self.extensions {
            digest.field(name.as_bytes());
            match value {
                AnnotationColumnValue::Null => digest.0.update([0]),
                AnnotationColumnValue::Integer(value) => {
                    digest.0.update([1]);
                    digest.0.update(value.to_le_bytes());
                }
                AnnotationColumnValue::Real(value) => {
                    digest.0.update([2]);
                    digest.0.update(value.to_bits().to_le_bytes());
                }
                AnnotationColumnValue::Text(value) => {
                    digest.0.update([3]);
                    digest.field(value.as_bytes());
                }
                AnnotationColumnValue::Blob(value) => {
                    digest.0.update([4]);
                    digest.field(value);
                }
            }
        }
        Ok(AnnotationRevision(digest.0.finalize().into()))
    }
}
