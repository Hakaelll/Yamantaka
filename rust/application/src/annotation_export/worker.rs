use super::{
    AnnotationExportFormat, AnnotationExportMetadata, write_document_annotations,
    write_review_snapshot,
};
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    annotations::repository::{AnnotationQuery, AnnotationRepository},
    memory::MemoryGovernor,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AnnotationExportScope {
    Document(String),
    Review,
}
pub enum AnnotationExportSource {
    Document {
        document: String,
        title: String,
        author: Option<String>,
        progress: f64,
    },
    Review {
        query: AnnotationQuery,
    },
}
pub struct AnnotationExportRequest {
    pub source: AnnotationExportSource,
    pub exported_at: String,
    pub destination: PathBuf,
    pub format: AnnotationExportFormat,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct AnnotationExportReply {
    pub scope: AnnotationExportScope,
    pub result: Result<u64>,
    pub cancel: Cancellation,
}
impl AnnotationExportRequest {
    pub fn new(
        document: &yamantaka_domain::Document,
        progress: f64,
        destination: PathBuf,
        format: AnnotationExportFormat,
        governor: MemoryGovernor,
    ) -> Self {
        Self {
            source: AnnotationExportSource::Document {
                document: document.id.clone(),
                title: document.display_title().to_owned(),
                author: document.author.clone(),
                progress,
            },
            exported_at: chrono::Utc::now().to_rfc3339(),
            destination,
            format,
            governor,
            cancel: Cancellation::default(),
        }
    }
    pub fn review(
        query: AnnotationQuery,
        destination: PathBuf,
        format: AnnotationExportFormat,
        governor: MemoryGovernor,
    ) -> Self {
        Self {
            source: AnnotationExportSource::Review { query },
            destination,
            format,
            governor,
            exported_at: chrono::Utc::now().to_rfc3339(),
            cancel: Cancellation::default(),
        }
    }
    pub fn scope(&self) -> AnnotationExportScope {
        match &self.source {
            AnnotationExportSource::Document { document, .. } => {
                AnnotationExportScope::Document(document.clone())
            }
            AnnotationExportSource::Review { .. } => AnnotationExportScope::Review,
        }
    }
    fn execute(self, repository: &dyn AnnotationRepository) -> AnnotationExportReply {
        let scope = self.scope();
        let result = self.cancel.check().and_then(|()| {
            if !self
                .destination
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(self.format.extension()))
            {
                return Err(Failure::Invalid(
                    "Annotation export extension does not match its format".into(),
                ));
            }
            repository.publish_annotation_export(&self.destination, &self.cancel, &mut |writer| {
                match &self.source {
                    AnnotationExportSource::Document {
                        document,
                        title,
                        author,
                        progress,
                    } => write_document_annotations(
                        repository,
                        writer,
                        self.format,
                        AnnotationExportMetadata {
                            document,
                            title,
                            author: author.as_deref(),
                            progress: *progress,
                            exported_at: &self.exported_at,
                        },
                        &self.governor,
                        self.cancel.clone(),
                    )
                    .map(|(_, count)| count),
                    AnnotationExportSource::Review { query } => {
                        let snapshot = repository.annotation_review_snapshot(
                            query,
                            &self.governor,
                            &self.cancel,
                        )?;
                        write_review_snapshot(
                            writer,
                            self.format,
                            &snapshot,
                            &self.exported_at,
                            &self.governor,
                            self.cancel.clone(),
                        )
                        .map(|(_, count)| count)
                    }
                }
            })
        });
        AnnotationExportReply {
            scope,
            result,
            cancel: self.cancel,
        }
    }
}

pub(crate) struct ExportJob {
    scope: AnnotationExportScope,
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
        let scope = request.scope();
        let cancel = request.cancel.clone();
        let (send, replies) = bounded(1);
        let handle = thread::Builder::new()
            .name("yamantaka-annotation-export".into())
            .spawn(move || {
                let _ = send.send(request.execute(&*reader));
            })
            .map_err(|_| Failure::Storage("Annotation export worker could not start".into()))?;
        Ok(Self {
            scope,
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
                scope: self.scope.clone(),
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
