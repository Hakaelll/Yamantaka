use crate::{Database, database::storage};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use yamantaka_domain::{Cancellation, Failure, MAX_LIBRARY_PAGE, Result, collections::*};

fn invalid(message: &str) -> Failure {
    Failure::Invalid(message.into())
}
fn name(value: &str) -> Result<&str> {
    let clean = value.trim();
    if clean.is_empty() || clean.encode_utf16().count() > 60 {
        return Err(invalid("Use a collection name between 1 and 60 characters"));
    }
    Ok(clean)
}
fn exists(db: &Connection, document: &str) -> Result<()> {
    if !db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE id=?)",
            [document],
            |row| row.get::<_, bool>(0),
        )
        .map_err(storage)?
    {
        return Err(invalid("Document is no longer in the library"));
    }
    Ok(())
}
fn verify(db: &Connection, revision: &CollectionRevision) -> Result<()> {
    let current = db
        .query_row(
            "SELECT updated_at FROM collections WHERE id=?",
            [&revision.id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(storage)?;
    if current.as_deref() != Some(revision.updated_at.as_str()) {
        return Err(invalid("Collection changed; refresh before retrying"));
    }
    Ok(())
}
fn unique(db: &Connection, clean: &str, except: &str) -> Result<()> {
    if db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM collections WHERE normalized_name=? AND id<>?)",
            params![clean.to_lowercase(), except],
            |row| row.get::<_, bool>(0),
        )
        .map_err(storage)?
    {
        return Err(invalid("A collection with that name already exists"));
    }
    Ok(())
}
fn create(db: &Connection, value: &str, icon: &str) -> Result<String> {
    let clean = name(value)?;
    unique(db, clean, "")?;
    if !["folder", "study", "reference", "archive", "star"].contains(&icon) {
        return Err(invalid("Unknown collection icon"));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = revision_after(None);
    db.execute("INSERT INTO collections(id,name,normalized_name,icon,sort_order,is_pinned,created_at,updated_at) SELECT ?,?,?,?,COALESCE(MAX(sort_order),-1)+1,0,?,? FROM collections",params![id,clean,clean.to_lowercase(),icon,now,now]).map_err(storage)?;
    Ok(id)
}
fn revision_after(previous: Option<&str>) -> String {
    let mut now = chrono::Utc::now();
    if let Some(previous) =
        previous.and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        && now <= previous
    {
        now = previous.with_timezone(&chrono::Utc) + chrono::Duration::nanoseconds(1);
    }
    now.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}
impl Database {
    pub(crate) fn collection_page(
        &self,
        query: &CollectionQuery,
        cancel: &Cancellation,
    ) -> Result<CollectionPage> {
        cancel.check()?;
        if query.text.len() > 4096 {
            return Err(invalid("Collection search is too long"));
        }
        if let Some(document) = &query.document {
            exists(&self.connection, document)?;
        }
        let pattern = format!(
            "%{}%",
            query
                .text
                .trim()
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let total = self
            .connection
            .query_row(
                "SELECT count(*) FROM collections WHERE (?1 IS NULL OR id=?1) AND name LIKE ?2 ESCAPE '\\'",
                params![query.id,pattern],
                |row| row.get::<_, i64>(0),
            )
            .map_err(storage)?;
        let mut statement=self.connection.prepare("SELECT c.id,c.name,c.icon,c.sort_order,c.is_pinned,c.updated_at,(SELECT count(*) FROM collection_documents m WHERE m.collection_id=c.id),EXISTS(SELECT 1 FROM collection_documents m WHERE m.collection_id=c.id AND m.document_id=?) FROM collections c WHERE c.name LIKE ? ESCAPE '\\' AND (? IS NULL OR c.id=?) ORDER BY c.is_pinned DESC,c.sort_order,c.name COLLATE NOCASE,c.id LIMIT ? OFFSET ?").map_err(storage)?;
        let offset = i64::try_from(query.offset)
            .map_err(|_| invalid("Collection page offset is too large"))?;
        let rows = statement
            .query_map(
                params![
                    query.document,
                    pattern,
                    query.id,
                    query.id,
                    MAX_LIBRARY_PAGE as i64,
                    offset
                ],
                |row| {
                    Ok(Collection {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        icon: row.get(2)?,
                        sort_order: row.get(3)?,
                        pinned: row.get(4)?,
                        updated_at: row.get(5)?,
                        document_count: row.get::<_, i64>(6)? as u64,
                        contains_document: row.get(7)?,
                    })
                },
            )
            .map_err(storage)?;
        let mut entries = Vec::new();
        for row in rows {
            cancel.check()?;
            entries.push(row.map_err(storage)?);
        }
        cancel.check()?;
        Ok(CollectionPage {
            entries,
            total: total as u64,
        })
    }
    pub(crate) fn apply_collection(
        &mut self,
        change: &CollectionChange,
        cancel: &Cancellation,
    ) -> Result<String> {
        cancel.check()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let id = match change {
            CollectionChange::Create { name, icon } => create(&tx, name, icon)?,
            CollectionChange::EnsureForDocument {
                name: value,
                icon,
                document,
            } => {
                exists(&tx, document)?;
                let clean = name(value)?;
                let existing = tx
                    .query_row(
                        "SELECT id FROM collections WHERE normalized_name=?",
                        [clean.to_lowercase()],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .map_err(storage)?;
                let id = match existing {
                    Some(id) => id,
                    None => create(&tx, clean, icon)?,
                };
                tx.execute("INSERT OR IGNORE INTO collection_documents(collection_id,document_id,added_at) VALUES(?,?,?)",params![id,document,revision_after(None)]).map_err(storage)?;
                id
            }
            CollectionChange::Rename {
                expected,
                name: value,
            } => {
                verify(&tx, expected)?;
                let clean = name(value)?;
                unique(&tx, clean, &expected.id)?;
                tx.execute(
                    "UPDATE collections SET name=?,normalized_name=?,updated_at=? WHERE id=?",
                    params![
                        clean,
                        clean.to_lowercase(),
                        revision_after(Some(&expected.updated_at)),
                        expected.id
                    ],
                )
                .map_err(storage)?;
                expected.id.clone()
            }
            CollectionChange::Pin { expected, pinned } => {
                verify(&tx, expected)?;
                tx.execute(
                    "UPDATE collections SET is_pinned=?,updated_at=? WHERE id=?",
                    params![
                        pinned,
                        revision_after(Some(&expected.updated_at)),
                        expected.id
                    ],
                )
                .map_err(storage)?;
                expected.id.clone()
            }
            CollectionChange::Delete { expected } => {
                verify(&tx, expected)?;
                tx.execute("DELETE FROM collections WHERE id=?", [&expected.id])
                    .map_err(storage)?;
                expected.id.clone()
            }
            CollectionChange::SetMembership {
                expected,
                document,
                included,
            } => {
                verify(&tx, expected)?;
                exists(&tx, document)?;
                if *included {
                    tx.execute("INSERT OR IGNORE INTO collection_documents(collection_id,document_id,added_at) VALUES(?,?,?)",params![expected.id,document,revision_after(None)]).map_err(storage)?;
                } else {
                    tx.execute(
                        "DELETE FROM collection_documents WHERE collection_id=? AND document_id=?",
                        params![expected.id, document],
                    )
                    .map_err(storage)?;
                }
                expected.id.clone()
            }
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(id)
    }
}
