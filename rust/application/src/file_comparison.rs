//! Read-only, cancellable whole-file comparison outside the repository actor.
use crate::file_digest::HashedFile;
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, Result,
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
};

pub struct ComparisonRequest {
    pub document: Document,
    pub candidate: PathBuf,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct ComparisonReply {
    pub document: Document,
    pub candidate: PathBuf,
    pub result: Result<bool>,
    pub cancel: Cancellation,
}
impl ComparisonRequest {
    fn validate(&self, repository: &impl LibraryRepository) -> Result<()> {
        self.cancel.check()?;
        let current = repository.get(&self.document.id)?.ok_or_else(changed)?;
        if current.missing
            || current.path != self.document.path
            || current.fingerprint != self.document.fingerprint
            || current.page_count != self.document.page_count
        {
            return Err(changed());
        }
        Ok(())
    }
}
fn changed() -> Failure {
    Failure::Invalid("Comparison source changed; refresh and compare again".into())
}

struct Proof {
    left: HashedFile,
    right: HashedFile,
}
impl Proof {
    fn compute(request: &ComparisonRequest) -> Result<Self> {
        let mut buffer =
            TrackedBytesBuilder::try_zeroed(&request.governor, MemoryCategory::Other, 65536)
                .map_err(|_| Failure::MemoryBudget)?;
        let left = HashedFile::read(
            &request.document.path,
            buffer.as_mut_slice(),
            &request.cancel,
        )?;
        if left.fingerprint() != request.document.fingerprint {
            return Err(changed());
        }
        let right = HashedFile::read(&request.candidate, buffer.as_mut_slice(), &request.cancel)?;
        left.verify()?;
        request.cancel.check()?;
        Ok(Self { left, right })
    }
    fn equal(&self) -> Result<bool> {
        self.left.verify()?;
        self.right.verify()?;
        Ok(self.left.size() == self.right.size() && self.left.digest() == self.right.digest())
    }
}
pub(crate) struct ComparisonJob {
    request: ComparisonRequest,
    replies: Receiver<Result<Proof>>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl ComparisonJob {
    pub(crate) fn start(
        request: ComparisonRequest,
        repository: &impl LibraryRepository,
    ) -> Result<Self> {
        request.validate(repository)?;
        let (send, replies) = bounded(1);
        let worker = ComparisonRequest {
            document: request.document.clone(),
            candidate: request.candidate.clone(),
            governor: request.governor.clone(),
            cancel: request.cancel.clone(),
        };
        let handle = thread::Builder::new()
            .name("yamantaka-file-comparison".into())
            .spawn(move || {
                let _ = send.send(Proof::compute(&worker));
            })
            .map_err(|_| Failure::Storage("File comparison worker could not start".into()))?;
        Ok(Self {
            request,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn poll(&mut self, repository: &impl LibraryRepository) -> Option<ComparisonReply> {
        let proof = match self.replies.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(Failure::Storage("File comparison worker stopped".into()))
            }
        };
        self.completed = true;
        let result = self
            .request
            .validate(repository)
            .and_then(|()| proof.and_then(|proof| proof.equal()));
        Some(ComparisonReply {
            document: self.request.document.clone(),
            candidate: self.request.candidate.clone(),
            result,
            cancel: self.request.cancel.clone(),
        })
    }
}
impl Drop for ComparisonJob {
    fn drop(&mut self) {
        if !self.completed {
            self.request.cancel.cancel();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use yamantaka_storage::{Database, identity};
    fn setup() -> (tempfile::TempDir, Database, Document) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("original.pdf");
        let mut bytes = vec![b'a'; 65536];
        bytes.extend_from_slice(b"original tail");
        std::fs::write(&path, &bytes).unwrap();
        let db = Database::open(&dir.path().join("library.sqlite3")).unwrap();
        let escaped = path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .replace('\'', "''");
        let fingerprint = identity::legacy_fingerprint(&path).unwrap();
        let modified =
            chrono::DateTime::<chrono::Utc>::from(path.metadata().unwrap().modified().unwrap())
                .to_rfc3339();
        db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('book','{escaped}','book','original.pdf','A book',2,65549,'{modified}','2026-10-18','2026-10-18','{fingerprint}')")).unwrap();
        let document = db.get("book").unwrap().unwrap();
        (dir, db, document)
    }
    fn request(
        document: &Document,
        candidate: &Path,
        governor: &MemoryGovernor,
    ) -> ComparisonRequest {
        ComparisonRequest {
            document: document.clone(),
            candidate: candidate.to_owned(),
            governor: governor.clone(),
            cancel: Cancellation::default(),
        }
    }
    fn complete(job: &mut ComparisonJob, db: &Database) -> ComparisonReply {
        let mut select = Select::new();
        job.register(&mut select);
        select
            .ready_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        job.poll(db).unwrap()
    }
    #[test]
    fn background_comparison_rejects_partial_hash_collision_and_confirms_full_copy() {
        if crate::automation_execution::legacy_recovery_verification::child_entry() {
            return;
        }
        let (dir, mut db, document) = setup();
        let candidate = dir.path().join("candidate.pdf");
        let governor = MemoryGovernor::new(Default::default());
        let mut bytes = std::fs::read(&document.path).unwrap();
        *bytes.last_mut().unwrap() = b'!';
        std::fs::write(&candidate, &bytes).unwrap();
        assert_eq!(
            identity::legacy_fingerprint(&candidate).unwrap(),
            document.fingerprint
        );
        let mut job = ComparisonJob::start(request(&document, &candidate, &governor), &db).unwrap();
        let reply = complete(&mut job, &db);
        assert!(!reply.result.unwrap());
        assert!(!reply.cancel.is_cancelled());
        drop(job);
        assert_eq!(governor.usage().total, 0);
        std::fs::copy(&document.path, &candidate).unwrap();
        let mut job = ComparisonJob::start(request(&document, &candidate, &governor), &db).unwrap();
        let reply = complete(&mut job, &db);
        assert!(reply.result.unwrap());
        drop(job);
        assert_eq!(governor.usage().total, 0);
        assert_eq!(
            std::fs::read(&document.path).unwrap(),
            std::fs::read(&candidate).unwrap()
        );
        crate::file_check::verification_test::verify(&mut db, &document, dir.path(), &governor);
    }
    #[test]
    fn cancellation_and_catalogue_replacement_cannot_publish_exact_confirmation() {
        let (dir, db, document) = setup();
        let candidate = dir.path().join("copy.pdf");
        std::fs::copy(&document.path, &candidate).unwrap();
        let governor = MemoryGovernor::new(Default::default());
        let cancelled = request(&document, &candidate, &governor);
        cancelled.cancel.cancel();
        assert!(matches!(
            ComparisonJob::start(cancelled, &db),
            Err(Failure::Cancelled)
        ));
        let mut job = ComparisonJob::start(request(&document, &candidate, &governor), &db).unwrap();
        db.execute_batch("UPDATE documents SET fingerprint='replacement' WHERE id='book'")
            .unwrap();
        assert!(matches!(
            complete(&mut job, &db).result,
            Err(Failure::Invalid(_))
        ));
        drop(job);
        assert_eq!(governor.usage().total, 0);
        // Changed physical sources are rejected even when the catalogue stayed stale.
        let mut bytes = std::fs::read(&document.path).unwrap();
        bytes[0] = b'x';
        std::fs::write(&document.path, bytes).unwrap();
        db.execute_batch(&format!(
            "UPDATE documents SET fingerprint='{}' WHERE id='book'",
            document.fingerprint
        ))
        .unwrap();
        let mut job = ComparisonJob::start(request(&document, &candidate, &governor), &db).unwrap();
        assert!(matches!(
            complete(&mut job, &db).result,
            Err(Failure::Invalid(_))
        ));
        drop(job);
        assert_eq!(governor.usage().total, 0);
    }
}
