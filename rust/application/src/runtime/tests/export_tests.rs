use super::*;
use crate::annotation_export::AnnotationExportFormat;
use yamantaka_domain::{annotations::repository::AnnotationRecord, memory::MemoryGovernor};

pub(super) struct Gate {
    started: Sender<()>,
    release: Receiver<()>,
    ended: Sender<()>,
}
pub(super) struct Reader(pub Arc<Gate>);
impl AnnotationRepository for Reader {
    fn visit_annotations(
        &self,
        _: &str,
        _: &MemoryGovernor,
        cancel: &Cancellation,
        _: &mut dyn FnMut(&AnnotationRecord) -> Result<()>,
    ) -> Result<u64> {
        self.0.started.send(()).unwrap();
        loop {
            cancel.check()?;
            match self.0.release.recv_timeout(Duration::from_millis(5)) {
                Ok(()) => return Ok(0),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return Err(Failure::Cancelled),
            }
        }
    }
    fn publish_annotation_export(
        &self,
        _: &std::path::Path,
        _: &Cancellation,
        write: &mut dyn FnMut(&mut dyn std::io::Write) -> Result<u64>,
    ) -> Result<u64> {
        write(&mut Vec::new())
    }
}
impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.0.ended.send(());
    }
}
fn request(cancel: Cancellation) -> Action {
    Action::AnnotationExport(Box::new(AnnotationExportRequest {
        source: crate::annotation_export::worker::AnnotationExportSource::Document {
            document: "study".into(),
            title: "Study".into(),
            author: None,
            progress: 0.5,
        },
        exported_at: "2026-10-10T12:00:00Z".into(),
        destination: PathBuf::from("synthetic.json"),
        format: AnnotationExportFormat::Json,
        governor: MemoryGovernor::default(),
        cancel,
    }))
}
fn setup() -> (Runtime, Receiver<()>, Sender<()>, Receiver<()>) {
    let (start, started) = bounded(1);
    let (release, held) = bounded(1);
    let (end, ended) = bounded(1);
    let repo = Repository {
        export_gate: Some(Arc::new(Gate {
            started: start,
            release: held,
            ended: end,
        })),
        ..Default::default()
    };
    (
        Runtime::start(repo, Engine, || {}).unwrap(),
        started,
        release,
        ended,
    )
}

#[test]
fn a_held_export_does_not_block_catalogue_commands_and_rejects_another_export() {
    let (runtime, started, release, ended) = setup();
    let first = Cancellation::default();
    runtime.submit(request(first.clone())).unwrap();
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    assert!(matches!(
        runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap(),
        Update::Library(_, 0, _)
    ));
    let second = Cancellation::default();
    runtime.submit(request(second.clone())).unwrap();
    let Update::AnnotationExport(reply) = runtime
        .updates
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
    else {
        panic!("Expected busy reply")
    };
    assert!(reply.cancel.same_request(&second));
    assert!(matches!(reply.result, Err(Failure::QueueFull)));
    release.send(()).unwrap();
    let Update::AnnotationExport(reply) = runtime
        .updates
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
    else {
        panic!("Expected completion")
    };
    assert_eq!(
        reply.scope,
        crate::annotation_export::worker::AnnotationExportScope::Document("study".into())
    );
    assert_eq!(reply.result.unwrap(), 0);
    assert!(reply.cancel.same_request(&first));
    assert!(
        !first.is_cancelled(),
        "Joining a successful job must preserve its token"
    );
    ended.recv_timeout(Duration::from_secs(5)).unwrap();
}

#[test]
fn runtime_shutdown_cancels_and_joins_a_held_export_without_ui_draining() {
    let (runtime, started, _release, ended) = setup();
    let cancel = Cancellation::default();
    runtime.submit(request(cancel.clone())).unwrap();
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(runtime);
    assert!(cancel.is_cancelled());
    ended
        .try_recv()
        .expect("Runtime drop must join and release the export reader");
}

#[test]
fn user_cancellation_returns_a_scoped_terminal_reply_and_releases_the_export_slot() {
    let (runtime, started, release, ended) = setup();
    let cancel = Cancellation::default();
    runtime.submit(request(cancel.clone())).unwrap();
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    cancel.cancel();
    let Update::AnnotationExport(reply) = runtime
        .updates
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
    else {
        panic!("Expected cancellation")
    };
    assert!(reply.cancel.same_request(&cancel));
    assert!(matches!(reply.result, Err(Failure::Cancelled)));
    ended.recv_timeout(Duration::from_secs(5)).unwrap();
    runtime.submit(request(Cancellation::default())).unwrap();
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    release.send(()).unwrap();
    assert!(matches!(
        runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap(),
        Update::AnnotationExport(AnnotationExportReply { result: Ok(0), .. })
    ));
}
