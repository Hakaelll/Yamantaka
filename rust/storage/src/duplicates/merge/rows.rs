use super::*;
const TABLES: &[(&str, &str, &str)] = &[
    ("documents", "id IN (?1,?2)", "id"),
    (
        "document_enrichment",
        "document_id IN (?1,?2)",
        "document_id",
    ),
    (
        "document_topics",
        "document_id IN (?1,?2)",
        "document_id,topic_key",
    ),
    (
        "document_preferences",
        "document_id IN (?1,?2)",
        "document_id",
    ),
    ("document_notes", "document_id IN (?1,?2)", "document_id"),
    ("document_covers", "document_id IN (?1,?2)", "document_id"),
    ("bookmarks", "document_id IN (?1,?2)", "id"),
    ("reading_sessions", "document_id IN (?1,?2)", "id"),
    ("annotations", "document_id IN (?1,?2)", "id"),
    (
        "annotation_tags",
        "annotation_id IN (SELECT id FROM annotations WHERE document_id IN (?1,?2))",
        "annotation_id,tag_id",
    ),
    (
        "document_tags",
        "document_id IN (?1,?2)",
        "document_id,tag_id",
    ),
    (
        "collection_documents",
        "document_id IN (?1,?2)",
        "collection_id,document_id",
    ),
    ("workspace_tabs", "document_id IN (?1,?2)", "id"),
    (
        "workspaces",
        "id IN (SELECT workspace_id FROM workspace_tabs WHERE document_id IN (?1,?2))",
        "id",
    ),
    (
        "document_relations",
        "source_document_id IN (?1,?2) OR target_document_id IN (?1,?2)",
        "id",
    ),
    (
        "document_revision_members",
        "document_id IN (?1,?2)",
        "group_id,document_id",
    ),
    (
        "document_revision_groups",
        "current_document_id IN (?1,?2) OR id IN (SELECT group_id FROM document_revision_members WHERE document_id IN (?1,?2))",
        "id",
    ),
    (
        "search_index_state",
        "document_id IN (?1,?2)",
        "document_id",
    ),
    (
        "app_metadata",
        "key IN ('reading_goals_v1','rust.frontMatterEvidence.v1:'||?1,'rust.frontMatterEvidence.v1:'||?2)",
        "key",
    ),
];
pub(super) fn revision(
    db: &Connection,
    selection: &Selection,
    cancel: &Cancellation,
) -> Result<([u8; 32], BTreeMap<&'static str, u64>)> {
    let mut hash = Sha256::new();
    let mut counts = BTreeMap::new();
    for &(table, condition, order) in TABLES {
        cancel.check()?;
        hash.update((table.len() as u64).to_le_bytes());
        hash.update(table.as_bytes());
        let mut statement = db
            .prepare(&format!(
                "SELECT * FROM {table} WHERE {condition} ORDER BY {order}"
            ))
            .map_err(storage)?;
        let columns = statement.column_count();
        let mut rows = statement
            .query(params![
                selection.kept.document,
                selection.incoming.document
            ])
            .map_err(storage)?;
        let mut count = 0u64;
        while let Some(row) = rows.next().map_err(storage)? {
            cancel.check()?;
            count += 1;
            if count > 100000 {
                return Err(invalid("Duplicate review exceeds supported record limits"));
            }
            hash.update([1]);
            hash.update((columns as u64).to_le_bytes());
            for index in 0..columns {
                let value = row.get_ref(index).map_err(storage)?;
                match value {
                    ValueRef::Null => hash.update([0]),
                    ValueRef::Integer(value) => {
                        hash.update([1]);
                        hash.update(value.to_le_bytes());
                    }
                    ValueRef::Real(value) => {
                        hash.update([2]);
                        hash.update(value.to_bits().to_le_bytes());
                    }
                    ValueRef::Text(value) | ValueRef::Blob(value) => {
                        if value.len() > 4 * 1024 * 1024 {
                            return Err(invalid(
                                "Duplicate review record exceeds supported bounds",
                            ));
                        }
                        hash.update([
                            if matches!(value_type(row, index)?, rusqlite::types::Type::Text) {
                                3
                            } else {
                                4
                            },
                        ]);
                        hash.update((value.len() as u64).to_le_bytes());
                        hash.update(value);
                    }
                }
            }
        }
        hash.update([0]);
        counts.insert(table, count);
    }
    Ok((hash.finalize().into(), counts))
}
fn value_type(row: &rusqlite::Row<'_>, index: usize) -> Result<rusqlite::types::Type> {
    Ok(row.get_ref(index).map_err(storage)?.data_type())
}
