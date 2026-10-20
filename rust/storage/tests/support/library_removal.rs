use rusqlite::{Connection, types::Value};
use std::{collections::BTreeMap, fs};
use yamantaka_domain::{Cancellation, Failure, LibraryQuery, LibraryRepository};
use yamantaka_storage::{Database, preferences::Preferences};

fn snapshot(connection: &Connection) -> BTreeMap<String, Vec<Vec<Value>>> {
    let tables = connection.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'library_search_%' ORDER BY name").unwrap()
        .query_map([], |row|row.get::<_,String>(0)).unwrap()
        .collect::<rusqlite::Result<Vec<_>>>().unwrap();
    tables
        .into_iter()
        .map(|table| {
            let name = table.replace('"', "\"\"");
            let mut statement = connection
                .prepare(&format!("SELECT * FROM \"{name}\""))
                .unwrap();
            let count = statement.column_count();
            let mut rows = statement
                .query_map([], |row| {
                    (0..count)
                        .map(|column| row.get::<_, Value>(column))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            rows.sort_by_key(|row| format!("{row:?}"));
            (table, rows)
        })
        .collect()
}

pub(super) fn verify() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("original document.pdf");
    let thumbnail = root.path().join("unowned thumbnail.pdf");
    let source_bytes = fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../build/rust-fixtures/textual.pdf"),
    )
    .unwrap();
    fs::write(&source, &source_bytes).unwrap();
    fs::write(
        &thumbnail,
        b"Unowned file: never delete through stored thumbnail paths",
    )
    .unwrap();
    let thumbnail_bytes = fs::read(&thumbnail).unwrap();
    let path = root.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    db.execute_batch(include_str!("../fixtures/schema-v11.sql"))
        .unwrap();
    let inspect = Connection::open(&path).unwrap();
    inspect
        .execute(
            "UPDATE documents SET path=?1,thumbnail_path=?2",
            rusqlite::params![source.to_str().unwrap(), thumbnail.to_str().unwrap()],
        )
        .unwrap();
    inspect
        .execute(
            "UPDATE document_covers SET small_thumbnail_path=?1,large_thumbnail_path=?1",
            [thumbnail.to_str().unwrap()],
        )
        .unwrap();
    db.execute_batch(r#"
        INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint)
          VALUES('untouched','untouched.pdf','untouched.pdf','untouched.pdf',1,1,'2026-10-20','2026-10-20','2026-10-20','untouched');
        INSERT INTO document_notes VALUES('incoming','Keep this note','2026-10-20');
        INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at)
          VALUES('incoming-bookmark','incoming',0,'Keep this bookmark','2026-10-20','2026-10-20');
        INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at,origin_document_id)
          VALUES('incoming-note','incoming',0,'note','violet','Keep this annotation','incoming','2026-10-20','2026-10-20','original');
        INSERT INTO annotation_tags VALUES('incoming-note','tag');
        INSERT INTO document_tags VALUES('incoming','tag');
        INSERT INTO collection_documents VALUES('collection','incoming','2026-10-20');
        INSERT INTO workspace_tabs(id,workspace_id,document_id,sort_order) VALUES('incoming-tab','workspace','incoming',1);
        INSERT INTO document_relations(id,source_document_id,target_document_id,relation_type,created_at)
          VALUES('reverse','incoming','original','related','2026-10-20');
        UPDATE document_revision_groups SET current_document_id='original';
        INSERT INTO document_revision_members VALUES('revision','incoming','2026-10-20');
        INSERT INTO app_metadata VALUES('reading_goals_v1','{"original":{"pages":12,"future":true},"incoming":{"minutes":20},"unknown":false}','2026-10-20');
        UPDATE search_index_state SET status='excluded',indexed_pages=0,updated_at='old-a' WHERE document_id='original';
        INSERT INTO search_index_state VALUES('incoming','incoming','completed',7,'old-b',NULL);
        INSERT INTO library_search(document_id,page_index,kind,title,body,tags) VALUES
          ('original',-1,'metadata','Remove metadata','cached text','Estudio'),
          ('original',0,'pageText','Remove page','cached text','Estudio'),
          ('original',2,'bookmark','Remove bookmark','cached text',''),
          ('original',2,'annotation','Remove annotation','cached text','Estudio'),
          ('original',-1,'tag','Remove tag','','Estudio'),
          ('incoming',0,'pageText','Keep indexed page','cached text','Estudio'),
          ('gone',0,'pageText','Pre-existing orphan','cached text','');
    "#).unwrap();
    let before = snapshot(&inspect);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        db.remove_from_library("original", &cancelled),
        Err(Failure::Cancelled)
    ));
    for invalid in ["", "   ", "gone"] {
        assert!(
            db.remove_from_library(invalid, &Cancellation::default())
                .is_err()
        );
        assert_eq!(
            snapshot(&inspect),
            before,
            "invalid target must not change any stored row"
        );
    }
    assert!(
        db.remove_from_library(&"x".repeat(4097), &Cancellation::default())
            .is_err()
    );
    assert_eq!(snapshot(&inspect), before);
    // FTS deletion runs first; both root and cascade failures must restore its rows.
    for (table, condition) in [
        ("documents", "OLD.id='original'"),
        ("bookmarks", "OLD.document_id='original'"),
    ] {
        db.execute_batch(&format!("CREATE TRIGGER reject_library_removal BEFORE DELETE ON {table} WHEN {condition} BEGIN SELECT RAISE(ABORT,'synthetic removal interruption'); END;")).unwrap();
        assert!(
            db.remove_from_library("original", &Cancellation::default())
                .is_err()
        );
        assert_eq!(
            snapshot(&inspect),
            before,
            "{table} failure must roll back the whole removal"
        );
        db.execute_batch("DROP TRIGGER reject_library_removal")
            .unwrap();
    }
    db.remove_from_library("original", &Cancellation::default())
        .unwrap();
    assert!(db.get("original").unwrap().is_none());
    assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 2);
    assert_eq!(
        db.get("incoming").unwrap().unwrap().display_title(),
        "Mi título manual"
    );
    let after = snapshot(&inspect);
    for table in [
        "document_enrichment",
        "document_topics",
        "document_preferences",
        "reading_sessions",
        "document_covers",
        "document_relations",
    ] {
        assert!(
            after[table].is_empty(),
            "{table}: dependent rows must be removed"
        );
    }
    for table in [
        "document_notes",
        "bookmarks",
        "annotations",
        "annotation_tags",
        "document_tags",
        "collection_documents",
        "workspace_tabs",
        "document_revision_members",
        "search_index_state",
    ] {
        assert_eq!(
            after[table].len(),
            1,
            "{table}: the other document's row must survive"
        );
    }
    for table in [
        "tags",
        "collections",
        "workspaces",
        "reading_profiles",
        "automation_rules",
        "operation_journal",
        "duplicate_merge_history",
        "app_metadata",
    ] {
        assert_eq!(
            after[table], before[table],
            "{table}: shared and opaque state must be preserved"
        );
    }
    let current: Option<String> = inspect
        .query_row(
            "SELECT current_document_id FROM document_revision_groups WHERE id='revision'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(current, None);
    let rows = |id: &str| -> i64 {
        inspect
            .query_row(
                "SELECT count(*) FROM library_search WHERE document_id=?",
                [id],
                |row| row.get(0),
            )
            .unwrap()
    };
    assert_eq!(rows("original"), 0);
    assert_eq!(rows("incoming"), 1);
    assert_eq!(rows("gone"), 1);
    for table in [
        "document_notes",
        "bookmarks",
        "annotations",
        "search_index_state",
    ] {
        let retained = after[table].first().unwrap();
        assert!(
            before[table].contains(retained),
            "{table}: unselected data must stay byte-for-byte equal"
        );
    }
    assert!(
        db.remove_from_library("original", &Cancellation::default())
            .is_err()
    );
    assert_eq!(snapshot(&inspect), after);
    db.integrity().unwrap();
    let backup = root.path().join("after-removal.json");
    db.export_backup(&backup, &Preferences::default(), &Cancellation::default())
        .unwrap();
    let restored = yamantaka_storage::backup::preview(&backup, &Cancellation::default()).unwrap();
    assert!(restored.database.get("original").unwrap().is_none());
    assert_eq!(restored.counts["documents"], 2);
    assert_eq!(restored.counts["annotations"], 1);
    assert_eq!(fs::read(source).unwrap(), source_bytes);
    assert_eq!(fs::read(thumbnail).unwrap(), thumbnail_bytes);
}
