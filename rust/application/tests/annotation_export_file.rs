use std::fs;
use yamantaka_application::annotation_export::*;
use yamantaka_domain::{Cancellation, Failure, memory::MemoryGovernor};
use yamantaka_storage::Database;

fn fixture() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa'); WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180) INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state) SELECT printf('r%03d',x),'a',x/80,'pageComment','blue','Study 学习','[]',0,'fa','attached','2026-10-08T12:00:00Z','2026-10-09T12:00:00Z','pending' FROM n;").unwrap();
    db
}
fn metadata() -> AnnotationExportMetadata<'static> {
    AnnotationExportMetadata {
        document: "a",
        title: "Étude 学习",
        author: None,
        progress: 0.5,
        exported_at: "2026-10-10T12:00:00Z",
    }
}

#[test]
fn the_connected_service_publishes_all_rows_in_both_formats() {
    let db = fixture();
    let governor = MemoryGovernor::default();
    let dir = tempfile::tempdir().unwrap();
    for (name, format) in [
        ("Étude.json", AnnotationExportFormat::Json),
        ("Étude.md", AnnotationExportFormat::Markdown),
    ] {
        let path = dir.path().join(name);
        let cancel = Cancellation::default();
        let count = db
            .create_annotation_export(&path, &cancel, |file| {
                let (_, count) = write_document_annotations(
                    &db,
                    file,
                    format,
                    metadata(),
                    &governor,
                    cancel.clone(),
                )?;
                assert!(!path.exists());
                Ok(count)
            })
            .unwrap();
        assert_eq!(count, 181);
        assert_eq!(governor.usage().text, 0);
        let content = fs::read_to_string(path).unwrap();
        if name.ends_with(".json") {
            let json: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert_eq!(json["annotations"].as_array().unwrap().len(), 181);
            assert_eq!(json["annotations"][180]["id"], "r180");
        } else {
            assert_eq!(content.matches("Study 学习").count(), 181);
            assert!(content.contains("page-3"));
        }
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn a_malformed_late_record_leaves_no_partial_export_and_releases_memory() {
    let db = fixture();
    db.execute_batch("UPDATE annotations SET created_at='invalid' WHERE id='r180';")
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let result = db.create_annotation_export(&path, &cancel, |file| {
        write_document_annotations(
            &db,
            file,
            AnnotationExportFormat::Json,
            metadata(),
            &governor,
            cancel.clone(),
        )
        .map(|(_, count)| count)
    });
    assert!(matches!(result, Err(Failure::Invalid(_))));
    assert_eq!(governor.usage().text, 0);
    assert!(!path.exists());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn cancellation_inside_one_large_note_discards_staged_output_in_both_formats() {
    use std::io::{self, Write};
    struct CancellingFile<'a> {
        file: &'a mut fs::File,
        cancel: Cancellation,
        bytes: usize,
    }
    impl Write for CancellingFile<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let count = self.file.write(bytes)?;
            self.bytes += count;
            self.cancel.cancel();
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.file.flush()
        }
    }
    let db = fixture();
    db.execute_batch(
        "UPDATE annotations SET note=replace(hex(zeroblob(128000)),'00','a') WHERE id='r000';",
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let governor = MemoryGovernor::default();
    for (name, format) in [
        ("notes.json", AnnotationExportFormat::Json),
        ("notes.md", AnnotationExportFormat::Markdown),
    ] {
        let path = dir.path().join(name);
        let cancel = Cancellation::default();
        let mut written = 0;
        let result = db.create_annotation_export(&path, &cancel, |file| {
            let mut writer = CancellingFile {
                file,
                cancel: cancel.clone(),
                bytes: 0,
            };
            let result = write_document_annotations(
                &db,
                &mut writer,
                format,
                metadata(),
                &governor,
                cancel.clone(),
            )
            .map(|(_, count)| count);
            written = writer.bytes;
            result
        });
        assert!(matches!(result, Err(Failure::Cancelled)));
        assert!(
            written > 0 && written < 128000,
            "Cancellation must stop inside the first note"
        );
        assert!(!path.exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn denied_output_memory_leaves_no_file_and_a_fresh_governor_can_retry() {
    use yamantaka_domain::memory::GlobalMemoryLimits;
    let db = fixture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let cancel = Cancellation::default();
    let denied = MemoryGovernor::new(GlobalMemoryLimits {
        other: Some(0),
        ..Default::default()
    });
    let result = db.create_annotation_export(&path, &cancel, |file| {
        write_document_annotations(
            &db,
            file,
            AnnotationExportFormat::Json,
            metadata(),
            &denied,
            cancel.clone(),
        )
        .map(|(_, count)| count)
    });
    assert!(matches!(result, Err(Failure::MemoryBudget)));
    assert_eq!(denied.usage().total, 0);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    let governor = MemoryGovernor::default();
    assert_eq!(
        db.create_annotation_export(&path, &cancel, |file| {
            write_document_annotations(
                &db,
                file,
                AnnotationExportFormat::Json,
                metadata(),
                &governor,
                cancel.clone(),
            )
            .map(|(_, count)| count)
        })
        .unwrap(),
        181
    );
    assert_eq!(governor.usage().total, 0);
}
