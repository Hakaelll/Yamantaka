//! Exercises visible-reader ownership using only generated fixture copies.
use std::{fs, io::Write, path::PathBuf, sync::Arc, time::Duration};
use yamantaka_application::{
    reader_close::ReaderCloseRequest,
    runtime::{Action, Runtime, Update},
};
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, Priority, RenderKey, RenderedPage,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;

fn next(runtime: &Runtime) -> Update {
    runtime
        .updates
        .recv_timeout(Duration::from_secs(10))
        .unwrap()
}

fn confirm(runtime: &Runtime, id: &str, cancel: Cancellation, keep: &[&str]) {
    runtime
        .submit(Action::ConfirmOpen {
            document: id.into(),
            visible: keep.iter().map(|id| id.to_string()).collect(),
            cancel: cancel.clone(),
        })
        .unwrap();
    assert!(matches!(next(runtime), Update::OpenConfirmed {
        document, cancel: actual, result: Ok(Some(ids))
    } if document == id && actual.same_request(&cancel)
        && ids == keep.iter().map(|id| id.to_string()).collect::<Vec<_>>()));
}

fn open(runtime: &Runtime, document: &Document, keep: &[&str]) {
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Open(document.clone(), None, cancel.clone()))
        .unwrap();
    assert!(matches!(next(runtime), Update::Opened(actual, _, token)
        if actual.id == document.id && token.same_request(&cancel)));
    confirm(runtime, &document.id, cancel, keep);
}

fn duplicate(runtime: &Runtime, document: &Document) -> bool {
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Open(document.clone(), None, cancel.clone()))
        .unwrap();
    match next(runtime) {
        Update::OpenFailed(actual, Failure::ReaderAlreadyOpen, token)
            if actual.id == document.id && token.same_request(&cancel) =>
        {
            true
        }
        Update::Opened(actual, _, token)
            if actual.id == document.id && token.same_request(&cancel) =>
        {
            confirm(runtime, &document.id, cancel, &["primary", "reference"]);
            false
        }
        _ => panic!("Unexpected result for an already visible reading"),
    }
}

fn text(runtime: &Runtime, id: &str) -> String {
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Text(id.into(), 0, cancel.clone()))
        .unwrap();
    match next(runtime) {
        Update::Text(actual, 0, text, token) if actual == id && token.same_request(&cancel) => text,
        _ => panic!("Expected text from the admitted reading"),
    }
}

fn close(runtime: &Runtime, id: &str) {
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Close(ReaderCloseRequest {
            document: id.into(),
            cancel: cancel.clone(),
        }))
        .unwrap();
    assert!(matches!(next(runtime), Update::ReaderClosed(reply)
        if reply.document == id && reply.cancel.same_request(&cancel) && reply.result.is_ok()));
}

fn render(runtime: &Runtime) -> (Arc<RenderedPage>, u64) {
    let cancel = Cancellation::default();
    let key = RenderKey {
        document_id: "primary".into(),
        page: 0,
        region: None,
        physical_width: 320,
        physical_height: 420,
        rotation: 0,
        generation: 1,
    };
    runtime
        .submit(Action::Render(
            key.clone(),
            Priority::Visible,
            cancel.clone(),
        ))
        .unwrap();
    match next(runtime) {
        Update::Page(page, token, _, epoch) if page.key == key && token.same_request(&cancel) => {
            (page, epoch)
        }
        _ => panic!("Expected the primary raster"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let report = PathBuf::from(args.first().ok_or("Expected a fresh JSON report path")?);
    if report
        .extension()
        .is_none_or(|extension| extension != "json")
    {
        return Err("Expected a fresh JSON report path".into());
    }
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&report)?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temporary = tempfile::tempdir_in(report.parent().ok_or("Expected a report directory")?)?;
    let primary = temporary.path().join("primary.pdf");
    let reference = temporary.path().join("reference.pdf");
    let original = fs::read(root.join("build/rust-fixtures/textual.pdf"))?;
    let replacement = fs::read(root.join("build/rust-fixtures/front-matter-conflict.pdf"))?;
    fs::write(&primary, &original)?;
    fs::write(&reference, &replacement)?;
    let database = Database::open(&temporary.path().join("library.sqlite3"))?;
    for (id, path) in [("primary", &primary), ("reference", &reference)] {
        let escaped = path.to_string_lossy().replace('\'', "''");
        database.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('{id}','{escaped}','{id}','fixture.pdf',2,1,'now','now','now','synthetic')"))?;
    }
    let document = database.get("primary")?.ok_or("Missing primary fixture")?;
    let other = database
        .get("reference")?
        .ok_or("Missing reference fixture")?;
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )?;
    let runtime = Runtime::start(database, worker, || {})?;
    open(&runtime, &document, &["primary"]);
    open(&runtime, &other, &["primary", "reference"]);
    let original_text = text(&runtime, "primary");
    let reference_text = text(&runtime, "reference");
    assert_ne!(original_text, reference_text);
    let (original_page, original_epoch) = render(&runtime);
    let unchanged_rejected = duplicate(&runtime, &document);
    assert_eq!(text(&runtime, "primary"), original_text);
    let (cached_page, retained_epoch) = render(&runtime);
    assert_eq!(original_epoch, retained_epoch);
    assert!(Arc::ptr_eq(&original_page, &cached_page));
    let cancel = Cancellation::default();
    cancel.cancel();
    runtime.submit(Action::Open(document.clone(), None, cancel.clone()))?;
    assert!(
        matches!(next(&runtime), Update::OpenFailed(actual, Failure::Cancelled, token)
        if actual.id == document.id && token.same_request(&cancel))
    );
    fs::write(&primary, &replacement)?;
    let cancel = Cancellation::default();
    runtime.submit(Action::Text("primary".into(), 0, cancel.clone()))?;
    assert!(
        matches!(next(&runtime), Update::Failed(Failure::PdfSourceChanged, Some(token))
        if token.same_request(&cancel))
    );
    let changed_rejected = duplicate(&runtime, &document);
    assert_eq!(text(&runtime, "reference"), reference_text);
    assert!(
        yamantaka_domain::quarantine::QuarantineReason::from_failure(&Failure::ReaderAlreadyOpen)
            .is_none()
    );
    close(&runtime, "primary");
    open(&runtime, &document, &["reference", "primary"]);
    assert_eq!(text(&runtime, "primary"), reference_text);
    assert_eq!(text(&runtime, "reference"), reference_text);
    close(&runtime, "primary");
    close(&runtime, "reference");
    drop(runtime);
    let result = serde_json::json!({
        "version":1,"synthetic":true,"unchangedVisibleOpenRejected":unchanged_rejected,
        "changedVisibleOpenRejected":changed_rejected,"oldTextRetainedBeforeEdit":true,
        "oldSourceReadRejectedAfterEdit":true,"referenceUnaffected":true,
        "nativeEpochRetained":true,"cachedRasterRetained":true,"cancellationPreserved":true,
        "ownershipRejectionExcludedFromQuarantine":true,
        "closedReaderReopenedWithNewText":true,"bothReadersClosed":true
    });
    output.write_all(serde_json::to_string_pretty(&result)?.as_bytes())?;
    output.sync_all()?;
    if !args.iter().any(|arg| arg == "--observe") {
        assert!(
            unchanged_rejected && changed_rejected,
            "Visible native ownership was replaced"
        );
    }
    println!("{result}");
    Ok(())
}
