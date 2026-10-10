use crate::{Database, database::storage};
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter, types::Value};
use serde::{Deserialize, Serialize};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    annotations::repository::*,
    annotations::*,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    reader_preferences::PageRotation,
    search::IndexAccess,
};
mod batch;
mod marks;
mod writes;
const SELECT_RECORDS: &str = "SELECT a.id,a.document_id,a.page_index,a.type,a.color,a.note,a.selected_text,a.context_before,a.context_after,a.rects_json,a.page_rotation,a.document_fingerprint,a.status,a.created_at,a.updated_at,a.review_state,a.origin_document_id,a.origin_fingerprint,a.* FROM annotations a";
const KNOWN_COLUMNS: [&str; 18] = [
    "id",
    "document_id",
    "page_index",
    "type",
    "color",
    "note",
    "selected_text",
    "context_before",
    "context_after",
    "rects_json",
    "page_rotation",
    "document_fingerprint",
    "status",
    "created_at",
    "updated_at",
    "review_state",
    "origin_document_id",
    "origin_fingerprint",
];
fn known_column(name: &str) -> bool {
    KNOWN_COLUMNS
        .iter()
        .any(|known| known.eq_ignore_ascii_case(name))
}

fn name(value: impl Serialize) -> Result<String> {
    match serde_json::to_value(value).map_err(storage)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(Failure::Invalid(
            "Annotation enum encoding is invalid".into(),
        )),
    }
}
fn decode<'a, T: Deserialize<'a>>(value: &'a str) -> Result<T> {
    T::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
        .map_err(|_| Failure::Invalid("Unsupported annotation enum value".into()))
}
fn field<'a>(row: &'a Row<'_>, index: usize) -> Result<&'a str> {
    row.get_ref(index)
        .map_err(storage)?
        .as_str()
        .map_err(storage)
}
fn read_title(
    db: &Connection,
    document: &str,
    lease: &mut yamantaka_domain::memory::ByteLease,
    bytes: usize,
) -> Result<Option<String>> {
    let mut title_statement = db.prepare("SELECT COALESCE(title_override,title,source_title,file_name) FROM documents WHERE id=?").map_err(storage)?;
    let mut title_rows = title_statement.query([document]).map_err(storage)?;
    if let Some(title_row) = title_rows.next().map_err(storage)? {
        let title = field(title_row, 0)?;
        lease
            .try_resize(
                bytes
                    .checked_add(title.len())
                    .ok_or(Failure::MemoryBudget)?,
            )
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(Some(title.into()))
    } else {
        Ok(None)
    }
}
fn read_record(
    db: &Connection,
    row: &Row<'_>,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
    include_title: bool,
) -> Result<AnnotationRecord> {
    cancel.check()?;
    let mut bytes = row
        .as_ref()
        .column_count()
        .checked_mul(std::mem::size_of::<(String, AnnotationColumnValue)>())
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or(Failure::MemoryBudget)?;
    for index in 0..row.as_ref().column_count() {
        let value = row.get_ref(index).map_err(storage)?;
        if let rusqlite::types::ValueRef::Text(value) | rusqlite::types::ValueRef::Blob(value) =
            value
        {
            bytes = bytes
                .checked_add(value.len().checked_mul(3).ok_or(Failure::MemoryBudget)?)
                .ok_or(Failure::MemoryBudget)?;
        }
    }
    let mut lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let id = field(row, 0)?;
    let (tag_count,tag_bytes):(i64,i64)=db.query_row("SELECT count(*),COALESCE(sum(length(CAST(t.name AS BLOB))),0) FROM annotation_tags a JOIN tags t ON t.id=a.tag_id WHERE a.annotation_id=?",[id],|row| Ok((row.get(0)?,row.get(1)?))).map_err(storage)?;
    let tag_count = usize::try_from(tag_count).map_err(|_| Failure::MemoryBudget)?;
    let tag_bytes = usize::try_from(tag_bytes).map_err(|_| Failure::MemoryBudget)?;
    bytes = bytes
        .checked_add(
            tag_count
                .checked_mul(std::mem::size_of::<String>())
                .ok_or(Failure::MemoryBudget)?,
        )
        .and_then(|bytes| bytes.checked_add(tag_bytes))
        .ok_or(Failure::MemoryBudget)?;
    lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
    let mut tags = Vec::new();
    tags.try_reserve_exact(tag_count)
        .map_err(|_| Failure::MemoryBudget)?;
    let mut statement=db.prepare("SELECT t.name FROM tags t JOIN annotation_tags a ON a.tag_id=t.id WHERE a.annotation_id=? ORDER BY t.name COLLATE NOCASE,t.id").map_err(storage)?;
    for tag in statement
        .query_map([id], |row| row.get::<_, String>(0))
        .map_err(storage)?
    {
        cancel.check()?;
        tags.push(tag.map_err(storage)?);
    }
    let values = AnnotationDraftValues {
        document_id: row.get(1).map_err(storage)?,
        page_index: row.get(2).map_err(storage)?,
        kind: decode(field(row, 3)?)?,
        color: decode(field(row, 4)?)?,
        note: row.get(5).map_err(storage)?,
        selected_text: row.get(6).map_err(storage)?,
        context_before: row.get(7).map_err(storage)?,
        context_after: row.get(8).map_err(storage)?,
        rects: serde_json::from_str(field(row, 9)?)
            .map_err(|_| Failure::Invalid("Annotation geometry cannot be decoded safely".into()))?,
        page_rotation: PageRotation::from_degrees(row.get(10).map_err(storage)?)?.degrees(),
        document_fingerprint: row.get(11).map_err(storage)?,
        status: decode(field(row, 12)?)?,
        review_state: decode(field(row, 15)?)?,
    };
    let mut extensions = Vec::new();
    let columns = row.as_ref().column_names();
    let extra_count = columns
        .iter()
        .skip(18)
        .filter(|name| !known_column(name))
        .count();
    let extra_bytes = extra_count
        .checked_mul(std::mem::size_of::<(String, AnnotationColumnValue)>())
        .and_then(|value| {
            value.checked_add(
                columns
                    .iter()
                    .skip(18)
                    .filter(|name| !known_column(name))
                    .map(|name| name.len())
                    .sum::<usize>(),
            )
        })
        .ok_or(Failure::MemoryBudget)?;
    lease
        .try_resize(
            bytes
                .checked_add(extra_bytes)
                .ok_or(Failure::MemoryBudget)?,
        )
        .map_err(|_| Failure::MemoryBudget)?;
    extensions
        .try_reserve_exact(extra_count)
        .map_err(|_| Failure::MemoryBudget)?;
    for (index, name) in columns
        .iter()
        .enumerate()
        .skip(18)
        .filter(|(_, name)| !known_column(name))
    {
        let value = match row.get_ref(index).map_err(storage)? {
            rusqlite::types::ValueRef::Null => AnnotationColumnValue::Null,
            rusqlite::types::ValueRef::Integer(value) => AnnotationColumnValue::Integer(value),
            rusqlite::types::ValueRef::Real(value) => AnnotationColumnValue::Real(value),
            rusqlite::types::ValueRef::Text(value) => {
                AnnotationColumnValue::Text(std::str::from_utf8(value).map_err(storage)?.into())
            }
            rusqlite::types::ValueRef::Blob(value) => AnnotationColumnValue::Blob(value.into()),
        };
        extensions.push(((*name).into(), value));
    }
    drop(columns);
    let document_title = if include_title {
        read_title(db, &values.document_id, &mut lease, bytes)?
    } else {
        None
    };
    AnnotationRecord::from_reserved(
        AnnotationRecordData {
            document_title,
            id: id.into(),
            values,
            tags,
            created_at: row.get(13).map_err(storage)?,
            updated_at: row.get(14).map_err(storage)?,
            origin_document_id: row.get(16).map_err(storage)?,
            origin_fingerprint: row.get(17).map_err(storage)?,
            extensions,
        },
        lease,
    )
}
fn read_page(
    db: &Connection,
    query: &AnnotationQuery,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<AnnotationPage> {
    cancel.check()?;
    let mut where_sql = String::from(" WHERE 1=1");
    let mut values = Vec::new();
    for (column, value) in [
        ("document_id", query.document_id.clone()),
        ("type", query.kind.map(name).transpose()?),
        ("color", query.color.map(name).transpose()?),
        ("review_state", query.review.map(name).transpose()?),
    ] {
        if let Some(value) = value {
            where_sql.push_str(&format!(" AND a.{column}=?"));
            values.push(Value::Text(value));
        }
    }
    for (operator, value) in [
        (">=", query.updated_after.as_ref()),
        ("<", query.updated_before.as_ref()),
    ] {
        if let Some(value) = value {
            where_sql.push_str(&format!(" AND a.updated_at{operator}?"));
            values.push(Value::Text(value.clone()));
        }
    }
    if let Some(page) = query.page {
        where_sql.push_str(" AND a.page_index=?");
        values.push(Value::Integer(page.into()));
    }
    if let Some(tag) = query
        .tag
        .as_ref()
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
    {
        where_sql.push_str(" AND EXISTS(SELECT 1 FROM annotation_tags at JOIN tags t ON t.id=at.tag_id WHERE at.annotation_id=a.id AND t.normalized_name=?)");
        values.push(Value::Text(tag.to_lowercase()));
    }
    if !query.text.trim().is_empty() {
        where_sql.push_str(" AND (lower(COALESCE(a.note,'')) LIKE ? ESCAPE '\\' OR lower(COALESCE(a.selected_text,'')) LIKE ? ESCAPE '\\')");
        let text = query
            .text
            .trim()
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        values.extend([
            Value::Text(format!("%{text}%")),
            Value::Text(format!("%{text}%")),
        ]);
    }
    let total = db
        .query_row(
            &format!("SELECT count(*) FROM annotations a{where_sql}"),
            params_from_iter(&values),
            |row| row.get::<_, i64>(0),
        )
        .map_err(storage)?;
    let total =
        u64::try_from(total).map_err(|_| Failure::Invalid("Annotation count is invalid".into()))?;
    let offset = query.offset.min(i64::MAX as usize);
    let order = match query.grouping {
        Some(AnnotationGrouping::Document) => {
            "(SELECT COALESCE(title_override,title,source_title,file_name) FROM documents WHERE id=a.document_id) COLLATE NOCASE,a.document_id,a.updated_at,a.id"
        }
        Some(AnnotationGrouping::Page) => "a.page_index,a.updated_at,a.id",
        Some(AnnotationGrouping::Review) => {
            "CASE a.review_state WHEN 'pending' THEN 0 WHEN 'accepted' THEN 1 ELSE 2 END,a.updated_at,a.id"
        }
        None if query.newest_first || query.document_id.is_none() => {
            "a.updated_at DESC,a.page_index,a.id"
        }
        None => "a.page_index,a.created_at,a.id",
    };
    values.extend([
        Value::Integer(ANNOTATION_PAGE_SIZE as i64),
        Value::Integer(offset as i64),
    ]);
    let mut statement = db
        .prepare(&format!(
            "{SELECT_RECORDS}{where_sql} ORDER BY {order} LIMIT ? OFFSET ?"
        ))
        .map_err(storage)?;
    let mut rows = statement
        .query(params_from_iter(&values))
        .map_err(storage)?;
    let page_bytes = ANNOTATION_PAGE_SIZE
        .checked_mul(std::mem::size_of::<AnnotationRecord>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<AnnotationPage>()))
        .ok_or(Failure::MemoryBudget)?;
    let page_lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            page_bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(ANNOTATION_PAGE_SIZE)
        .map_err(|_| Failure::MemoryBudget)?;
    while let Some(row) = rows.next().map_err(storage)? {
        entries.push(read_record(db, row, governor, cancel, true)?);
    }
    cancel.check()?;
    AnnotationPage::from_reserved(entries, total, offset, page_lease)
}
impl AnnotationRepository for Database {
    fn export_reader(&self) -> Result<Box<dyn AnnotationRepository + Send>> {
        let path = self
            .connection
            .path()
            .filter(|path| !path.is_empty())
            .ok_or_else(|| {
                Failure::Unsupported("Independent export requires a file-backed library".into())
            })?;
        let connection = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| Failure::Storage("Annotation export reader could not open".into()))?;
        connection
            .busy_timeout(std::time::Duration::from_secs(3))
            .map_err(|_| Failure::Storage("Annotation export reader could not open".into()))?;
        Ok(Box::new(Database { connection }))
    }

    fn publish_annotation_export(
        &self,
        destination: &std::path::Path,
        cancel: &Cancellation,
        write: &mut dyn FnMut(&mut dyn std::io::Write) -> Result<u64>,
    ) -> Result<u64> {
        self.create_annotation_export(destination, cancel, |file| write(file))
    }

    fn visit_annotations(
        &self,
        document: &str,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
        visitor: &mut dyn FnMut(&AnnotationRecord) -> Result<()>,
    ) -> Result<u64> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let mut count = 0u64;
        {
            let mut statement = tx
                .prepare(&format!(
                    "{SELECT_RECORDS} WHERE a.document_id=? ORDER BY a.page_index,a.created_at,a.id"
                ))
                .map_err(storage)?;
            let mut rows = statement.query([document]).map_err(storage)?;
            while let Some(row) = rows.next().map_err(storage)? {
                let record = read_record(&tx, row, governor, cancel, false)?;
                visitor(&record)?;
                cancel.check()?;
                count = count.checked_add(1).ok_or(Failure::MemoryBudget)?;
            }
        }
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(count)
    }

    fn annotation_marks(
        &self,
        document: &str,
        page: u32,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<yamantaka_domain::annotations::marks::AnnotationMarks> {
        marks::read(self, document, page, governor, cancel)
    }

    fn annotations(
        &self,
        query: &AnnotationQuery,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<AnnotationPage> {
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let result = read_page(&tx, query, governor, cancel)?;
        tx.commit().map_err(storage)?;
        Ok(result)
    }
    fn change_annotation_batch(
        &mut self,
        change: AnnotationBatchChange<'_>,
        query: &AnnotationQuery,
        access: IndexAccess,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<AnnotationPage> {
        batch::change(self, change, query, access, governor, cancel)
    }
    fn change_annotation(
        &mut self,
        document_id: &str,
        change: AnnotationChange<'_>,
        query: &AnnotationQuery,
        access: IndexAccess,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<AnnotationPage> {
        writes::change(self, document_id, change, query, access, governor, cancel)
    }
}
