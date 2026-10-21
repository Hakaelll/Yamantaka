use rusqlite::{Connection, OpenFlags, params, params_from_iter, types::Value};
use std::{path::Path, time::Duration};
use yamantaka_domain::{
    Document, Failure, LibraryQuery, LibraryReadingState, LibraryRepository, LibrarySort,
    MAX_LIBRARY_PAGE, NewDocument, Result, SCHEMA_VERSION, SortDirection,
};

pub(crate) fn storage(error: impl std::fmt::Display) -> Failure {
    Failure::Storage(error.to_string())
}
pub const MIGRATIONS: [&str; 11] = [
    include_str!("../migrations/01.sql"),
    include_str!("../migrations/02.sql"),
    include_str!("../migrations/03.sql"),
    include_str!("../migrations/04.sql"),
    include_str!("../migrations/05.sql"),
    include_str!("../migrations/06.sql"),
    include_str!("../migrations/07.sql"),
    include_str!("../migrations/08.sql"),
    include_str!("../migrations/09.sql"),
    include_str!("../migrations/10.sql"),
    include_str!("../migrations/11.sql"),
];

pub struct Database {
    pub(crate) connection: Connection,
    pub(crate) import_journal: std::cell::RefCell<Option<Connection>>,
    pub(crate) file_checks: std::cell::RefCell<Option<Connection>>,
}
impl Database {
    pub fn file_path(&self) -> Option<std::path::PathBuf> {
        self.connection
            .path()
            .filter(|path| !path.is_empty())
            .map(std::path::PathBuf::from)
    }

