use crate::file_digest::HashedFile;
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    collections::BTreeMap,
    sync::Arc,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Failure, MAX_LIBRARY_PAGE, Result,
    duplicates::*,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation, TrackedBytesBuilder},
    source_revision::FullSourceHash,
};

pub struct Request {
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
    pub watch: Arc<Watch>,
}
pub struct Reply {
    pub cancel: Cancellation,
    pub result: Result<Arc<Report>>,
}
fn discover(request: &Request, reader: &dyn Reader) -> Result<Arc<Report>> {
    let mut progress = Progress::default();
    let mut buffer =
        TrackedBytesBuilder::try_zeroed(&request.governor, MemoryCategory::Other, 65536)
            .map_err(|_| Failure::MemoryBudget)?;
    let mut groups: BTreeMap<(u64, [u8; 32]), Vec<VerifiedSource>> = BTreeMap::new();
    let mut after: Option<Arc<Source>> = None;
    loop {
        request.cancel.check()?;
        let page = reader.page(
            after.as_ref().map(|source| source.document.as_str()),
            &request.governor,
            &request.cancel,
        )?;
        if page.len() > MAX_LIBRARY_PAGE
            || page
                .windows(2)
                .any(|pair| pair[0].document >= pair[1].document)
            || page.first().is_some_and(|first| {
                after
                    .as_ref()
                    .is_some_and(|last| first.document <= last.document)
            })
        {
            return Err(Failure::Invalid(
                "Duplicate candidates are outside their ordered bounds".into(),
            ));
        }
        if page.is_empty() {
            break;
        }
        for source in page {
            request.cancel.check()?;
            if progress.checked >= 100000 {
                return Err(Failure::Invalid(
                    "Duplicate discovery exceeds 100000 candidates".into(),
                ));
            }
            progress.checked += 1;
            let file = if source.file.missing {
                None
            } else {
                match HashedFile::read(&source.file.path, buffer.as_mut_slice(), &request.cancel) {
                    Ok(file)
                        if file.size() == source.file.size
                            && file.fingerprint() == source.file.fingerprint
                            && file.modified_matches(&source.file.modified) == Some(true) =>
                    {
                        Some(file)
                    }
                    Err(Failure::Cancelled) => return Err(Failure::Cancelled),
                    _ => None,
                }
            };
            if let Some(file) = file {
                let revision = file.source_revision();
                let bytes = revision
                    .canonical_path
                    .as_os_str()
                    .as_encoded_bytes()
                    .len()
                    .checked_mul(2)
                    .and_then(|size| size.checked_add(512))
                    .ok_or(Failure::MemoryBudget)?;
                let lease = request
                    .governor
                    .try_reserve_one(Cost::new(
                        MemoryCategory::Other,
                        MemoryRepresentation::Cpu,
                        bytes,
                    ))
                    .map_err(|_| Failure::MemoryBudget)?;
                let proof = FullSourceHash {
                    source: revision,
                    digest: file.digest(),
                };
                let group = groups.entry((file.size(), file.digest())).or_default();
                progress.matched += match group.len() {
                    0 => 0,
                    1 => 2,
                    _ => 1,
                };
                group.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
                group.push(VerifiedSource::new(source.clone(), proof, lease));
            } else {
                progress.unverified += 1;
            }
            after = Some(source);
            request.watch.publish(progress);
        }
    }
    request.cancel.check()?;
    let mut output = Vec::new();
    for ((size, digest), sources) in groups {
        if sources.len() > 1 {
            output.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
            output.push(Group {
                size,
                digest,
                sources,
            });
        }
    }
    Ok(Arc::new(Report {
        groups: output,
        progress,
    }))
}
pub(crate) struct Job {
    cancel: Cancellation,
    replies: Receiver<Reply>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl Job {
    pub(crate) fn start(request: Request, factory: ReaderFactory) -> Result<Self> {
        request.cancel.check()?;
        let cancel = request.cancel.clone();
        let (send, replies) = bounded(1);
        let handle = thread::Builder::new()
            .name("yamantaka-duplicates".into())
            .spawn(move || {
                let result = factory().and_then(|reader| discover(&request, &*reader));
                let _ = send.send(Reply {
                    cancel: request.cancel,
                    result,
                });
            })
            .map_err(|_| Failure::Storage("Duplicate worker could not start".into()))?;
        Ok(Self {
            cancel,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn poll(&mut self) -> Option<Reply> {
        let reply = match self.replies.try_recv() {
            Ok(reply) => reply,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Reply {
                cancel: self.cancel.clone(),
                result: Err(Failure::Storage("Duplicate worker stopped".into())),
            },
        };
        self.completed = true;
        Some(reply)
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        if !self.completed {
            self.cancel.cancel();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
#[cfg(test)]
#[path = "duplicates/verification_test.rs"]
pub(crate) mod verification_test;
