use super::*;
use crate::{file_digest::HashedFile, import_hash::ImportHashJob};
use std::sync::Arc;
use yamantaka_domain::file_check::FileSnapshot;

pub(super) struct Hashing {
    pub(super) job: ImportHashJob,
    snapshot: FileSnapshot,
}
pub(super) struct Verified {
    guard: HashedFile,
    snapshot: FileSnapshot,
}
impl SearchIndexController {
    pub(super) fn prepare_source<R: LibraryRepository, E: AsyncPdfEngine>(
        &mut self,
        repository: &R,
        engine: &E,
        id: &str,
    ) -> Result<()> {
        let snapshot = repository.file_snapshot(id)?;
        let run = self.run.as_mut().unwrap();
        let governor = engine
            .memory_governor()
            .ok_or_else(|| Failure::Unsupported("Index source memory accounting".into()))?;
        let job = ImportHashJob::start(snapshot.file.path.clone(), governor, run.cancel.clone())?;
        self.progress.current_document = Some(id.into());
        self.progress.current_page = 0;
        self.progress.total_pages = snapshot.file.page_count;
        self.dirty = true;
        run.pending = Some(Pending::Hash(Box::new(Hashing { job, snapshot })));
        Ok(())
    }
    pub(super) fn finish_hash<R: LibraryRepository + SearchRepository>(
        &mut self,
        mut job: Hashing,
        repository: &mut R,
    ) -> Result<()> {
        let result = job.job.finish();
        if self.run.as_ref().unwrap().cancel.is_cancelled() {
            return self.finish_run(repository, true);
        }
        match result {
            Ok(guard) => {
                self.run.as_mut().unwrap().pending = Some(Pending::Verified(Box::new(Verified {
                    guard,
                    snapshot: job.snapshot,
                })))
            }
            Err(error) => self.record_failure(repository, &error)?,
        }
        Ok(())
    }
    pub(super) fn admit_source<R: LibraryRepository + SearchRepository, E: AsyncPdfEngine>(
        &mut self,
        source: Verified,
        repository: &mut R,
        engine: &E,
    ) -> Result<()> {
        let run = self.run.as_mut().unwrap();
        if run.cancel.is_cancelled() {
            return self.finish_run(repository, true);
        }
        let Verified { guard, snapshot } = source;
        let valid = guard.verify().and_then(|_| {
            if repository.file_snapshot(&snapshot.document)? != snapshot
                || guard.fingerprint() != snapshot.file.fingerprint
                || snapshot
                    .digest
                    .is_some_and(|digest| digest != guard.digest())
            {
                return Err(Failure::Invalid(
                    "Indexed PDF content changed; check the file first".into(),
                ));
            }
            Ok(())
        });
        if let Err(error) = valid {
            return self.record_failure(repository, &error);
        }
        let session = repository.begin_index(
            &snapshot,
            guard.digest(),
            true,
            IndexAccess::Enabled,
            &run.cancel,
        )?;
        let native_id = crate::temporary_document::reserve(repository, "index")?;
        self.progress.current_page = session.cursor;
        self.dirty = true;
        run.active = Some(Active {
            session,
            guard: Arc::new(guard),
            native_id,
            path: snapshot.file.path,
            attempted: false,
            yielded: false,
            info: None,
            source: None,
            cancel: Cancellation::default(),
        });
        self.open(engine, None)
    }
}
