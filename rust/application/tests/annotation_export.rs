use std::io::{self, Write};
use yamantaka_application::annotation_export::*;
use yamantaka_domain::{
    Cancellation, Failure,
    annotations::{repository::*, *},
    memory::*,
    pdf_geometry::NormalizedPageRect,
};
use yamantaka_storage::Database;
fn metadata() -> AnnotationExportMetadata<'static> {
    AnnotationExportMetadata {
        document: "a",
        title: "Study *[book]* <script>",
        author: Some("Auteur 学习"),
        progress: 1.5,
        exported_at: "2026-10-09T12:00:00Z",
    }
}
fn record(governor: &MemoryGovernor, page: u32) -> AnnotationRecord {
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .unwrap();
    AnnotationRecord::from_reserved(
        AnnotationRecordData {
            id: format!("r{page}"),
            values: AnnotationDraftValues {
                document_id: "a".into(),
                page_index: page,
                kind: AnnotationKind::Underline,
                color: AnnotationColor::Violet,
                note: Some("A <b>note</b> #tag".into()),
                selected_text: Some("café\n学习 אבג".into()),
                context_before: Some("Before".into()),
                context_after: Some("After".into()),
                rects: vec![
                    NormalizedPageRect::new([0.1, 0.2], [0.3, 0.4])
                        .unwrap()
                        .into(),
                ],
                page_rotation: 270,
                document_fingerprint: "original-sha".into(),
                status: AttachmentStatus::ReviewRequired,
                review_state: AnnotationReviewState::Resolved,
            },
            tags: vec!["Optics".into(), "Étude".into()],
            created_at: "2026-10-08T14:00:00+02:00".into(),
            updated_at: "2026-10-09T13:00:00+01:00".into(),
            origin_document_id: Some("original".into()),
            origin_fingerprint: Some("origin-sha".into()),
            extensions: vec![],
        },
        lease,
    )
    .unwrap()
}
#[test]
fn versioned_json_preserves_unicode_geometry_identity_tags_and_review() {
    let governor = MemoryGovernor::default();
    let record = record(&governor, u32::MAX);
    let mut exporter = AnnotationExporter::new(
        Vec::new(),
        AnnotationExportFormat::Json,
        metadata(),
        Cancellation::default(),
    )
    .unwrap();
    exporter.push(&record).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&exporter.finish().unwrap()).unwrap();
    assert_eq!(json["format"], "yamantaka-annotations");
    assert_eq!(json["version"], 1);
    assert_eq!(json["configuration"]["coordinates"], "normalized-page-v1");
    assert_eq!(json["metadata"]["progress"], 1.0);
    assert_eq!(json["metadata"]["documentId"], "a");
    let entry = &json["annotations"][0];
    assert_eq!(entry["pageIndex"], u32::MAX);
    assert_eq!(entry["type"], "underline");
    assert_eq!(entry["color"], "violet");
    assert_eq!(entry["selectedText"], "café\n学习 אבג");
    assert_eq!(
        entry["rects"][0],
        serde_json::json!({"left":0.1,"top":0.2,"right":0.3,"bottom":0.4})
    );
    assert_eq!(entry["pageRotation"], 270);
    assert_eq!(entry["documentFingerprint"], "original-sha");
    assert_eq!(entry["status"], "reviewRequired");
    assert_eq!(entry["reviewState"], "resolved");
    assert_eq!(entry["originDocumentId"], "original");
    assert_eq!(entry["originFingerprint"], "origin-sha");
    assert_eq!(entry["tags"], serde_json::json!(["Optics", "Étude"]));
    assert_eq!(entry["createdAt"], "2026-10-08T12:00:00Z");
    assert_eq!(entry["updatedAt"], "2026-10-09T12:00:00Z");
    drop(record);
    assert_eq!(governor.usage().text, 0);
}
#[test]
fn markdown_escapes_content_quotes_each_line_and_emits_one_anchor_per_page() {
    let governor = MemoryGovernor::default();
    let record = record(&governor, 0);
    let mut exporter = AnnotationExporter::new(
        Vec::new(),
        AnnotationExportFormat::Markdown,
        metadata(),
        Cancellation::default(),
    )
    .unwrap();
    exporter.push(&record).unwrap();
    exporter.push(&record).unwrap();
    let output = String::from_utf8(exporter.finish().unwrap()).unwrap();
    assert!(output.contains("Study \\*\\[book\\]\\* &lt;script&gt;"));
    assert!(output.contains("- Progreso: 100 %"));
    assert!(output.contains("> café\n> 学习 אבג"));
    assert!(output.contains("A &lt;b&gt;note&lt;/b&gt; \\#tag"));
    assert!(output.contains("Etiquetas: Optics, Étude"));
    assert_eq!(output.matches("<a id=\"page-1\"></a>").count(), 1);
    assert_eq!(output.matches("[Abrir página 1](#page-1)").count(), 2);
}
struct Broken;
impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("private destination details"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn cancellation_invalid_metadata_and_failed_streams_never_finish_successfully() {
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        AnnotationExporter::new(
            Vec::new(),
            AnnotationExportFormat::Json,
            metadata(),
            cancelled
        ),
        Err(Failure::Cancelled)
    ));
    let mut invalid = metadata();
    invalid.progress = f64::NAN;
    assert!(matches!(
        AnnotationExporter::new(
            Vec::new(),
            AnnotationExportFormat::Json,
            invalid,
            Cancellation::default()
        ),
        Err(Failure::Invalid(_))
    ));
    let error = AnnotationExporter::new(
        Broken,
        AnnotationExportFormat::Json,
        metadata(),
        Cancellation::default(),
    )
    .err()
    .unwrap();
    assert!(matches!(error, Failure::Storage(_)));
    assert!(!error.to_string().contains("private destination"));
    let governor = MemoryGovernor::default();
    let later = record(&governor, 2);
    let earlier = record(&governor, 1);
    let mut exporter = AnnotationExporter::new(
        Vec::new(),
        AnnotationExportFormat::Json,
        metadata(),
        Cancellation::default(),
    )
    .unwrap();
    exporter.push(&later).unwrap();
    assert!(exporter.push(&earlier).is_err());
    assert!(exporter.push(&later).is_err());
    assert!(exporter.finish().is_err());
    let mut other = metadata();
    other.document = "b";
    let mut exporter = AnnotationExporter::new(
        Vec::new(),
        AnnotationExportFormat::Json,
        other,
        Cancellation::default(),
    )
    .unwrap();
    assert!(exporter.push(&later).is_err());
    assert!(exporter.finish().is_err());
    let cancel = Cancellation::default();
    let mut exporter = AnnotationExporter::new(
        Vec::new(),
        AnnotationExportFormat::Json,
        metadata(),
        cancel.clone(),
    )
    .unwrap();
    exporter.push(&earlier).unwrap();
    cancel.cancel();
    assert!(matches!(exporter.finish(), Err(Failure::Cancelled)));
}
#[test]
fn export_streams_across_repository_pages_without_truncating_at_eighty() {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf',3,1,'now','now','now','fa'); WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180) INSERT INTO annotations(id,document_id,page_index,type,color,note,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state) SELECT printf('r%03d',x),'a',x/80,'pageComment','blue','Study','[]',0,'fa','attached','2026-10-08T12:00:00Z','2026-10-09T12:00:00Z','pending' FROM n;").unwrap();
    let governor = MemoryGovernor::default();
    let mut exporter = AnnotationExporter::new(
        Vec::new(),
        AnnotationExportFormat::Json,
        metadata(),
        Cancellation::default(),
    )
    .unwrap();
    for offset in [0, 80, 160] {
        let page = db
            .annotations(
                &AnnotationQuery {
                    document_id: Some("a".into()),
                    offset,
                    ..Default::default()
                },
                &governor,
                &Cancellation::default(),
            )
            .unwrap();
        for record in page.entries() {
            exporter.push(record).unwrap();
        }
    }
    assert_eq!(governor.usage().text, 0);
    let json: serde_json::Value = serde_json::from_slice(&exporter.finish().unwrap()).unwrap();
    let entries = json["annotations"].as_array().unwrap();
    assert_eq!(entries.len(), 181);
    assert_eq!(entries[180]["id"], "r180");
    assert!(entries[0].get("selectedText").is_none());
    assert_eq!(entries[0]["rects"], serde_json::json!([]));
    let empty = AnnotationExporter::new(
        Vec::new(),
        AnnotationExportFormat::Json,
        metadata(),
        Cancellation::default(),
    )
    .unwrap()
    .finish()
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&empty).unwrap()["annotations"],
        serde_json::json!([])
    );
}

