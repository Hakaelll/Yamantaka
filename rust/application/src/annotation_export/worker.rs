use super::{AnnotationExportFormat, AnnotationExportMetadata, write_document_annotations};
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Failure, Result, annotations::repository::AnnotationRepository,
    memory::MemoryGovernor,
};

pub struct AnnotationExportRequest {
    pub document: String,
    pub title: String,
    pub author: Option<String>,
    pub progress: f64,
    pub exported_at: String,
    pub destination: PathBuf,
    pub format: AnnotationExportFormat,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct AnnotationExportReply {
    pub document: String,
    pub result: Result<u64>,
    pub cancel: Cancellation,
}
impl AnnotationExportRequest {
    fn execute(self, repository: &dyn AnnotationRepository) -> AnnotationExportReply {
        let result = self.cancel.check().and_then(|()| {
            repository.publish_annotation_export(&self.destination, &self.cancel, &mut |writer| {
                write_document_annotations(
                    repository,
                    writer,
                    self.format,
                    AnnotationExportMetadata {
                        document: &self.document,
                        title: &self.title,
                        author: self.author.as_deref(),
                        progress: self.progress,
                        exported_at: &self.exported_at,
                    },
                    &self.governor,
                    self.cancel.clone(),
                )
                .map(|(_, count)| count)
            })
        });
        AnnotationExportReply {
            document: self.document,
            result,
            cancel: self.cancel,
        }
    }
}

pub(crate) struct ExportJob {
    document: String,
    cancel: Cancellation,
    replies: Receiver<AnnotationExportReply>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl ExportJob {
    pub(crate) fn start(
        request: AnnotationExportRequest,
        reader: Box<dyn AnnotationRepository + Send>,
    ) -> Result<Self> {
        let document = request.document.clone();
        let cancel = request.cancel.clone();
        let (send, replies) = bounded(1);
        let handle = thread::Builder::new()
            .name("yamantaka-annotation-export".into())
            .spawn(move || {
                let _ = send.send(request.execute(&*reader));
            })
            .map_err(|_| Failure::Storage("Annotation export worker could not start".into()))?;
        Ok(Self {
            document,
            cancel,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn poll(&mut self) -> Option<AnnotationExportReply> {
        let reply = match self.replies.try_recv() {
            Ok(reply) => reply,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => AnnotationExportReply {
                document: self.document.clone(),
                result: Err(Failure::Storage("Annotation export worker stopped".into())),
                cancel: self.cancel.clone(),
            },
        };
        self.completed = true;
        Some(reply)
    }
}
impl Drop for ExportJob {
    fn drop(&mut self) {
        if !self.completed {
            self.cancel.cancel();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
