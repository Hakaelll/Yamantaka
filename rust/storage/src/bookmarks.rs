use crate::{Database, database::storage};
use chrono::{SecondsFormat, Utc};
use rusqlite::{OptionalExtension, params};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    bookmarks::{BOOKMARK_PAGE_SIZE, Bookmark, BookmarkChange, BookmarkPage, BookmarkRepository},
};

fn label(value: &Option<String>) -> Result<Option<String>> {
    let value = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if value.is_some_and(|value| value.chars().count() > 4096) {
        return Err(Failure::Invalid(
            "Bookmark label exceeds 4096 characters".into(),
        ));
    }
    Ok(value.map(str::to_owned))
}

fn read_page(
    connection: &rusqlite::Connection,
    document_id: &str,
    offset: usize,
    cancel: &Cancellation,
) -> Result<BookmarkPage> {
    cancel.check()?;
    let total: i64 = connection
        .query_row(
            "SELECT count(*) FROM bookmarks WHERE document_id=?",
            [document_id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let total =
        u64::try_from(total).map_err(|_| Failure::Invalid("Negative bookmark count".into()))?;
    let last = total.saturating_sub(1) / BOOKMARK_PAGE_SIZE as u64 * BOOKMARK_PAGE_SIZE as u64;
    let offset = (offset / BOOKMARK_PAGE_SIZE * BOOKMARK_PAGE_SIZE).min(
        usize::try_from(last)
            .map_err(|_| Failure::Invalid("Bookmark count exceeds platform limits".into()))?,
    );
    let sql_offset = i64::try_from(offset)
        .map_err(|_| Failure::Invalid("Bookmark offset exceeds SQLite limits".into()))?;
    let mut statement = connection.prepare("SELECT id,document_id,page_index,label,created_at,updated_at FROM bookmarks WHERE document_id=? ORDER BY page_index,id LIMIT ? OFFSET ?").map_err(storage)?;
    let rows = statement
        .query_map(
            params![document_id, BOOKMARK_PAGE_SIZE as i64, sql_offset],
            |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    page_index: row.get(2)?,
                    label: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(storage)?;
    let mut entries = Vec::with_capacity(BOOKMARK_PAGE_SIZE);
    for row in rows {
        cancel.check()?;
        entries.push(row.map_err(storage)?);
    }
    cancel.check()?;
    Ok(BookmarkPage {
        entries,
        total,
        offset,
    })
}

impl BookmarkRepository for Database {
    fn bookmarks(
        &self,
        document_id: &str,
        offset: usize,
        cancel: &Cancellation,
    ) -> Result<BookmarkPage> {
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let page = read_page(&tx, document_id, offset, cancel)?;
        tx.commit().map_err(storage)?;
        Ok(page)
    }

    fn change_bookmark(
        &mut self,
        document_id: &str,
        change: &BookmarkChange,
        offset: usize,
        cancel: &Cancellation,
    ) -> Result<BookmarkPage> {
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        let exists: Option<u8> = tx
            .query_row("SELECT 1 FROM documents WHERE id=?", [document_id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(storage)?;
        exists.ok_or_else(|| Failure::Invalid("Bookmark document no longer exists".into()))?;
        let now = Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);
        match change {
            BookmarkChange::Ensure { page } => {
                tx.execute("INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at) VALUES(?,?,?,NULL,?,?) ON CONFLICT(document_id,page_index) DO NOTHING", params![uuid::Uuid::new_v4().to_string(), document_id, page, now, now]).map_err(storage)?;
            }
            BookmarkChange::Save { page, label: value } => {
                // The canonical reader validates actual PDF page bounds.
                // Persisted catalogue page counts can be stale after file changes;
                // Flutter permits bookmarks beyond that metadata count as well.
                let value = label(value)?;
                tx.execute("INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at) VALUES(?,?,?,?,?,?) ON CONFLICT(document_id,page_index) DO UPDATE SET label=excluded.label,updated_at=excluded.updated_at", params![uuid::Uuid::new_v4().to_string(), document_id, page, value, now, now]).map_err(storage)?;
            }
            BookmarkChange::Rename { id, label: value } => {
                let value = label(value)?;
                if tx
                    .execute(
                        "UPDATE bookmarks SET label=?,updated_at=? WHERE id=? AND document_id=?",
                        params![value, now, id, document_id],
                    )
                    .map_err(storage)?
                    == 0
                {
                    return Err(Failure::Invalid(
                        "Bookmark no longer belongs to this document".into(),
                    ));
                }
            }
            BookmarkChange::Remove { id } => {
                if tx
                    .execute(
                        "DELETE FROM bookmarks WHERE id=? AND document_id=?",
                        params![id, document_id],
                    )
                    .map_err(storage)?
                    == 0
                {
                    return Err(Failure::Invalid(
                        "Bookmark no longer belongs to this document".into(),
                    ));
                }
            }
        }
        let page = read_page(&tx, document_id, offset, cancel)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_change(
        db: &mut Database,
        id: &str,
        change: &BookmarkChange,
        cancel: &Cancellation,
    ) -> Result<BookmarkPage> {
        db.change_bookmark(id, change, 0, cancel)
    }

    fn database() -> Database {
        let db = Database::in_memory_at(11).unwrap();
        db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',5000,1,'now','now','now','a'),('b','b.pdf','b.pdf','b.pdf',2,1,'now','now','now','b');").unwrap();
        db
    }

    #[test]
    fn ensuring_a_page_preserves_an_existing_named_bookmark_exactly() {
        let mut db = database();
        let cancel = Cancellation::default();
        let first = apply_change(
            &mut db,
            "a",
            &BookmarkChange::Save {
                page: 2,
                label: Some("Study this passage".into()),
            },
            &cancel,
        )
        .unwrap();
        let repeated =
            apply_change(&mut db, "a", &BookmarkChange::Ensure { page: 2 }, &cancel).unwrap();
        assert_eq!(first, repeated);
        let added =
            apply_change(&mut db, "a", &BookmarkChange::Ensure { page: 4 }, &cancel).unwrap();
        assert_eq!(added.total, 2);
        assert_eq!(added.entries[0], first.entries[0]);
        assert_eq!(added.entries[1].label, None);
    }

    #[test]
    fn canonical_page_can_be_saved_when_catalogue_page_count_is_stale() {
        let mut db = database();
        db.execute_batch("UPDATE documents SET page_count=1 WHERE id='a';")
            .unwrap();
        let page = db
            .change_bookmark(
                "a",
                &BookmarkChange::Save {
                    page: 9,
                    label: None,
                },
                0,
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(page.entries[0].page_index, 9);
    }

    #[test]
    fn snapshot_conversion_failure_rolls_back_the_bookmark_mutation() {
        let mut db = database();
        db.execute_batch(
            "INSERT INTO bookmarks VALUES('unrepresentable','a',4294967296,NULL,'now','now');",
        )
        .unwrap();
        assert!(
            db.change_bookmark(
                "a",
                &BookmarkChange::Save {
                    page: 8,
                    label: None
                },
                0,
                &Cancellation::default()
            )
            .is_err()
        );
        let count: i64 = db
            .connection
            .query_row(
                "SELECT count(*) FROM bookmarks WHERE document_id='a' AND page_index=8",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            count, 0,
            "failed result conversion must not commit an unseen change"
        );
        let count: i64 = db
            .connection
            .query_row(
                "SELECT count(*) FROM bookmarks WHERE id='unrepresentable'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "original data must remain available for recovery");
    }

    #[test]
    fn all_historical_schemas_support_bookmarks_without_changing_the_schema_version() {
        for version in 1..=11 {
            let mut db = Database::in_memory_at(version).unwrap();
            db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',2,1,'now','now','now','a');").unwrap();
            let page = db
                .change_bookmark(
                    "a",
                    &BookmarkChange::Save {
                        page: 1,
                        label: Some("Historical study position".into()),
                    },
                    0,
                    &Cancellation::default(),
                )
                .unwrap();
            assert_eq!(page.total, 1);
            assert_eq!(page.entries[0].page_index, 1);
            let stored: u32 = db
                .connection
                .query_row("PRAGMA user_version", [], |row| row.get(0))
                .unwrap();
            assert_eq!(stored, version);
        }
    }

    #[test]
    fn bookmarks_survive_reopening_and_version_nine_backup_restore_with_exact_values() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("synthetic-library.sqlite3");
        let cancel = Cancellation::default();
        let mut db = Database::open(&path).unwrap();
        db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',2,1,'now','now','now','a');").unwrap();
        let saved = db
            .change_bookmark(
                "a",
                &BookmarkChange::Save {
                    page: 1,
                    label: Some("Résumé · دراسة".into()),
                },
                0,
                &cancel,
            )
            .unwrap();
        drop(db);
        let db = Database::open(&path).unwrap();
        assert_eq!(db.bookmarks("a", 0, &cancel).unwrap(), saved);
        let backup = directory.path().join("synthetic-backup.json");
        db.export_backup(
            &backup,
            &crate::preferences::Preferences::default(),
            &cancel,
        )
        .unwrap();
        let restored = crate::backup::preview(&backup, &cancel).unwrap();
        assert_eq!(restored.backup_version, 9);
        assert_eq!(restored.database.bookmarks("a", 0, &cancel).unwrap(), saved);
    }

    #[test]
    fn editing_legacy_bookmark_preserves_id_creation_time_and_other_document() {
        let mut db = database();
        db.execute_batch("INSERT INTO bookmarks VALUES('legacy-id','a',7,'Original','2020-01-02T03:04:05Z','2020-01-02T03:04:05Z'),('other-id','b',1,'Other','2021-01-02T03:04:05Z','2021-01-02T03:04:05Z');").unwrap();
        let cancel = Cancellation::default();
        apply_change(
            &mut db,
            "a",
            &BookmarkChange::Save {
                page: 7,
                label: Some("  Étude · دراسة  ".into()),
            },
            &cancel,
        )
        .unwrap();
        let page = db.bookmarks("a", 0, &cancel).unwrap();
        assert_eq!(page.total, 1);
        let bookmark = &page.entries[0];
        assert_eq!(bookmark.id, "legacy-id");
        assert_eq!(bookmark.created_at, "2020-01-02T03:04:05Z");
        assert_eq!(bookmark.label.as_deref(), Some("Étude · دراسة"));
        assert_ne!(bookmark.updated_at, bookmark.created_at);
        assert_eq!(
            db.bookmarks("b", 0, &cancel).unwrap().entries[0]
                .label
                .as_deref(),
            Some("Other")
        );
        assert!(
            apply_change(
                &mut db,
                "b",
                &BookmarkChange::Remove {
                    id: "legacy-id".into()
                },
                &cancel
            )
            .is_err()
        );
        assert!(
            apply_change(
                &mut db,
                "b",
                &BookmarkChange::Rename {
                    id: "legacy-id".into(),
                    label: None
                },
                &cancel
            )
            .is_err()
        );
        apply_change(
            &mut db,
            "a",
            &BookmarkChange::Rename {
                id: "legacy-id".into(),
                label: Some("  ".into()),
            },
            &cancel,
        )
        .unwrap();
        assert_eq!(
            db.bookmarks("a", 0, &cancel).unwrap().entries[0].label,
            None
        );
    }

    #[test]
    fn paging_is_bounded_sorted_and_clamps_after_the_last_page_is_removed() {
        let mut db = database();
        let cancel = Cancellation::default();
        for page in (0..161).rev() {
            apply_change(
                &mut db,
                "a",
                &BookmarkChange::Save { page, label: None },
                &cancel,
            )
            .unwrap();
        }
        let first = db.bookmarks("a", 0, &cancel).unwrap();
        assert_eq!((first.total, first.entries.len()), (161, 80));
        assert_eq!(
            (first.entries[0].page_index, first.entries[79].page_index),
            (0, 79)
        );
        let last = db.bookmarks("a", usize::MAX, &cancel).unwrap();
        assert_eq!((last.offset, last.entries.len()), (160, 1));
        apply_change(
            &mut db,
            "a",
            &BookmarkChange::Remove {
                id: last.entries[0].id.clone(),
            },
            &cancel,
        )
        .unwrap();
        let last = db.bookmarks("a", 160, &cancel).unwrap();
        assert_eq!((last.offset, last.entries.len(), last.total), (80, 80, 160));
    }

    #[test]
    fn invalid_and_cancelled_writes_leave_persisted_bookmarks_unchanged() {
        let mut db = database();
        let cancel = Cancellation::default();
        apply_change(
            &mut db,
            "a",
            &BookmarkChange::Save {
                page: 4999,
                label: None,
            },
            &cancel,
        )
        .unwrap();
        let before = db.bookmarks("a", 0, &cancel).unwrap();
        for (id, change) in [
            (
                "b",
                BookmarkChange::Remove {
                    id: before.entries[0].id.clone(),
                },
            ),
            (
                "missing",
                BookmarkChange::Save {
                    page: 0,
                    label: None,
                },
            ),
            (
                "a",
                BookmarkChange::Save {
                    page: 8,
                    label: Some("x".repeat(4097)),
                },
            ),
        ] {
            assert!(apply_change(&mut db, id, &change, &cancel).is_err());
        }
        let stopped = Cancellation::default();
        stopped.cancel();
        assert!(matches!(
            apply_change(
                &mut db,
                "a",
                &BookmarkChange::Remove {
                    id: before.entries[0].id.clone()
                },
                &stopped
            ),
            Err(Failure::Cancelled)
        ));
        assert!(matches!(
            db.bookmarks("a", 0, &stopped),
            Err(Failure::Cancelled)
        ));
        assert_eq!(db.bookmarks("a", 0, &cancel).unwrap(), before);
    }
}
