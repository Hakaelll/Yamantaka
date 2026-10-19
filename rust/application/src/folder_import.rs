//! Pull-based traversal keeps folder imports bounded independently of directory size.
use crossbeam_channel::{Receiver, Select, Sender, bounded};
use std::{
    fs::{self, ReadDir},
    path::PathBuf,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{Cancellation, Failure, Result};

fn inaccessible(_: std::io::Error) -> Failure {
    Failure::Storage("Could not read the selected folder; check access and retry".into())
}
struct Walker {
    root: PathBuf,
    stack: Vec<ReadDir>,
}
impl Walker {
    fn open(root: PathBuf) -> Result<Self> {
        let root = root.canonicalize().map_err(inaccessible)?;
        let entries = fs::read_dir(&root).map_err(inaccessible)?;
        Ok(Self {
            root,
            stack: vec![entries],
        })
    }
    fn next(&mut self, cancel: &Cancellation, stop: &Cancellation) -> Result<Option<PathBuf>> {
        loop {
            cancel.check()?;
            stop.check()?;
            let Some(entries) = self.stack.last_mut() else {
                return Ok(None);
            };
            let Some(entry) = entries.next() else {
                self.stack.pop();
                continue;
            };
            let entry = entry.map_err(inaccessible)?;
            let kind = entry.file_type().map_err(inaccessible)?;
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    // A junction can escape the authorized tree or form a cycle.
                    if entry.metadata().map_err(inaccessible)?.file_attributes() & 0x400 != 0 {
                        continue;
                    }
                }
                if self.stack.len() >= 256 {
                    return Err(Failure::Invalid(
                        "Folder nesting exceeds the supported depth of 256".into(),
                    ));
                }
                let path = entry.path().canonicalize().map_err(inaccessible)?;
                if !path.starts_with(&self.root) {
                    return Err(Failure::Invalid("Folder changed during import".into()));
                }
                self.stack.push(fs::read_dir(path).map_err(inaccessible)?);
            } else if kind.is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.eq_ignore_ascii_case("pdf"))
            {
                let path = entry.path().canonicalize().map_err(inaccessible)?;
                if !path.starts_with(&self.root) {
                    return Err(Failure::Invalid("Folder changed during import".into()));
                }
                return Ok(Some(path));
            }
        }
    }
}
pub(crate) struct FolderImport {
    requests: Option<Sender<()>>,
    replies: Receiver<Result<Option<PathBuf>>>,
    stop: Cancellation,
    handle: Option<JoinHandle<()>>,
}
impl FolderImport {
    pub(crate) fn start(root: PathBuf, cancel: Cancellation) -> Result<Self> {
        cancel.check()?;
        let (request, demands) = bounded(1);
        let (send, replies) = bounded(1);
        let stop = Cancellation::default();
        let worker_stop = stop.clone();
        let handle = thread::Builder::new()
            .name("yamantaka-folder-import".into())
            .spawn(move || {
                let mut walker = Walker::open(root);
                while demands.recv().is_ok() {
                    let reply = match &mut walker {
                        Ok(walker) => walker.next(&cancel, &worker_stop),
                        Err(_) => Err(Failure::Storage(
                            "Could not open the selected folder".into(),
                        )),
                    };
                    let finished = !matches!(reply, Ok(Some(_)));
                    if send.send(reply).is_err() || finished {
                        break;
                    }
                }
            })
            .map_err(|_| Failure::Storage("Folder import worker could not start".into()))?;
        Ok(Self {
            requests: Some(request),
            replies,
            stop,
            handle: Some(handle),
        })
    }
    pub(crate) fn request_next(&self) -> Result<()> {
        self.requests
            .as_ref()
            .ok_or(Failure::Cancelled)?
            .try_send(())
            .map_err(|_| Failure::Storage("Folder import worker is unavailable".into()))
    }
    pub(crate) fn ready(&self) -> bool {
        !self.replies.is_empty()
            || self
                .handle
                .as_ref()
                .is_some_and(|handle| handle.is_finished())
    }
    pub(crate) fn receive(&self) -> Result<Option<PathBuf>> {
        self.replies
            .try_recv()
            .map_err(|_| Failure::Storage("Folder import worker stopped".into()))?
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
}
impl Drop for FolderImport {
    fn drop(&mut self) {
        self.stop.cancel();
        self.requests.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
#[path = "folder_import/recovery_test.rs"]
mod recovery_test;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{Action, Runtime, Update};
    use std::time::Duration;
    use yamantaka_domain::LibraryQuery;
    use yamantaka_pdf::{MemoryLimits, PdfWorker};
    use yamantaka_storage::{Database, identity::full_sha256};
    pub(super) fn folder_result(runtime: &Runtime) -> Result<usize> {
        loop {
            match runtime
                .updates
                .recv_timeout(Duration::from_secs(15))
                .unwrap()
            {
                Update::Imported(count, cancel) => {
                    assert!(!cancel.is_cancelled());
                    return Ok(count);
                }
                Update::Failed(error, _) => return Err(error),
                _ => {}
            }
        }
    }
    pub(super) fn documents(runtime: &Runtime) -> Vec<String> {
        let cancel = Cancellation::default();
        runtime
            .submit(Action::Query(LibraryQuery::default(), cancel.clone()))
            .unwrap();
        loop {
            if let Update::Library(rows, count, token) = runtime
                .updates
                .recv_timeout(Duration::from_secs(15))
                .unwrap()
                && token.same_request(&cancel)
            {
                assert_eq!(count, 3);
                assert_eq!(rows.len(), 3);
                let mut ids = rows
                    .into_iter()
                    .map(|document| {
                        assert!(!document.display_title().is_empty());
                        document.id
                    })
                    .collect::<Vec<_>>();
                ids.sort();
                return ids;
            }
        }
    }
    #[test]
    fn folder_import_streams_nested_pdfs_retries_without_duplicates_and_releases_owned_work() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let temp = tempfile::tempdir().unwrap();
        let selected = temp.path().join("Biblioteca 日本語");
        fs::create_dir_all(selected.join("nested/looks-like.pdf/empty")).unwrap();
        let paths = [
            "first.pdf",
            "nested/segundo.PDF",
            "nested/looks-like.pdf/third.pdf",
        ]
        .map(|name| selected.join(name));
        for path in &paths {
            fs::copy(root.join("build/rust-fixtures/textual.pdf"), path).unwrap();
        }
        fs::copy(&paths[0], temp.path().join("outside.pdf")).unwrap();
        fs::write(selected.join("ignored.txt"), b"not a PDF").unwrap();
        let originals = paths
            .iter()
            .map(|path| full_sha256(path, &Cancellation::default()).unwrap())
            .collect::<Vec<_>>();
        // One demand returns one path. A cancelled scanner never supplies another.
        let cancel = Cancellation::default();
        let scanner = FolderImport::start(selected.clone(), cancel.clone()).unwrap();
        scanner.request_next().unwrap();
        assert!(
            scanner
                .replies
                .recv_timeout(Duration::from_secs(5))
                .unwrap()
                .unwrap()
                .is_some()
        );
        assert!(scanner.replies.is_empty());
        cancel.cancel();
        scanner.request_next().unwrap();
        assert!(matches!(
            scanner
                .replies
                .recv_timeout(Duration::from_secs(5))
                .unwrap(),
            Err(Failure::Cancelled)
        ));
        drop(scanner);
        let db = Database::in_memory_at(11).unwrap();
        let engine = PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap();
        let governor = engine.memory_governor();
        let runtime = Runtime::start(db, engine, || {}).unwrap();
        let mut previous_ids = None;
        for _ in 0..2 {
            runtime
                .submit(Action::ImportFolder(
                    selected.clone(),
                    Cancellation::default(),
                ))
                .unwrap();
            assert_eq!(folder_result(&runtime).unwrap(), 3);
            let ids = documents(&runtime);
            if let Some(previous) = &previous_ids {
                assert_eq!(&ids, previous);
            } else {
                previous_ids = Some(ids);
            }
        }
        runtime
            .submit(Action::ImportFolder(
                selected.join("nested/looks-like.pdf/empty"),
                Cancellation::default(),
            ))
            .unwrap();
        assert_eq!(folder_result(&runtime).unwrap(), 0);
        let cancelled = Cancellation::default();
        cancelled.cancel();
        runtime
            .submit(Action::ImportFolder(selected.clone(), cancelled))
            .unwrap();
        assert!(matches!(folder_result(&runtime), Err(Failure::Cancelled)));
        runtime.submit(Action::PendingImports).unwrap();
        let paused = loop {
            if let Update::PendingImports(result) = runtime
                .updates
                .recv_timeout(Duration::from_secs(10))
                .unwrap()
            {
                break result.unwrap();
            }
        };
        assert_eq!(paused.len(), 1);
        assert_eq!(paused[0].completed, 0);
        runtime
            .submit(Action::DiscardImport(paused[0].id.clone()))
            .unwrap();
        loop {
            if let Update::PendingImports(result) = runtime
                .updates
                .recv_timeout(Duration::from_secs(10))
                .unwrap()
            {
                assert!(result.unwrap().is_empty());
                break;
            }
        }

        runtime
            .submit(Action::ImportFolder(
                temp.path().join("does-not-exist"),
                Cancellation::default(),
            ))
            .unwrap();
        assert!(matches!(folder_result(&runtime), Err(Failure::Storage(_))));
        assert_eq!(documents(&runtime).len(), 3);
        drop(runtime);
        assert_eq!(governor.usage().total, 0);
        crate::import_hash::verification_test::verify(&root);
        super::recovery_test::verify(&root, &paths);
        for (path, original) in paths.iter().zip(originals) {
            assert_eq!(
                full_sha256(path, &Cancellation::default()).unwrap(),
                original
            );
        }
    }
}
