use super::*;
use yamantaka_application::annotation_export::{
    AnnotationExportFormat,
    worker::{AnnotationExportRequest, AnnotationExportScope},
};
use yamantaka_domain::{Failure, annotations::repository::*, memory::*};

#[test]
fn global_review_exports_publish_filtered_pages_while_pdf_work_is_held() {
    let directory = tempfile::tempdir().unwrap();
    let db = Database::open(&directory.path().join("library.sqlite3")).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,title_override,author_override,reading_progress,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('a','a.pdf','a.pdf','a.pdf','Source','A manual','Manual author',0.5,3,1,'now','now','now','fa'),('b','b.pdf','b.pdf','b.pdf','B',NULL,NULL,0.25,3,1,'now','now','now','fb'); WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<180) INSERT INTO annotations(id,document_id,page_index,type,color,note,document_fingerprint,created_at,updated_at) SELECT printf('review-%03d',x),CASE WHEN x<2 THEN 'a' ELSE 'b' END,x%3,'pageComment','violet',CASE WHEN x<2 THEN 'Unique passage' ELSE 'Other' END,CASE WHEN x<2 THEN 'fa' ELSE 'fb' END,'2026-10-10T12:00:00Z','2026-10-10T12:00:00Z' FROM n;").unwrap();
    let (runtime, native, _) = start_database(db);
    runtime
        .submit(Action::GetDocument("a".into(), Cancellation::default()))
        .unwrap();
    let Update::Document(_, Some(document), _) = update(&runtime) else {
        panic!("Expected document")
    };
    runtime
        .submit(Action::Open(document, None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, held) = next(&native) else {
        panic!("Expected held PDF open")
    };
    let governor = MemoryGovernor::default();
    for (name, format, offset, text, count) in [
        ("review.json", AnnotationExportFormat::Json, 0, "", 80),
        ("last.json", AnnotationExportFormat::Json, 160, "", 21),
        (
            "filtered.json",
            AnnotationExportFormat::Json,
            0,
            "Unique",
            2,
        ),
        ("review.md", AnnotationExportFormat::Markdown, 0, "", 80),
    ] {
        let destination = directory.path().join(name);
        let request = AnnotationExportRequest::review(
            AnnotationQuery {
                grouping: Some(AnnotationGrouping::Document),
                offset,
                text: text.into(),
                ..Default::default()
            },
            destination.clone(),
            format,
            governor.clone(),
        );
        let token = request.cancel.clone();
        runtime
            .submit(Action::AnnotationExport(Box::new(request)))
            .unwrap();
        let Update::AnnotationExport(reply) = update(&runtime) else {
            panic!("Expected review export")
        };
        assert_eq!(reply.scope, AnnotationExportScope::Review);
        assert!(reply.cancel.same_request(&token));
        assert_eq!(reply.result.unwrap(), count);
        let bytes = std::fs::read(&destination).unwrap();
        if matches!(format, AnnotationExportFormat::Json) {
            let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(value["format"], "yamantaka-global-annotations");
            let docs = value["documents"].as_array().unwrap();
            assert_eq!(
                docs.iter()
                    .map(|d| d["annotations"].as_array().unwrap().len() as u64)
                    .sum::<u64>(),
                count
            );
            if offset == 0 {
                assert_eq!(docs[0]["metadata"]["title"], "A manual");
                assert_eq!(docs[0]["metadata"]["author"], "Manual author");
            }
        } else {
            assert!(String::from_utf8(bytes).unwrap().contains("\n---\n\n# B"));
        }
        assert_eq!(governor.usage().total, 0);
        assert!(native.try_recv().is_err());
    }
    let occupied = directory.path().join("review.json");
    let original = std::fs::read(&occupied).unwrap();
    let request = AnnotationExportRequest::review(
        Default::default(),
        occupied.clone(),
        AnnotationExportFormat::Json,
        governor.clone(),
    );
    runtime
        .submit(Action::AnnotationExport(Box::new(request)))
        .unwrap();
    let Update::AnnotationExport(reply) = update(&runtime) else {
        panic!("Expected occupied destination")
    };
    assert_eq!(reply.scope, AnnotationExportScope::Review);
    assert!(matches!(reply.result, Err(Failure::DestinationOccupied)));
    assert_eq!(std::fs::read(occupied).unwrap(), original);
    for (name, denied, cancelled) in [
        ("denied.json", true, false),
        ("cancelled.json", false, true),
        ("wrong.md", false, false),
    ] {
        let destination = directory.path().join(name);
        let governor = if denied {
            MemoryGovernor::new(GlobalMemoryLimits {
                total: 1,
                ..Default::default()
            })
        } else {
            MemoryGovernor::default()
        };
        let request = AnnotationExportRequest::review(
            Default::default(),
            destination.clone(),
            AnnotationExportFormat::Json,
            governor.clone(),
        );
        if cancelled {
            request.cancel.cancel();
        }
        let token = request.cancel.clone();
        runtime
            .submit(Action::AnnotationExport(Box::new(request)))
            .unwrap();
        let Update::AnnotationExport(reply) = update(&runtime) else {
            panic!("Expected failed export")
        };
        assert_eq!(reply.scope, AnnotationExportScope::Review);
        assert!(reply.cancel.same_request(&token));
        if denied {
            assert!(matches!(reply.result, Err(Failure::MemoryBudget)));
        } else if cancelled {
            assert!(matches!(reply.result, Err(Failure::Cancelled)));
        } else {
            assert!(matches!(reply.result, Err(Failure::Invalid(_))));
        }
        assert!(!destination.exists());
        assert_eq!(governor.usage().total, 0);
    }
    assert!(native.try_recv().is_err());
    held.send(Err(Failure::Cancelled)).unwrap();
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}
