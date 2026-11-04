//! Session-only authorization. Notifications never enumerate, import or delete files.
use std::{
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
        Storage::FileSystem::{
            FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_FILE_NAME,
            FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE, FindCloseChangeNotification,
            FindFirstChangeNotificationW, FindNextChangeNotification,
        },
        System::Threading::{CreateEventW, INFINITE, SetEvent, WaitForMultipleObjects},
    },
    core::HSTRING,
};
use yamantaka_domain::{Failure, Result};

const DEBOUNCE: Duration = Duration::from_millis(350);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Starting,
    Active,
    Unavailable,
}

#[derive(Default)]
struct Shared {
    status: AtomicU8,
    changed: AtomicBool,
    paused: AtomicBool,
}

pub struct Watch {
    root: PathBuf,
    stop: Arc<OwnedHandle>,
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
}
impl Watch {
    pub fn start(root: PathBuf, notify: impl Fn() + Send + 'static) -> Result<Self> {
        if !root.is_absolute() || root.as_os_str().is_empty() {
            return Err(Failure::Invalid("Choose an absolute folder path".into()));
        }
        // The unnamed manual-reset event belongs to OwnedHandle and outlives the worker.
        let stop = unsafe {
            let event = CreateEventW(None, true, false, None)
                .map_err(|_| Failure::Storage("Folder observation could not start".into()))?;
            Arc::new(OwnedHandle::from_raw_handle(event.0))
        };
        let shared = Arc::new(Shared::default());
        let state = shared.clone();
        let stopping = stop.clone();
        let path = root.clone();
        let worker = thread::Builder::new()
            .name("authorized-folder-observer".into())
            .spawn(move || {
                let result = observe(&path, &stopping, &state, &notify);
                if result.is_err() {
                    state.status.store(2, Ordering::Release);
                    notify();
                }
            })
            .map_err(|_| Failure::Storage("Folder observation could not start".into()))?;
        Ok(Self {
            root,
            stop,
            shared,
            worker: Some(worker),
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn status(&self) -> Status {
        match self.shared.status.load(Ordering::Acquire) {
            0 => Status::Starting,
            1 => Status::Active,
            _ => Status::Unavailable,
        }
    }
    pub fn set_paused(&self, paused: bool) {
        self.shared.paused.store(paused, Ordering::Release);
        self.shared.changed.store(false, Ordering::Release);
    }
    pub fn take_changed(&self) -> bool {
        self.shared.changed.swap(false, Ordering::AcqRel)
            && !self.shared.paused.load(Ordering::Acquire)
    }
}
impl Drop for Watch {
    fn drop(&mut self) {
        // Signal the still-owned event before joining; no timed polling at idle.
        unsafe {
            let _ = SetEvent(HANDLE(self.stop.as_raw_handle()));
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct Notification(HANDLE);
impl Drop for Notification {
    fn drop(&mut self) {
        // Change-notification handles require their own close API, not CloseHandle.
        unsafe {
            let _ = FindCloseChangeNotification(self.0);
        }
    }
}
fn observe(
    root: &Path,
    stop: &OwnedHandle,
    state: &Shared,
    notify: &impl Fn(),
) -> std::result::Result<(), ()> {
    // Canonicalization stays off the UI thread and authorizes the selected target only.
    let root = root.canonicalize().map_err(|_| ())?;
    if !root.is_dir() {
        return Err(());
    }
    let path = HSTRING::from(root.as_os_str());
    // The path lives through creation; the worker alone owns, waits and rearms this handle.
    let changes = Notification(unsafe {
        FindFirstChangeNotificationW(
            &path,
            true,
            FILE_NOTIFY_CHANGE_FILE_NAME
                | FILE_NOTIFY_CHANGE_DIR_NAME
                | FILE_NOTIFY_CHANGE_SIZE
                | FILE_NOTIFY_CHANGE_LAST_WRITE,
        )
        .map_err(|_| ())?
    });
    state.status.store(1, Ordering::Release);
    notify();
    let handles = [HANDLE(stop.as_raw_handle()), changes.0];
    let mut pending: Option<Instant> = None;
    loop {
        let timeout = pending.map_or(INFINITE, |at| {
            DEBOUNCE
                .saturating_sub(at.elapsed())
                .as_millis()
                .min(u32::MAX as u128) as u32
        });
        // Both handles remain owned until after this synchronous wait returns.
        let event = unsafe { WaitForMultipleObjects(&handles, false, timeout) };
        if event == WAIT_OBJECT_0 {
            return Ok(());
        }
        if event.0 == WAIT_OBJECT_0.0 + 1 {
            // Rearm exactly once per signalled wait, including while paused.
            unsafe {
                FindNextChangeNotification(changes.0).map_err(|_| ())?;
            }
            pending = (!state.paused.load(Ordering::Acquire)).then(Instant::now);
        } else if event == WAIT_TIMEOUT {
            pending = None;
            if !state.paused.load(Ordering::Acquire) {
                state.changed.store(true, Ordering::Release);
                notify();
            }
        } else {
            return Err(());
        }
    }
}
