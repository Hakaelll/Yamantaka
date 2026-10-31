use super::*;
use std::path::PathBuf;
use yamantaka_domain::automation::simulation::*;
fn optional<'a>(row: &'a rusqlite::Row<'_>, index: usize) -> Result<Option<&'a str>> {
    match row.get_ref(index).map_err(storage)? {
        rusqlite::types::ValueRef::Null => Ok(None),
        value => value.as_str().map(Some).map_err(storage),
    }
}
impl RuleScanReader for Database {
    fn begin(
        &self,
        expected: &SavedRule,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<ScanBound> {
        cancel.check()?;
        self.connection
            .execute_batch("BEGIN DEFERRED")
            .map_err(storage)?;
        verify(&self.connection, expected)?;
        let mut statement = self
            .connection
            .prepare("SELECT id FROM documents ORDER BY id DESC LIMIT 1")
            .map_err(storage)?;
        let mut rows = statement.query([]).map_err(storage)?;
        let value = rows
            .next()
            .map_err(storage)?
            .map(|row| field(row, 0))
            .transpose()?;
        let bytes = value
            .map_or(0, str::len)
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(64))
            .ok_or(Failure::MemoryBudget)?;
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Cpu,
                bytes,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(ScanBound::new(value.map(str::to_owned), lease))
    }
    fn page(
        &self,
        after: Option<&str>,
        upper: &str,
        trigger: &RuleTrigger,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Vec<Arc<ScanEntry>>> {
        cancel.check()?;
        let value = trigger.value.as_deref().unwrap_or("").trim();
        let normalized = value.to_lowercase();
        let membership = match trigger.kind {
            TriggerKind::Tag => {
                "EXISTS(SELECT 1 FROM document_tags m JOIN tags t ON t.id=m.tag_id WHERE m.document_id=d.id AND (t.id=?4 OR (t.normalized_name=?5 AND NOT EXISTS(SELECT 1 FROM tags WHERE id=?4))))"
            }
            TriggerKind::Collection => {
                "EXISTS(SELECT 1 FROM collection_documents m JOIN collections c ON c.id=m.collection_id WHERE m.document_id=d.id AND (c.id=?4 OR (c.normalized_name=?5 AND NOT EXISTS(SELECT 1 FROM collections WHERE id=?4))))"
            }
            _ => "(?4 IS NOT NULL AND ?5 IS NULL)",
        };
        let mut statement=self.connection.prepare(&format!("SELECT d.id,COALESCE(d.title_override,d.title,d.source_title,d.file_name),d.path,d.normalized_path,d.file_name,COALESCE(d.author_override,d.author,d.source_author),d.is_missing,d.fingerprint,d.file_size,d.file_modified_at,{membership} FROM documents d WHERE (?1 IS NULL OR d.id>?1) AND d.id<=?2 ORDER BY d.id LIMIT ?3")).map_err(storage)?;
        let mut rows = statement
            .query(params![
                after,
                upper,
                MAX_LIBRARY_PAGE as i64,
                value,
                normalized
            ])
            .map_err(storage)?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next().map_err(storage)? {
            cancel.check()?;
            let mut bytes = std::mem::size_of::<ScanEntry>() + 128;
            for index in [0, 1, 2, 3, 4, 5, 7, 9] {
                bytes = bytes
                    .checked_add(
                        optional(row, index)?
                            .map_or(0, str::len)
                            .checked_mul(2)
                            .ok_or(Failure::MemoryBudget)?,
                    )
                    .ok_or(Failure::MemoryBudget)?;
            }
            let lease = governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Other,
                    MemoryRepresentation::Cpu,
                    bytes,
                ))
                .map_err(|_| Failure::MemoryBudget)?;
            let size = u64::try_from(row.get::<_, i64>(8).map_err(storage)?)
                .map_err(|_| Failure::Invalid("Invalid simulation file size".into()))?;
            let data = ScanData {
                id: field(row, 0)?.into(),
                title: field(row, 1)?.into(),
                path: PathBuf::from(field(row, 2)?),
                normalized_path: field(row, 3)?.into(),
                file_name: field(row, 4)?.into(),
                author: optional(row, 5)?.map(str::to_owned),
                missing: row.get(6).map_err(storage)?,
                fingerprint: field(row, 7)?.into(),
                size,
                modified: field(row, 9)?.into(),
                membership: row.get(10).map_err(storage)?,
            };
            entries.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
            entries.push(Arc::new(ScanEntry::new(data, lease)));
        }
        Ok(entries)
    }
}
