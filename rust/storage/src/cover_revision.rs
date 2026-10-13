use crate::database::storage;
use rusqlite::{Connection, OptionalExtension, types::ValueRef};
use sha2::{Digest, Sha256};
use yamantaka_domain::{Result, cover_revision::CoverRevision};

pub(crate) fn read(connection: &Connection, id: &str) -> Result<Option<CoverRevision>> {
    connection.query_row("SELECT document_id,source_page_index,small_thumbnail_path,large_thumbnail_path,generated_at,source_modified_at,confidence,selection_mode,algorithm_version,crop_mode,page_aspect_ratio FROM document_covers WHERE document_id=?1",[id],|row| {
        let mut hash=Sha256::new();
        for index in 0..11 {
            match row.get_ref(index)? {
                ValueRef::Null=>hash.update([0]),
                ValueRef::Integer(value)=>{hash.update([1]);hash.update(value.to_le_bytes());},
                ValueRef::Real(value)=>{hash.update([2]);hash.update(value.to_bits().to_le_bytes());},
                ValueRef::Text(value)=>{hash.update([3]);hash.update((value.len() as u64).to_le_bytes());hash.update(value);},
                ValueRef::Blob(value)=>{hash.update([4]);hash.update((value.len() as u64).to_le_bytes());hash.update(value);},
            }
        }
        Ok(CoverRevision::from_digest(hash.finalize().into()))
    }).optional().map_err(storage)
}
