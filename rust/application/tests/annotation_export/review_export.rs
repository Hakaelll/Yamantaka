use super::*;

#[test]
fn coherent_review_snapshot_exports_only_the_frozen_filtered_page() {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,title_override,author_override,reading_progress,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf','Source','A manual','Manual author',0.5,3,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf','B',NULL,NULL,0.25,3,1,'now','now','now','fb'); WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180) INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) SELECT printf('review-%03d',x),CASE WHEN x<2 THEN 'a' ELSE 'b' END,x%3,'pageComment','violet','Study',CASE WHEN x<2 THEN 'fa' ELSE 'fb' END,'2026-10-10T12:00:00Z','2026-10-10T12:00:00Z' FROM n;").unwrap();
    let governor = MemoryGovernor::default();
    let snapshot = db
        .annotation_review_snapshot(
            &AnnotationQuery {
                grouping: Some(AnnotationGrouping::Document),
                ..Default::default()
            },
            &governor,
            &Cancellation::default(),
        )
        .unwrap();
    db.execute_batch("UPDATE documents SET title_override='Changed',author_override='Changed',reading_progress=0.75 WHERE id='a'; UPDATE annotations SET note='Changed';").unwrap();
    let (bytes, count) = write_review_snapshot(
        Vec::new(),
        AnnotationExportFormat::Json,
        &snapshot,
        "2026-10-10T12:00:00Z",
        &governor,
        Cancellation::default(),
    )
    .unwrap();
    assert_eq!(count, 80);
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["documents"][0]["metadata"]["title"], "A manual");
    assert_eq!(value["documents"][0]["metadata"]["author"], "Manual author");
    assert_eq!(value["documents"][0]["metadata"]["progress"], 0.5);
    let docs = value["documents"].as_array().unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0]["annotations"].as_array().unwrap().len(), 2);
    assert_eq!(docs[1]["annotations"].as_array().unwrap().len(), 78);
    assert!(
        docs.iter()
            .flat_map(|d| d["annotations"].as_array().unwrap())
            .all(|a| a["note"] == "Study")
    );
    assert_eq!(governor.usage().other, 0);
    drop(snapshot);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn review_export_preserves_the_global_envelope_and_legacy_document_bodies() {
    let governor = MemoryGovernor::default();
    let later = record(&governor, 9);
    let earlier = record(&governor, 2);
    let other = record_for(&governor, 0, "b");
    let a = [&later, &earlier];
    let b = [&other];
    let mut second = metadata();
    second.document = "b";
    second.title = "Second 学习 book";
    second.progress = 0.25;
    let documents = [
        AnnotationReviewDocument {
            metadata: metadata(),
            annotations: &a,
        },
        AnnotationReviewDocument {
            metadata: second,
            annotations: &b,
        },
    ];
    let date = "2026-10-10T14:00:00+02:00";
    let (json, count) = write_annotation_review(
        Vec::new(),
        AnnotationExportFormat::Json,
        &documents,
        date,
        &governor,
        Cancellation::default(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(count, 3);
    assert_eq!(value["format"], "yamantaka-global-annotations");
    assert_eq!(value["version"], 1);
    assert_eq!(value["exportedAt"], "2026-10-10T12:00:00Z");
    assert_eq!(value["documents"].as_array().unwrap().len(), 2);
    assert_eq!(value["documents"][0]["metadata"]["progress"], 1.0);
    assert_eq!(value["documents"][1]["metadata"]["progress"], 0.25);
    for (index, records) in [&[&earlier, &later][..], &b[..]].iter().enumerate() {
        let m = &documents[index].metadata;
        let mut standalone = AnnotationExporter::new(
            Vec::new(),
            AnnotationExportFormat::Json,
            AnnotationExportMetadata {
                document: m.document,
                title: m.title,
                author: m.author,
                progress: m.progress,
                exported_at: date,
            },
            Cancellation::default(),
        )
        .unwrap();
        for row in *records {
            standalone.push(row).unwrap();
        }
        let expected: serde_json::Value =
            serde_json::from_slice(&standalone.finish().unwrap()).unwrap();
        assert_eq!(value["documents"][index], expected);
    }
    let (markdown, count) = write_annotation_review(
        Vec::new(),
        AnnotationExportFormat::Markdown,
        &documents,
        date,
        &governor,
        Cancellation::default(),
    )
    .unwrap();
    assert_eq!(count, 3);
    let text = String::from_utf8(markdown).unwrap();
    assert!(text.contains("\n---\n\n# Second 学习 book"));
    assert!(text.find("Página 3").unwrap() < text.find("Página 10").unwrap());
    assert!(text.contains("> café\n> 学习 אבג"));
    assert_eq!(governor.usage().other, 0);
    drop((later, earlier, other));
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn review_export_rejects_invalid_frozen_groups_before_writing() {
    let governor = MemoryGovernor::default();
    let row = record(&governor, 0);
    let mut output = Vec::new();
    for refs in [vec![&row, &row], vec![&row; 81]] {
        let groups = [AnnotationReviewDocument {
            metadata: metadata(),
            annotations: &refs,
        }];
        assert!(matches!(
            write_annotation_review(
                &mut output,
                AnnotationExportFormat::Json,
                &groups,
                "2026-10-10T12:00:00Z",
                &governor,
                Cancellation::default()
            ),
            Err(Failure::Invalid(_))
        ));
        assert!(output.is_empty());
    }
    let refs = [&row];
    let mut wrong = metadata();
    wrong.document = "b";
    let groups = [AnnotationReviewDocument {
        metadata: wrong,
        annotations: &refs,
    }];
    assert!(matches!(
        write_annotation_review(
            &mut output,
            AnnotationExportFormat::Json,
            &groups,
            "2026-10-10T12:00:00Z",
            &governor,
            Cancellation::default()
        ),
        Err(Failure::Invalid(_))
    ));
    assert!(output.is_empty());
    let groups = [
        AnnotationReviewDocument {
            metadata: metadata(),
            annotations: &refs,
        },
        AnnotationReviewDocument {
            metadata: metadata(),
            annotations: &refs,
        },
    ];
    assert!(matches!(
        write_annotation_review(
            &mut output,
            AnnotationExportFormat::Json,
            &groups,
            "2026-10-10T12:00:00Z",
            &governor,
            Cancellation::default()
        ),
        Err(Failure::Invalid(_))
    ));
    assert!(output.is_empty());
    assert_eq!(governor.usage().other, 0);
}

#[test]
fn empty_and_cancelled_review_exports_do_not_leak_output_memory() {
    let governor = MemoryGovernor::default();
    let (bytes, count) = write_annotation_review(
        Vec::new(),
        AnnotationExportFormat::Json,
        &[],
        "2026-10-10T12:00:00Z",
        &governor,
        Cancellation::default(),
    )
    .unwrap();
    assert_eq!(count, 0);
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(value["documents"].as_array().unwrap().is_empty());
    let cancel = Cancellation::default();
    cancel.cancel();
    let mut bytes = Vec::new();
    assert!(matches!(
        write_annotation_review(
            &mut bytes,
            AnnotationExportFormat::Json,
            &[],
            "2026-10-10T12:00:00Z",
            &governor,
            cancel
        ),
        Err(Failure::Cancelled)
    ));
    assert!(bytes.is_empty());
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn cancellation_after_a_document_flush_cannot_finalize_the_global_envelope() {
    struct CancelOnWrite {
        bytes: Vec<u8>,
        cancel: Cancellation,
    }
    impl Write for CancelOnWrite {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            self.cancel.cancel();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let governor = MemoryGovernor::default();
    let row = record(&governor, 0);
    let rows = [&row];
    let documents = [AnnotationReviewDocument {
        metadata: metadata(),
        annotations: &rows,
    }];
    let cancel = Cancellation::default();
    let mut writer = CancelOnWrite {
        bytes: Vec::new(),
        cancel: cancel.clone(),
    };
    assert!(matches!(
        write_annotation_review(
            &mut writer,
            AnnotationExportFormat::Json,
            &documents,
            "2026-10-10T12:00:00Z",
            &governor,
            cancel
        ),
        Err(Failure::Cancelled)
    ));
    assert!(!writer.bytes.is_empty());
    assert!(serde_json::from_slice::<serde_json::Value>(&writer.bytes).is_err());
    assert_eq!(governor.usage().other, 0);
    drop(row);
    assert_eq!(governor.usage().total, 0);
}
