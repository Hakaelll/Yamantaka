use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use yamantaka_domain::{Cancellation, Failure, Result};
use yamantaka_storage::{Database, backup::BackupProgress, preferences::Preferences};

pub struct Request {
    pub database: PathBuf,
    pub protected_root: PathBuf,
    pub destination: PathBuf,
    pub preferences: Preferences,
    pub cancel: Cancellation,
    pub verification_cancel: bool,
}
pub struct Job {
    cancel: Cancellation,
    replies: Receiver<Result<PathBuf>>,
    progress: Arc<Mutex<Option<BackupProgress>>>,
    thread: Option<JoinHandle<()>>,
}
fn validate_destination(destination: &Path, root: &Path) -> Result<()> {
    if !destination
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        return Err(Failure::Invalid(
            "Choose a JSON backup outside the library folder".into(),
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| Failure::Invalid("Choose a backup folder".into()))?;
    let parent = parent
        .canonicalize()
        .map_err(|_| Failure::Storage("Backup folder is unavailable".into()))?;
    let root = root
        .canonicalize()
        .map_err(|_| Failure::Storage("Library folder is unavailable".into()))?;
    if parent.starts_with(root) {
        return Err(Failure::Invalid(
            "Choose a backup outside the library folder".into(),
        ));
    }
    match std::fs::symlink_metadata(destination) {
        Ok(metadata) => {
            use std::os::windows::fs::MetadataExt;
            if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
                return Err(Failure::Invalid("Choose an ordinary backup file".into()));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(Failure::Storage("Backup destination is unavailable".into())),
    }
    Ok(())
}
impl Job {
    pub fn start(request: Request, notify: impl Fn() + Send + 'static) -> Result<Self> {
        let cancel = request.cancel.clone();
        let (sender, replies) = mpsc::channel();
        let progress = Arc::new(Mutex::new(None));
        let status = progress.clone();
        let thread = thread::Builder::new()
            .name("library-backup".into())
            .spawn(move || {
                let result = (|| {
                    request.cancel.check()?;
                    validate_destination(&request.destination, &request.protected_root)?;
                    let database = Database::open_read_only(&request.database)?;
                    let mut last = Instant::now() - Duration::from_secs(1);
                    database.export_backup_progress(
                        &request.destination,
                        &request.preferences,
                        &request.cancel,
                        |progress| {
                            *status.lock().unwrap_or_else(|error| error.into_inner()) =
                                Some(progress);
                            if request.verification_cancel {
                                request.cancel.cancel();
                            }
                            if last.elapsed() >= Duration::from_millis(100) {
                                notify();
                                last = Instant::now();
                            }
                        },
                    )?;
                    Ok(request.destination)
                })();
                let _ = sender.send(result);
                notify();
            })
            .map_err(|_| Failure::Storage("Backup worker could not start".into()))?;
        Ok(Self {
            cancel,
            replies,
            progress,
            thread: Some(thread),
        })
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    pub fn progress(&self) -> Option<BackupProgress> {
        *self
            .progress
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
    pub fn poll(&mut self) -> Option<Result<PathBuf>> {
        let result = match self.replies.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                Some(Err(Failure::Storage("Backup worker stopped".into())))
            }
        };
        if result.is_some()
            && let Some(thread) = self.thread.take()
        {
            let _ = thread.join();
        }
        result
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
