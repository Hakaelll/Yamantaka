//! Synthetic fixtures use the exact historical SQLite column contracts.
//! They supplement, rather than impersonate, exports from historical executables.
use rusqlite::{Connection, OpenFlags, types::Value};
use yamantaka_storage::Database;

const FIXTURES: [&str; 11] = [
    include_str!("fixtures/schema-v01.sql"),
    include_str!("fixtures/schema-v02.sql"),
    include_str!("fixtures/schema-v03.sql"),
    include_str!("fixtures/schema-v04.sql"),
    include_str!("fixtures/schema-v05.sql"),
    include_str!("fixtures/schema-v06.sql"),
    include_str!("fixtures/schema-v07.sql"),
    include_str!("fixtures/schema-v08.sql"),
    include_str!("fixtures/schema-v09.sql"),
    include_str!("fixtures/schema-v10.sql"),
    include_str!("fixtures/schema-v11.sql"),
];

fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
fn rows(connection: &Connection, table: &str, columns: &[String]) -> Vec<Vec<Value>> {
    let columns_sql = columns
        .iter()
        .map(|c| quote(c))
        .collect::<Vec<_>>()
        .join(",");
    let mut statement = connection
        .prepare(&format!(
            "SELECT {columns_sql} FROM {} ORDER BY {columns_sql}",
            quote(table)
        ))
        .unwrap();
    statement
        .query_map([], |row| {
            (0..columns.len()).map(|index| row.get(index)).collect()
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[test]
fn all_populated_historical_tables_keep_every_original_column_value() {
    let root = tempfile::tempdir().unwrap();
    for (index, fixture) in FIXTURES.into_iter().enumerate() {
        let version = index as u32 + 1;
        let mut db = Database::in_memory_at(version).unwrap();
        db.execute_batch(fixture).unwrap();
        let before = root.path().join(format!("before-{version}.sqlite3"));
        db.snapshot(&before).unwrap();
        let before =
            Connection::open_with_flags(&before, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        db.migrate_current().unwrap();
        db.migrate_current().unwrap();
        db.integrity().unwrap();
        let after = root.path().join(format!("after-{version}.sqlite3"));
        db.snapshot(&after).unwrap();
        let after = Connection::open_with_flags(&after, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let tables = before.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'library_search_%' ORDER BY name").unwrap()
            .query_map([], |row| row.get::<_, String>(0)).unwrap()
            .collect::<rusqlite::Result<Vec<_>>>().unwrap();
        for table in tables {
            let columns = before
                .prepare(&format!("PRAGMA table_info({})", quote(&table)))
                .unwrap()
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            assert_eq!(
                rows(&before, &table, &columns),
                rows(&after, &table, &columns),
                "schema {version}, table {table}: original values changed"
            );
        }
    }
}
