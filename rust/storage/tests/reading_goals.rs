use rusqlite::Connection;
use serde_json::{Value, json};
use yamantaka_domain::{
    Cancellation, Failure,
    reading_sessions::{ReadingSessionRepository, goals::ReadingGoal},
};
use yamantaka_storage::Database;

fn seed(db: &Database) {
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',2,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf',2,1,'now','now','now','fb');").unwrap();
}
fn metadata(read: &Connection) -> Option<(String, String)> {
    use rusqlite::OptionalExtension;
    read.query_row(
        "SELECT value,updated_at FROM app_metadata WHERE key='reading_goals_v1'",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
    .unwrap()
}
#[test]
fn legacy_bounds_are_read_without_rewriting_and_updates_preserve_extensions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(path).unwrap();
    let id = "legacy\"quoted\\Ω";
    read.execute("UPDATE documents SET id=?1 WHERE id='a'", [id])
        .unwrap();
    let original = json!({id:{"pages":0,"minutes":9000,"future":{"kept":true}},"b":{"pages":8},"array":[true,1,null],"true":true,"false":false,"null":null,"number":1.25,"text":"quoted \"Ω\""});
    read.execute(
        "INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_goals_v1',?1,'original')",
        [original.to_string()],
    )
    .unwrap();
    let before = metadata(&read);
    assert_eq!(
        db.reading_goal(id, &Cancellation::default()).unwrap(),
        Some(ReadingGoal::new(Some(1), Some(1440)).unwrap())
    );
    assert_eq!(metadata(&read), before);
    let goal = ReadingGoal::new(Some(25), None).unwrap();
    assert_eq!(
        db.save_reading_goal(id, Some(goal), &Cancellation::default())
            .unwrap(),
        Some(goal)
    );
    let stored: Value = serde_json::from_str(&metadata(&read).unwrap().0).unwrap();
    assert_eq!(stored[id], json!({"pages":25,"future":{"kept":true}}));
    for (key, value) in original.as_object().unwrap() {
        if key != id {
            assert_eq!(&stored[key], value);
        }
    }
    assert_eq!(
        db.save_reading_goal(id, None, &Cancellation::default())
            .unwrap(),
        None
    );
    let stored: Value = serde_json::from_str(&metadata(&read).unwrap().0).unwrap();
    assert!(stored.get(id).is_none());
    assert_eq!(stored["b"], original["b"]);
    let before = metadata(&read);
    assert_eq!(
        db.save_reading_goal(id, None, &Cancellation::default())
            .unwrap(),
        None
    );
    assert_eq!(metadata(&read), before);
}
#[test]
fn invalid_known_data_is_preserved_and_cancelled_or_missing_targets_do_not_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(path).unwrap();
    let goal = ReadingGoal::new(Some(12), Some(30)).unwrap();
    for raw in [
        "broken",
        "[]",
        r#"{"a":false}"#,
        r#"{"a":{"pages":true}}"#,
        r#"{"a":{"minutes":2.5}}"#,
        r#"{"a":{"pages":1,"pages":2}}"#,
        r#"{"a":{},"a":{}}"#,
        r#"{"a":{"pages":9223372036854775808}}"#,
    ] {
        read.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_goals_v1',?1,'original') ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at",[raw]).unwrap();
        let before = metadata(&read);
        assert!(matches!(
            db.reading_goal("a", &Cancellation::default()),
            Err(Failure::Invalid(_))
        ));
        assert!(
            db.save_reading_goal("a", Some(goal), &Cancellation::default())
                .is_err()
        );
        assert!(
            db.save_reading_goal("a", None, &Cancellation::default())
                .is_err()
        );
        assert_eq!(metadata(&read), before);
    }
    read.execute("UPDATE app_metadata SET value='{}'", [])
        .unwrap();
    let before = metadata(&read);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        db.save_reading_goal("a", Some(goal), &cancel),
        Err(Failure::Cancelled)
    ));
    assert!(
        db.save_reading_goal("missing", Some(goal), &Cancellation::default())
            .is_err()
    );
    assert_eq!(metadata(&read), before);
}
#[test]
fn late_failure_and_ignored_writes_roll_back_metadata_and_time() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(path).unwrap();
    read.execute("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_goals_v1','{\"a\":{\"pages\":8,\"future\":true}}','original')",[]).unwrap();
    let goal = ReadingGoal::new(Some(12), Some(30)).unwrap();
    for action in ["RAISE(ABORT,'late failure')", "RAISE(IGNORE)"] {
        read.execute_batch(&format!(
            "CREATE TRIGGER fail_goal BEFORE UPDATE ON app_metadata BEGIN SELECT {action}; END;"
        ))
        .unwrap();
        let before = metadata(&read);
        assert!(
            db.save_reading_goal("a", Some(goal), &Cancellation::default())
                .is_err()
        );
        assert!(
            db.save_reading_goal("a", None, &Cancellation::default())
                .is_err()
        );
        assert_eq!(metadata(&read), before);
        read.execute_batch("DROP TRIGGER fail_goal;").unwrap();
    }
    read.execute_batch("CREATE TRIGGER fail_goal BEFORE DELETE ON app_metadata BEGIN SELECT RAISE(ABORT,'late failure'); END;").unwrap();
    let before = metadata(&read);
    assert!(
        db.save_reading_goal("a", None, &Cancellation::default())
            .is_err()
    );
    assert_eq!(metadata(&read), before);
}
#[test]
fn first_goal_and_clear_last_goal_match_the_legacy_metadata_contract() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(path).unwrap();
    let goal = ReadingGoal::new(None, Some(45)).unwrap();
    assert_eq!(
        db.reading_goal("a", &Cancellation::default()).unwrap(),
        None
    );
    assert_eq!(
        db.save_reading_goal("a", Some(goal), &Cancellation::default())
            .unwrap(),
        Some(goal)
    );
    assert_eq!(
        db.reading_goal("a", &Cancellation::default()).unwrap(),
        Some(goal)
    );
    assert_eq!(
        db.save_reading_goal(
            "a",
            Some(ReadingGoal::new(None, None).unwrap()),
            &Cancellation::default()
        )
        .unwrap(),
        None
    );
    assert!(metadata(&read).is_none());
}

#[test]
fn ignored_removal_of_the_last_metadata_record_rolls_back_the_clear() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut db = Database::open(&path).unwrap();
    seed(&db);
    let read = Connection::open(path).unwrap();
    read.execute_batch("INSERT INTO app_metadata(key,value,updated_at) VALUES('reading_goals_v1','{\"a\":{\"pages\":8}}','original'); CREATE TRIGGER ignore_clear BEFORE DELETE ON app_metadata BEGIN SELECT RAISE(IGNORE); END;").unwrap();
    let before = metadata(&read);
    assert!(
        db.save_reading_goal("a", None, &Cancellation::default())
            .is_err()
    );
    assert_eq!(metadata(&read), before);
}