    pub fn open_read_only(path: &Path) -> Result<Self> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(storage)?;
        connection
            .busy_timeout(Duration::from_secs(3))
            .map_err(storage)?;
        if version(&connection)? != SCHEMA_VERSION {
            return Err(Failure::Invalid(
                "The library must be migrated before opening a reader".into(),
            ));
        }
        Ok(Self {
            connection,
            import_journal: Default::default(),
            file_checks: Default::default(),
        })
    }
    pub fn open(path: &Path) -> Result<Self> {
        let mut connection = Connection::open(path).map_err(storage)?;
        configure(&connection)?;
        let version = version(&connection)?;
        if version > 0 && version < SCHEMA_VERSION {
            let backup = path.with_extension(format!(
                "pre-migration-v{version}-{}.sqlite3",
                uuid::Uuid::new_v4()
            ));
            snapshot(&connection, &backup)?;
        }
        migrate(&mut connection, SCHEMA_VERSION)?;
        validate(&connection)?;
        Ok(Self {
            connection,
            import_journal: Default::default(),
            file_checks: Default::default(),
        })
    }
    pub fn in_memory_at(target: u32) -> Result<Self> {
        let mut connection = Connection::open_in_memory().map_err(storage)?;
        configure(&connection)?;
        migrate(&mut connection, target)?;
        Ok(Self {
            connection,
            import_journal: Default::default(),
            file_checks: Default::default(),
        })
    }
    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        self.connection.execute_batch(sql).map_err(storage)
    }
    pub fn schema_version(&self) -> Result<u32> {
        version(&self.connection)
    }
    pub fn migrate_current(&mut self) -> Result<()> {
        migrate(&mut self.connection, SCHEMA_VERSION)
    }
    pub fn snapshot(&self, path: &Path) -> Result<()> {
        snapshot(&self.connection, path)
    }
    pub fn integrity(&self) -> Result<()> {
        validate(&self.connection)
    }
}
pub(crate) fn configure(connection: &Connection) -> Result<()> {
    connection
        .busy_timeout(Duration::from_secs(3))
        .map_err(storage)?;
    connection
        .execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
        .map_err(storage)
}
pub(crate) fn version(connection: &Connection) -> Result<u32> {
    connection
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(storage)
}
pub(crate) fn migrate(connection: &mut Connection, target: u32) -> Result<()> {
    let current = version(connection)?;
    if target == 0 || target > SCHEMA_VERSION || current > target {
        return Err(Failure::Invalid("Unsupported database schema".into()));
    }
    // All remaining steps commit together. An interrupted migration leaves its old schema intact.
    let tx = connection.transaction().map_err(storage)?;
    for sql in MIGRATIONS
        .iter()
        .take(target as usize)
        .skip(current as usize)
    {
        // Split only at generated blank statement boundaries, never within quoted JSON/SQL.
        for statement in sql
            .lines()
            .filter(|line| !line.starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n")
            .split(";\n\n")
        {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }
            if statement.starts_with("ALTER TABLE ") {
                let parts: Vec<_> = statement.split_whitespace().collect();
                let table = parts[2];
                let column = parts[5];
                let exists = tx
                    .prepare(&format!("PRAGMA table_info({table})"))
                    .map_err(storage)?
                    .query_map([], |row| row.get::<_, String>(1))
                    .map_err(storage)?
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(storage)?
                    .iter()
                    .any(|c| c == column);
                if exists {
                    continue;
                }
            }
            tx.execute_batch(statement).map_err(storage)?;
        }
    }
    if target >= 2 {
        tx.execute_batch("CREATE INDEX IF NOT EXISTS idx_rust_collection_documents_document ON collection_documents(document_id,collection_id); CREATE INDEX IF NOT EXISTS idx_rust_document_tags_tag ON document_tags(tag_id,document_id); CREATE INDEX IF NOT EXISTS idx_rust_annotation_tags_tag ON annotation_tags(tag_id,annotation_id); CREATE INDEX IF NOT EXISTS idx_rust_tags_name ON tags(name COLLATE NOCASE,id);").map_err(storage)?;
    }
    validate(&tx)?;
    tx.commit().map_err(storage)
}
pub(crate) fn validate(connection: &Connection) -> Result<()> {
    let check: String = connection
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(storage)?;
    if check != "ok" {
        return Err(Failure::Invalid("SQLite integrity check failed".into()));
    }
    if connection
        .prepare("PRAGMA foreign_key_check")
        .map_err(storage)?
        .query([])
        .map_err(storage)?
        .next()
        .map_err(storage)?
        .is_some()
    {
        return Err(Failure::Invalid("Broken database relationships".into()));
    }
    Ok(())
}
pub(crate) fn snapshot(source: &Connection, destination: &Path) -> Result<()> {
    snapshot_cancellable(
        source,
        destination,
        &yamantaka_domain::Cancellation::default(),
        |_, _| {},
    )
}
pub(crate) fn snapshot_cancellable(
    source: &Connection,
    destination: &Path,
    cancel: &yamantaka_domain::Cancellation,
    mut progress: impl FnMut(u32, u32),
) -> Result<()> {
    use rusqlite::backup::{Backup, StepResult};
    cancel.check()?;
    if destination.exists() {
        return Err(Failure::DestinationOccupied);
    }
    let _owned_destination = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                Failure::DestinationOccupied
            } else {
                storage(error)
            }
        })?;
    let mut copy = Connection::open_with_flags(destination, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(storage)?;
    let backup = Backup::new(source, &mut copy).map_err(storage)?;
    let mut blocked_since = None;
    loop {
        cancel.check()?;
        let step = backup.step(128).map_err(storage)?;
        let state = backup.progress();
        progress(
            state.pagecount.saturating_sub(state.remaining).max(0) as u32,
            state.pagecount.max(0) as u32,
        );
        cancel.check()?;
        match step {
            StepResult::Done => break,
            StepResult::More => {
                blocked_since = None;
            }
            StepResult::Busy | StepResult::Locked => {
                let since = blocked_since.get_or_insert_with(std::time::Instant::now);
                if since.elapsed() >= Duration::from_secs(3) {
                    return Err(Failure::Storage(
                        "Database remained locked during its snapshot".into(),
                    ));
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            _ => return Err(Failure::Storage("Unexpected database backup state".into())),
        }
    }
    drop(backup);
    cancel.check()?;
    validate(&copy)
}
pub(crate) fn readonly(path: &Path) -> Result<Connection> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(storage)
}

const PROJECTION: &str = "id,path,file_name,COALESCE(title_override,title,source_title),COALESCE(author_override,author,source_author),page_count,last_page_index,reading_progress,is_favorite,is_missing,fingerprint,thumbnail_path";
fn document(row: &rusqlite::Row<'_>) -> rusqlite::Result<Document> {
    let author: Option<String> = row.get(4)?;
    Ok(Document {
        id: row.get(0)?,
        path: row.get::<_, String>(1)?.into(),
        file_name: row.get(2)?,
        title: row.get(3)?,
        author: author.filter(|value| !value.trim().is_empty()),
        page_count: row.get(5)?,
        last_page_index: row.get(6)?,
        reading_progress: row.get(7)?,
        favorite: row.get(8)?,
        missing: row.get(9)?,
        fingerprint: row.get(10)?,
        thumbnail_path: row.get::<_, Option<String>>(11)?.map(Into::into),
    })
}
fn pattern(text: &str) -> String {
    format!(
        "%{}%",
        text.replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}
// All data values are bound; SQL fragments below are controlled by domain enums.
// EXISTS keeps a document unique even when several selected memberships match.
fn filter(query: &LibraryQuery) -> Result<(String, Vec<Value>)> {
    let mut sql = String::from(" WHERE (?=0 OR is_favorite=1) AND (?=0 OR is_missing=1)");
    let mut values = vec![
        Value::Integer(i64::from(query.favorites_only)),
        Value::Integer(i64::from(query.missing_only)),
    ];
    if query.modified_only {
        sql.push_str(" AND is_modified=1");
    }
    if !query.include_missing {
        sql.push_str(" AND is_missing=0");
    }
    match query.reading_state {
        LibraryReadingState::All => {}
        LibraryReadingState::InProgress => {
            sql.push_str(" AND reading_progress>0 AND reading_progress<1");
        }
        LibraryReadingState::NotStarted => sql.push_str(" AND reading_progress<=0"),
        LibraryReadingState::Completed => sql.push_str(" AND reading_progress>=1"),
    }
    if let Some(view) = query.smart_view {
        use yamantaka_domain::LibrarySmartView;
        match view {
            LibrarySmartView::InProgress => {
                sql.push_str(" AND reading_progress>0 AND reading_progress<0.995")
            }
            LibrarySmartView::NotStarted => sql.push_str(" AND reading_progress<=0"),
            LibrarySmartView::Finished => sql.push_str(" AND reading_progress>=0.995"),
            LibrarySmartView::Annotated => sql.push_str(
                " AND EXISTS(SELECT 1 FROM annotations a WHERE a.document_id=documents.id)",
            ),
            LibrarySmartView::Bookmarked => sql.push_str(
                " AND EXISTS(SELECT 1 FROM bookmarks b WHERE b.document_id=documents.id)",
            ),
            LibrarySmartView::Missing => sql.push_str(" AND is_missing=1"),
            LibrarySmartView::RecentlyOpened | LibrarySmartView::RecentlyAdded => {
                let invalid = || {
                    Failure::Invalid(
                        "Recent-view reference time is outside the supported range".into(),
                    )
                };
                let elapsed = query
                    .reference_time
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| invalid())?;
                let seconds = i64::try_from(elapsed.as_secs()).map_err(|_| invalid())?;
                let cutoff = chrono::DateTime::from_timestamp(seconds, elapsed.subsec_nanos())
                    .and_then(|now| now.checked_sub_signed(chrono::Duration::days(14)))
                    .ok_or_else(invalid)?;
                let column = if view == LibrarySmartView::RecentlyOpened {
                    "last_opened_at"
                } else {
                    "added_at"
                };
                sql.push_str(&format!(" AND julianday({column})>=julianday(?)"));
                values.push(Value::Text(cutoff.to_rfc3339()));
            }
        }
    }
    let authors: Vec<_> = query
        .authors
        .iter()
        .map(|author| author.trim().to_lowercase())
        .filter(|author| !author.is_empty())
        .collect();
    if !authors.is_empty() {
        sql.push_str(&format!(
            " AND lower(trim(COALESCE(author_override,author,source_author,''))) IN ({})",
            placeholders(authors.len()),
        ));
        values.extend(authors.into_iter().map(Value::Text));
    }
    if !query.topics.is_empty() {
        sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM document_topics filter_topics WHERE filter_topics.document_id=documents.id AND filter_topics.topic_key IN ({}))",
            placeholders(query.topics.len()),
        ));
        values.extend(query.topics.iter().cloned().map(Value::Text));
    }
    if !query.tag_ids.is_empty() {
        sql.push_str(&format!(
            " AND EXISTS (SELECT 1 FROM document_tags filter_tags WHERE filter_tags.document_id=documents.id AND filter_tags.tag_id IN ({}))",
            placeholders(query.tag_ids.len()),
        ));
        values.extend(query.tag_ids.iter().cloned().map(Value::Text));
    }
    if let Some(id) = query
        .collection_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        sql.push_str(" AND EXISTS (SELECT 1 FROM collection_documents filter_collection WHERE filter_collection.document_id=documents.id AND filter_collection.collection_id=?)");
        values.push(Value::Text(id.to_owned()));
    }
    let text = query.text.trim();
    if !text.is_empty() {
        sql.push_str(" AND (COALESCE(title_override,title,source_title,file_name) LIKE ? ESCAPE '\\' OR COALESCE(author_override,author,source_author,'') LIKE ? ESCAPE '\\' OR file_name LIKE ? ESCAPE '\\' OR EXISTS (SELECT 1 FROM document_notes text_notes WHERE text_notes.document_id=documents.id AND text_notes.note LIKE ? ESCAPE '\\') OR EXISTS (SELECT 1 FROM document_topics text_topics WHERE text_topics.document_id=documents.id AND text_topics.topic_key LIKE ? ESCAPE '\\'))");
        let escaped = pattern(text);
        values.extend((0..5).map(|_| Value::Text(escaped.clone())));
    }
    Ok((sql, values))
}

