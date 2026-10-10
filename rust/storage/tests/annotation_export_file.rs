use std::{fs, io::Write, path::Path};
use yamantaka_domain::{Cancellation, Failure};
use yamantaka_storage::Database;

fn entries(path: &Path) -> usize {
    fs::read_dir(path).unwrap().count()
}

#[test]
fn complete_unicode_output_is_published_only_after_the_writer_finishes() {
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().join("Étude 学习.json");
    let db = Database::in_memory_at(11).unwrap();
    let result = db
        .create_annotation_export(&destination, &Cancellation::default(), |file| {
            file.write_all("{\"note\":\"Étude 学习 אבג\"}".as_bytes())
                .unwrap();
            assert!(!destination.exists());
            assert_eq!(entries(dir.path()), 1);
            Ok(181)
        })
        .unwrap();
    assert_eq!(result, 181);
    assert_eq!(
        fs::read_to_string(&destination).unwrap(),
        "{\"note\":\"Étude 学习 אבג\"}"
    );
    assert_eq!(entries(dir.path()), 1);
}

#[test]
fn partial_failure_and_cancellation_leave_no_published_file_or_temporary() {
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().join("notes.md");
    let db = Database::in_memory_at(11).unwrap();
    let cancel = Cancellation::default();
    let failed: yamantaka_domain::Result<()> =
        db.create_annotation_export(&destination, &cancel, |file| {
            file.write_all(b"partial").unwrap();
            Err(Failure::Storage("Synthetic writer failure".into()))
        });
    assert!(matches!(failed, Err(Failure::Storage(_))));
    assert_eq!(entries(dir.path()), 0);
    let cancelled = db.create_annotation_export(&destination, &cancel, |file| {
        file.write_all(b"partial").unwrap();
        cancel.cancel();
        Ok(())
    });
    assert!(matches!(cancelled, Err(Failure::Cancelled)));
    assert_eq!(entries(dir.path()), 0);
    assert!(matches!(
        db.create_annotation_export(&destination, &cancel, |_| -> yamantaka_domain::Result<()> {
            panic!("Cancelled writer must not run")
        }),
        Err(Failure::Cancelled)
    ));
}

#[test]
fn existing_and_late_destinations_keep_their_original_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().join("notes.json");
    let db = Database::in_memory_at(11).unwrap();
    fs::write(&destination, b"original").unwrap();
    assert!(matches!(
        db.create_annotation_export(
            &destination,
            &Cancellation::default(),
            |_| -> yamantaka_domain::Result<()> {
                panic!("Existing destinations must fail before writing")
            }
        ),
        Err(Failure::DestinationOccupied)
    ));
    assert_eq!(fs::read(&destination).unwrap(), b"original");
    fs::remove_file(&destination).unwrap();
    assert!(
        db.create_annotation_export(&destination, &Cancellation::default(), |file| {
            file.write_all(b"complete export").unwrap();
            fs::write(&destination, b"external writer").unwrap();
            Ok(())
        })
        .is_err()
    );
    assert_eq!(fs::read(&destination).unwrap(), b"external writer");
    assert_eq!(entries(dir.path()), 1);
}

#[test]
fn missing_document_paths_extensions_and_bad_parents_fail_before_writing() {
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().canonicalize().unwrap().join("source.json");
    let normalized = destination
        .to_string_lossy()
        .to_lowercase()
        .replace('\'', "''");
    let legacy = normalized.strip_prefix(r"\\?\").unwrap_or(&normalized);
    assert!(!legacy.is_empty());
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','{legacy}','{legacy}','source.json',1,1,'now','now','now','fa');")).unwrap();
    for path in [
        destination,
        dir.path().join("source.pdf"),
        dir.path().join("missing").join("notes.md"),
    ] {
        let error = db
            .create_annotation_export(
                &path,
                &Cancellation::default(),
                |_| -> yamantaka_domain::Result<()> {
                    panic!("Invalid destinations must fail before writing")
                },
            )
            .unwrap_err();
        assert!(!error.to_string().contains(&*dir.path().to_string_lossy()));
    }
    assert_eq!(entries(dir.path()), 0);
}

#[test]
fn an_independent_export_reader_cannot_change_the_live_library() {
    use yamantaka_domain::{
        annotations::repository::*, memory::MemoryGovernor, search::IndexAccess,
    };
    let dir = tempfile::tempdir().unwrap();
    let db = Database::open(&dir.path().join("library.sqlite3")).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',1,1,'now','now','now','fa'); INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state) VALUES('r','a',0,'pageComment','blue','Original','[]',0,'fa','attached','2026-10-08T12:00:00Z','2026-10-09T12:00:00Z','pending');").unwrap();
    let mut reader = db.export_reader().unwrap();
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let query = AnnotationQuery {
        document_id: Some("a".into()),
        ..Default::default()
    };
    assert!(matches!(
        reader.change_annotation(
            "a",
            AnnotationChange::Remove { id: "r" },
            &query,
            IndexAccess::Disabled,
            &governor,
            &cancel
        ),
        Err(Failure::Storage(_))
    ));
    let original = db.annotations(&query, &governor, &cancel).unwrap();
    assert_eq!(original.total, 1);
    assert_eq!(
        original.entries()[0].data().values.note.as_deref(),
        Some("Original")
    );
    drop(original);
    db.execute_batch("UPDATE annotations SET note='Changed';")
        .unwrap();
    let changed = reader.annotations(&query, &governor, &cancel).unwrap();
    assert_eq!(
        changed.entries()[0].data().values.note.as_deref(),
        Some("Changed")
    );
    drop(changed);
    assert_eq!(governor.usage().text, 0);
    assert!(matches!(
        Database::in_memory_at(11).unwrap().export_reader(),
        Err(Failure::Unsupported(_))
    ));
}
