use crate::{Database, database::storage};
use rusqlite::params_from_iter;
use yamantaka_domain::{Cancellation, Failure, LibraryMembership, MAX_LIBRARY_PAGE, Result};

impl Database {
    pub(crate) fn group_memberships(
        &self,
        ids: &[String],
        cancel: &Cancellation,
    ) -> Result<Vec<LibraryMembership>> {
        cancel.check()?;
        if ids.len() > MAX_LIBRARY_PAGE {
            return Err(Failure::Invalid(
                "Group only the current catalogue page".into(),
            ));
        }
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let sql=format!("SELECT d.id,
            (SELECT c.name FROM collection_documents m JOIN collections c ON c.id=m.collection_id WHERE m.document_id=d.id ORDER BY c.is_pinned DESC,c.sort_order,c.name COLLATE NOCASE,c.id LIMIT 1),
            (SELECT t.name FROM document_tags m JOIN tags t ON t.id=m.tag_id WHERE m.document_id=d.id ORDER BY t.name COLLATE NOCASE,t.id LIMIT 1)
            FROM documents d WHERE d.id IN ({})",vec!["?";ids.len()].join(","));
        let mut statement = self.connection.prepare(&sql).map_err(storage)?;
        let rows = statement
            .query_map(params_from_iter(ids), |row| {
                Ok(LibraryMembership {
                    document_id: row.get(0)?,
                    collection: row.get(1)?,
                    tag: row.get(2)?,
                })
            })
            .map_err(storage)?;
        let mut result = Vec::new();
        for row in rows {
            cancel.check()?;
            result.push(row.map_err(storage)?);
        }
        cancel.check()?;
        Ok(result)
    }
}
