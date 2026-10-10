use yamantaka_application::annotation_export::*;
use yamantaka_domain::{Cancellation, memory::MemoryGovernor};
use yamantaka_storage::Database;

#[test]
fn markdown_preserves_legacy_line_boundaries_and_json_preserves_original_bytes() {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',1,1,'now','now','now','fa'); INSERT INTO annotations(id,document_id,page_index,type,color,note,selected_text,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state) VALUES('r','a',0,'pageComment','blue','note','passage','[]',0,'fa','attached','2026-10-08T12:00:00Z','2026-10-09T12:00:00Z','pending');").unwrap();
    let governor = MemoryGovernor::default();
    for (original, normalized, quoted) in [
        ("café\n学习\nאבג", "café\n学习\nאבג", "café\n> 学习\n> אבג"),
        (
            "café\r\n学习\r\nאבג",
            "café\n学习\nאבג",
            "café\n> 学习\n> אבג",
        ),
        ("café\r学习\rאבג", "café\n学习\nאבג", "café\n> 学习\n> אבג"),
        (
            "\rcafé\r\n学习\nאבג\r",
            "\ncafé\n学习\nאבג\n",
            "\n> café\n> 学习\n> אבג\n> ",
        ),
        ("café\r\r\n学习", "café\n\n学习", "café\n> \n> 学习"),
    ] {
        db.execute_batch(&format!(
            "UPDATE annotations SET note='{original}', selected_text='{original}';"
        ))
        .unwrap();
        for format in [
            AnnotationExportFormat::Markdown,
            AnnotationExportFormat::Json,
        ] {
            let (bytes, count) = write_document_annotations(
                &db,
                Vec::new(),
                format,
                AnnotationExportMetadata {
                    document: "a",
                    title: "Study\rLibrary",
                    author: Some("A\r\nB"),
                    progress: 0.5,
                    exported_at: "2026-10-10T12:00:00Z",
                },
                &governor,
                Cancellation::default(),
            )
            .unwrap();
            assert_eq!(count, 1);
            match format {
                AnnotationExportFormat::Markdown => {
                    let output = String::from_utf8(bytes).unwrap();
                    assert!(!output.contains('\r'));
                    assert!(output.starts_with("# Study\nLibrary\n"));
                    assert!(output.contains("- Autor: A\nB\n"));
                    assert!(output.contains(&format!("> {quoted}\n")));
                    assert!(output.contains(&format!("\n{normalized}\n")));
                }
                AnnotationExportFormat::Json => {
                    let output: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                    assert_eq!(output["annotations"][0]["note"], original);
                    assert_eq!(output["annotations"][0]["selectedText"], original);
                    assert_eq!(output["metadata"]["title"], "Study\rLibrary");
                    assert_eq!(output["metadata"]["author"], "A\r\nB");
                }
            }
            assert_eq!(governor.usage().total, 0);
        }
    }
}
