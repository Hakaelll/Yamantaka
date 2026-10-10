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