#[test]
fn a_late_write_or_flush_failure_cannot_return_a_successful_export() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct Switched {
        fail: Arc<AtomicBool>,
        flush: bool,
    }
    impl Write for Switched {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            if self.fail.load(Ordering::Relaxed) {
                Err(io::Error::other("private destination details"))
            } else {
                Ok(data.len())
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            if self.flush {
                Err(io::Error::other("private flush details"))
            } else {
                Ok(())
            }
        }
    }
    let governor = MemoryGovernor::default();
    let row = record(&governor, 0);
    let fail = Arc::new(AtomicBool::new(false));
    let mut exporter = AnnotationExporter::new(
        Switched {
            fail: fail.clone(),
            flush: false,
        },
        AnnotationExportFormat::Json,
        metadata(),
        Cancellation::default(),
    )
    .unwrap();
    fail.store(true, Ordering::Relaxed);
    let error = exporter.push(&row).unwrap_err();
    assert!(matches!(error, Failure::Storage(_)));
    assert!(!error.to_string().contains("private destination"));
    fail.store(false, Ordering::Relaxed);
    assert!(exporter.push(&row).is_err());
    assert!(exporter.finish().is_err());
    let mut exporter = AnnotationExporter::new(
        Switched { fail, flush: true },
        AnnotationExportFormat::Markdown,
        metadata(),
        Cancellation::default(),
    )
    .unwrap();
    exporter.push(&row).unwrap();
    assert!(matches!(exporter.finish(), Err(Failure::Storage(_))));
    let mut invalid = metadata();
    invalid.exported_at = "invalid timestamp";
    assert!(matches!(
        AnnotationExporter::new(
            Vec::new(),
            AnnotationExportFormat::Json,
            invalid,
            Cancellation::default()
        ),
        Err(Failure::Invalid(_))
    ));
    drop(row);
    assert_eq!(governor.usage().text, 0);
}