fn placeholders(count: usize) -> String {
    vec!["?"; count].join(",")
}

fn order(query: &LibraryQuery) -> String {
    let column = match query.sort {
        LibrarySort::LastOpened => "last_opened_at",
        LibrarySort::Title => "lower(COALESCE(title_override,title,source_title,file_name))",
        LibrarySort::Author => "lower(trim(COALESCE(author_override,author,source_author,'')))",
        LibrarySort::Added => "added_at",
        LibrarySort::Progress => "reading_progress",
        LibrarySort::PageCount => "page_count",
    };
    let direction = match query.sort_direction {
        SortDirection::Ascending => "ASC",
        SortDirection::Descending => "DESC",
        SortDirection::Default => match query.sort {
            LibrarySort::Title | LibrarySort::Author => "ASC",
            _ => "DESC",
        },
    };
    format!("{column} {direction},id ASC")
}
impl LibraryRepository for Database {
    fn remove_from_library(
        &mut self,
        id: &str,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<()> {
        crate::library_removal::remove(self, id, cancel)
    }
    fn tags(
        &self,
        query: &yamantaka_domain::tags::TagQuery,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<yamantaka_domain::tags::TagPage> {
        self.tag_page(query, cancel)
    }
    fn change_tag(
        &mut self,
        change: &yamantaka_domain::tags::TagChange,
        access: yamantaka_domain::search::IndexAccess,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<String> {
        self.apply_tag(change, access, cancel)
    }
    fn collections(
        &self,
        query: &yamantaka_domain::collections::CollectionQuery,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<yamantaka_domain::collections::CollectionPage> {
        self.collection_page(query, cancel)
    }
    fn change_collection(
        &mut self,
        change: &yamantaka_domain::collections::CollectionChange,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<String> {
        self.apply_collection(change, cancel)
    }
    fn registered_import_path(&self, path: &Path) -> Result<Option<String>> {
        crate::identity::registered_document(&self.connection, path)
    }

    fn file_check_upper_bound(&self) -> Result<Option<String>> {
        self.connection
            .query_row("SELECT max(id) FROM documents", [], |row| row.get(0))
            .map_err(storage)
    }
    fn file_check_page(&self, after: Option<&str>, upper: &str) -> Result<Vec<Document>> {
        let (predicate, values) = match after {
            Some(after) => ("id>? AND id<=?", vec![after, upper]),
            None => ("id<=?", vec![upper]),
        };
        let sql = format!(
            "SELECT {PROJECTION} FROM documents WHERE {predicate} ORDER BY id LIMIT {MAX_LIBRARY_PAGE}"
        );
        self.connection
            .prepare(&sql)
            .map_err(storage)?
            .query_map(params_from_iter(values), document)
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)
    }

    fn relocate_file(
        &mut self,
        plan: &yamantaka_domain::relocation::RelocationPlan,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<()> {
        crate::file_check::relocate(self, plan, cancel)
    }

    fn file_snapshot(&self, id: &str) -> Result<yamantaka_domain::file_check::FileSnapshot> {
        crate::file_check::read(self, id)
    }
    fn apply_file_check(
        &mut self,
        expected: &yamantaka_domain::file_check::FileSnapshot,
        result: yamantaka_domain::file_check::FileCheckResult,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<yamantaka_domain::file_check::FileCheckResult> {
        crate::file_check::apply(self, expected, result, cancel)
    }

    fn pending_imports(&self) -> Result<Vec<yamantaka_domain::import_recovery::PendingImport>> {
        crate::import_recovery::pending(self)
    }
    fn queue_import(
        &mut self,
        source: &yamantaka_domain::import_recovery::ImportSource,
    ) -> Result<Option<String>> {
        crate::import_recovery::queue(self, source).map(Some)
    }
    fn resume_import(&self, id: &str) -> Result<yamantaka_domain::import_recovery::ImportSource> {
        crate::import_recovery::resume(self, id)
    }
    fn import_already_committed(&self, job: &str, path: &Path) -> Result<bool> {
        crate::import_recovery::committed(self, job, path)
    }
    fn checkpoint_import(&mut self, job: &str, path: &Path, document: &str) -> Result<()> {
        crate::import_recovery::checkpoint(self, job, path, document)
    }
    fn remove_import(&mut self, id: &str) -> Result<()> {
        crate::import_recovery::remove(self, id)
    }

    fn reanalyze_metadata(
        &mut self,
        expected: &yamantaka_domain::metadata_review::MetadataReview,
        original: &Document,
        analyzed: &NewDocument,
        access: yamantaka_domain::search::IndexAccess,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<yamantaka_domain::metadata_review::MetadataReview> {
        crate::metadata_reanalysis::save(self, expected, original, analyzed, access, cancel)
    }
    fn metadata_source(
        &self,
        expected: &yamantaka_domain::metadata_review::MetadataReview,
        field: yamantaka_domain::metadata_review::MetadataField,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<yamantaka_domain::metadata_review::MetadataLocation> {
        crate::metadata_source::open(self, expected, field, cancel)
    }
    fn metadata_review(
        &self,
        document: &str,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<yamantaka_domain::metadata_review::MetadataReview> {
        crate::metadata_review::read(&self.connection, document, cancel)
    }
    fn change_metadata(
        &mut self,
        expected: &yamantaka_domain::metadata_review::MetadataReview,
        change: &yamantaka_domain::metadata_review::MetadataMutation,
        access: yamantaka_domain::search::IndexAccess,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<yamantaka_domain::metadata_review::MetadataReview> {
        crate::metadata_review::change(self, expected, change, access, cancel)
    }
    fn cover_snapshot(&self, id: &str) -> Result<yamantaka_domain::cover_snapshot::CoverSnapshot> {
        crate::cover_snapshot::read(&self.connection, id)
    }
    fn cover_revision(
        &self,
        id: &str,
    ) -> Result<Option<yamantaka_domain::cover_revision::CoverRevision>> {
        crate::cover_revision::read(&self.connection, id)
    }
    fn replace_automatic_cover(
        &mut self,
        choice: &yamantaka_domain::automatic_cover::AutomaticCover,
        expected: Option<yamantaka_domain::cover_revision::CoverRevision>,
    ) -> Result<bool> {
        crate::automatic_cover::replace(self, choice, expected)
    }
    fn save_automatic_cover(
        &mut self,
        choice: &yamantaka_domain::automatic_cover::AutomaticCover,
    ) -> Result<bool> {
        crate::automatic_cover::save(self, choice)
    }
    fn save_manual_cover(
        &mut self,
        choice: &yamantaka_domain::cover_selection::ManualCover,
    ) -> Result<()> {
        crate::cover_selection::save(self, choice)
    }
    fn cover_source_page(&self, document_id: &str) -> Result<Option<u32>> {
        crate::cover_selection::source_page(self, document_id)
    }
    fn reader_continuity(&self) -> Result<Option<yamantaka_domain::reader_tabs::DocumentTabs>> {
        let Some(path) = self.connection.path().filter(|path| !path.is_empty()) else {
            return Ok(None);
        };
        crate::reader_continuity::load(&Path::new(path).with_extension("reader-continuity.json"))
    }
    fn save_reader_continuity(
        &mut self,
        tabs: &yamantaka_domain::reader_tabs::DocumentTabs,
    ) -> Result<()> {
        let path = self
            .connection
            .path()
            .filter(|path| !path.is_empty())
            .ok_or_else(|| {
                Failure::Unsupported("Reader continuity requires a file-backed library".into())
            })?;
        crate::reader_continuity::save(
            &Path::new(path).with_extension("reader-continuity.json"),
            tabs,
        )
    }
    fn get_reader_preferences(
        &self,
        document_id: &str,
    ) -> Result<Option<yamantaka_domain::reader_preferences::ReaderPreferences>> {
        crate::reader_preferences::load(self, document_id)
    }
    fn save_reader_preferences(
        &mut self,
        document_id: &str,
        preferences: &yamantaka_domain::reader_preferences::ReaderPreferences,
    ) -> Result<()> {
        crate::reader_preferences::save(self, document_id, preferences)
    }
    fn import(
        &mut self,
        document: &NewDocument,
        reference: &yamantaka_domain::source_revision::FullSourceHash,
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<String> {
        crate::import_document::save(self, document, reference, cancel)
    }
    fn library_memberships(
        &self,
        ids: &[String],
        cancel: &yamantaka_domain::Cancellation,
    ) -> Result<Vec<yamantaka_domain::LibraryMembership>> {
        self.group_memberships(ids, cancel)
    }
    fn query(&self, query: &LibraryQuery) -> Result<Vec<Document>> {
        let (filter, mut values) = filter(query)?;
        let sql = format!(
            "SELECT {PROJECTION} FROM documents{filter} ORDER BY {} LIMIT ? OFFSET ?",
            order(query),
        );
        let limit = if query.page_size == 0 {
            MAX_LIBRARY_PAGE
        } else {
            query.page_size.min(MAX_LIBRARY_PAGE)
        };
        values.push(Value::Integer(limit as i64));
        values.push(Value::Integer(query.offset.min(i64::MAX as usize) as i64));
        self.connection
            .prepare(&sql)
            .map_err(storage)?
            .query_map(params_from_iter(values.iter()), document)
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)
    }
    fn count(&self, query: &LibraryQuery) -> Result<u64> {
        let (filter, values) = filter(query)?;
        self.connection
            .query_row(
                &format!("SELECT COUNT(*) FROM documents{filter}"),
                params_from_iter(values.iter()),
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n as u64)
            .map_err(storage)
    }
    fn set_favorite(&mut self, id: &str, favorite: bool) -> Result<()> {
        let changed = self
            .connection
            .execute(
                "UPDATE documents SET is_favorite=? WHERE id=?",
                params![favorite, id],
            )
            .map_err(storage)?;
        if changed == 0 {
            return Err(Failure::Invalid("Document no longer exists".into()));
        }
        Ok(())
    }
    fn save_position(&mut self, id: &str, page: u32) -> Result<()> {
        self.connection.execute("UPDATE documents SET last_page_index=MIN(?1,MAX(page_count-1,0)),reading_progress=CASE WHEN page_count>0 THEN (MIN(?1,page_count-1)+1.0)/page_count ELSE 0 END,last_opened_at=?2 WHERE id=?3", params![page, chrono::Utc::now().to_rfc3339(), id]).map_err(storage)?;
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Option<Document>> {
        use rusqlite::OptionalExtension;
        self.connection
            .query_row(
                &format!("SELECT {PROJECTION} FROM documents WHERE id=?"),
                [id],
                document,
            )
            .optional()
            .map_err(storage)
    }
}
