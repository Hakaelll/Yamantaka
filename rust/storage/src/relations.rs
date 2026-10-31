use crate::{Database, database::storage};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use yamantaka_domain::{
    Cancellation, Failure, MAX_LIBRARY_PAGE, Result, relations::*, search::IndexAccess,
};

pub(crate) fn exists(db: &Connection, id: &str) -> Result<()> {
    if !db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE id=?)",
            [id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(storage)?
    {
        return Err(Failure::Invalid(
            "Document is no longer in the library".into(),
        ));
    }
    Ok(())
}
fn record(row: &rusqlite::Row<'_>) -> rusqlite::Result<Relation> {
    let kind: String = row.get(3)?;
    let kind = RelationKind::parse(&kind).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Relation {
        id: row.get(0)?,
        source: row.get(1)?,
        target: row.get(2)?,
        kind,
        source_anchor_kind: row.get(4)?,
        source_anchor_id: row.get(5)?,
        target_anchor_kind: row.get(6)?,
        target_anchor_id: row.get(7)?,
        created_at: row.get(8)?,
    })
}
const FIELDS: &str = "r.id,r.source_document_id,r.target_document_id,r.relation_type,r.source_anchor_kind,r.source_anchor_id,r.target_anchor_kind,r.target_anchor_id,r.created_at";
impl Database {
    pub(crate) fn relation_page(
        &self,
        query: &RelationQuery,
        cancel: &Cancellation,
    ) -> Result<RelationPage> {
        cancel.check()?;
        exists(&self.connection, &query.document)?;
        let offset = i64::try_from(query.offset)
            .map_err(|_| Failure::Invalid("Relation offset is too large".into()))?;
        let total=self.connection.query_row("SELECT count(*) FROM document_relations WHERE source_document_id=?1 OR target_document_id=?1",[&query.document],|row|row.get::<_,i64>(0)).map_err(storage)?;
        let mut statement=self.connection.prepare(&format!("SELECT {FIELDS},CASE WHEN r.source_document_id=?1 THEN COALESCE(td.title_override,td.title,td.source_title,td.file_name) ELSE COALESCE(sd.title_override,sd.title,sd.source_title,sd.file_name) END FROM document_relations r JOIN documents sd ON sd.id=r.source_document_id JOIN documents td ON td.id=r.target_document_id WHERE r.source_document_id=?1 OR r.target_document_id=?1 ORDER BY r.created_at DESC,r.id LIMIT ?2 OFFSET ?3")).map_err(storage)?;
        let rows = statement
            .query_map(
                params![query.document, MAX_LIBRARY_PAGE as i64, offset],
                |row| {
                    let relation = record(row)?;
                    let backlink = relation.target == query.document;
                    Ok(RelationEntry {
                        other_document: if backlink {
                            relation.source.clone()
                        } else {
                            relation.target.clone()
                        },
                        relation,
                        title: row.get(9)?,
                        backlink,
                    })
                },
            )
            .map_err(storage)?;
        let mut entries = Vec::new();
        for row in rows {
            cancel.check()?;
            entries.push(row.map_err(storage)?);
        }
        Ok(RelationPage {
            entries,
            total: total.max(0) as u64,
        })
    }
    pub(crate) fn apply_relation(
        &mut self,
        change: &RelationChange,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<String> {
        cancel.check()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let (id, source) = match change {
            RelationChange::Create {
                source,
                target,
                kind,
            } => {
                if source == target {
                    return Err(Failure::Invalid(
                        "A document cannot relate to itself".into(),
                    ));
                }
                exists(&tx, source)?;
                exists(&tx, target)?;
                let existing=tx.query_row("SELECT id FROM document_relations WHERE source_document_id=? AND target_document_id=? AND relation_type=?",params![source,target,kind.stored()],|row|row.get::<_,String>(0)).optional().map_err(storage)?;
                let id = if let Some(id) = existing {
                    id
                } else {
                    let id = uuid::Uuid::new_v4().to_string();
                    tx.execute("INSERT INTO document_relations(id,source_document_id,target_document_id,relation_type,created_at) VALUES(?,?,?,?,?)",params![id,source,target,kind.stored(),chrono::Utc::now().to_rfc3339()]).map_err(storage)?;
                    id
                };
                (id, source.clone())
            }
            RelationChange::Remove { document, expected } => {
                exists(&tx, document)?;
                let current = tx
                    .query_row(
                        &format!("SELECT {FIELDS} FROM document_relations r WHERE r.id=?"),
                        [&expected.id],
                        record,
                    )
                    .optional()
                    .map_err(storage)?;
                if current.as_ref() != Some(expected)
                    || document != &expected.source && document != &expected.target
                {
                    return Err(Failure::Invalid(
                        "Relation changed; refresh before removing it".into(),
                    ));
                }
                tx.execute("DELETE FROM document_relations WHERE id=?", [&expected.id])
                    .map_err(storage)?;
                (expected.id.clone(), expected.source.clone())
            }
        };
        crate::search::refresh_relations(&tx, &source, access)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(id)
    }
}
