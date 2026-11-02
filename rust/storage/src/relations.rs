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
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        exists(&tx, &query.document)?;
        let offset = i64::try_from(query.offset)
            .map_err(|_| Failure::Invalid("Relation offset is too large".into()))?;
        let total=tx.query_row("SELECT count(*) FROM document_relations WHERE source_document_id=?1 OR target_document_id=?1",[&query.document],|row|row.get::<_,i64>(0)).map_err(storage)?;
        let mut statement=tx.prepare(&format!("SELECT {FIELDS},CASE WHEN r.source_document_id=?1 THEN COALESCE(td.title_override,td.title,td.source_title,td.file_name) ELSE COALESCE(sd.title_override,sd.title,sd.source_title,sd.file_name) END FROM document_relations r JOIN documents sd ON sd.id=r.source_document_id JOIN documents td ON td.id=r.target_document_id WHERE r.source_document_id=?1 OR r.target_document_id=?1 ORDER BY r.created_at DESC,r.id LIMIT ?2 OFFSET ?3")).map_err(storage)?;
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
                        destination: RelationDestination::Document,
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
        drop(statement);
        for entry in &mut entries {
            cancel.check()?;
            let (kind, id) = if entry.backlink {
                (
                    &entry.relation.source_anchor_kind,
                    &entry.relation.source_anchor_id,
                )
            } else {
                (
                    &entry.relation.target_anchor_kind,
                    &entry.relation.target_anchor_id,
                )
            };
            entry.destination =
                destination(&tx, &entry.other_document, kind.as_deref(), id.as_deref())?;
        }
        tx.commit().map_err(storage)?;
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
                source_page,
                target_page,
            } => {
                if source == target {
                    return Err(Failure::Invalid(
                        "A document cannot relate to itself".into(),
                    ));
                }
                exists(&tx, source)?;
                exists(&tx, target)?;
                for (document, page) in [(source, source_page), (target, target_page)] {
                    if let Some(page) = page {
                        let count: i64 = tx
                            .query_row(
                                "SELECT page_count FROM documents WHERE id=?",
                                [document],
                                |row| row.get(0),
                            )
                            .map_err(storage)?;
                        if i64::from(*page) >= count {
                            return Err(Failure::Invalid(
                                "Connection page is outside the document; review its page count"
                                    .into(),
                            ));
                        }
                    }
                }
                let existing=tx.query_row("SELECT id FROM document_relations WHERE source_document_id=? AND target_document_id=? AND relation_type=?",params![source,target,kind.stored()],|row|row.get::<_,String>(0)).optional().map_err(storage)?;
                let id = if let Some(id) = existing {
                    if source_page.is_some() || target_page.is_some() {
                        let anchors: (Option<String>,Option<String>,Option<String>,Option<String>) = tx.query_row("SELECT source_anchor_kind,source_anchor_id,target_anchor_kind,target_anchor_id FROM document_relations WHERE id=?", [&id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).map_err(storage)?;
                        let expected = (
                            source_page.map(|_| "page".to_owned()),
                            source_page.map(|p| p.to_string()),
                            target_page.map(|_| "page".to_owned()),
                            target_page.map(|p| p.to_string()),
                        );
                        if anchors != expected {
                            return Err(Failure::Invalid("This connection already exists with different locations; review it before replacing it".into()));
                        }
                    }
                    id
                } else {
                    let id = uuid::Uuid::new_v4().to_string();
                    tx.execute("INSERT INTO document_relations(id,source_document_id,target_document_id,relation_type,source_anchor_kind,source_anchor_id,target_anchor_kind,target_anchor_id,created_at) VALUES(?,?,?,?,?,?,?,?,?)",params![id,source,target,kind.stored(),source_page.map(|_|"page"),source_page.map(|p|p.to_string()),target_page.map(|_|"page"),target_page.map(|p|p.to_string()),chrono::Utc::now().to_rfc3339()]).map_err(storage)?;
                    id
                };
                (id, source.clone())
            }
            RelationChange::UpdateLocation {
                document,
                expected,
                page,
            } => {
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
                    || (document != &expected.source && document != &expected.target)
                {
                    return Err(Failure::Invalid(
                        "Relation changed; refresh before editing its location".into(),
                    ));
                }
                if let Some(page) = page {
                    let count: i64 = tx
                        .query_row(
                            "SELECT page_count FROM documents WHERE id=?",
                            [document],
                            |row| row.get(0),
                        )
                        .map_err(storage)?;
                    if i64::from(*page) >= count {
                        return Err(Failure::Invalid(
                            "Relation page is outside this document".into(),
                        ));
                    }
                }
                let sql = if document == &expected.source {
                    "UPDATE document_relations SET source_anchor_kind=?1,source_anchor_id=?2 WHERE id=?3"
                } else {
                    "UPDATE document_relations SET target_anchor_kind=?1,target_anchor_id=?2 WHERE id=?3"
                };
                tx.execute(
                    sql,
                    params![
                        page.map(|_| "page"),
                        page.map(|value| value.to_string()),
                        expected.id
                    ],
                )
                .map_err(storage)?;
                (expected.id.clone(), expected.source.clone())
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

fn destination(
    db: &Connection,
    document: &str,
    kind: Option<&str>,
    id: Option<&str>,
) -> Result<RelationDestination> {
    let (kind, id) = match (kind, id) {
        (None, None) => return Ok(RelationDestination::Document),
        (Some(kind), Some(id)) => (kind, id),
        _ => return Ok(RelationDestination::Unavailable),
    };
    let page = match kind {
        "page" => id.parse::<u32>().ok().map(i64::from),
        "bookmark" => db
            .query_row(
                "SELECT page_index FROM bookmarks WHERE id=? AND document_id=?",
                params![id, document],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(storage)?,
        "annotation" => db
            .query_row(
                "SELECT page_index FROM annotations WHERE id=? AND document_id=? AND status='attached'",
                params![id, document],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(storage)?,
        _ => None,
    };
    let count: i64 = db
        .query_row(
            "SELECT page_count FROM documents WHERE id=?",
            [document],
            |row| row.get(0),
        )
        .map_err(storage)?;
    Ok(
        match page
            .and_then(|page| u32::try_from(page).ok())
            .filter(|page| i64::from(*page) < count)
        {
            Some(page) => RelationDestination::Page(page),
            None => RelationDestination::Unavailable,
        },
    )
}
