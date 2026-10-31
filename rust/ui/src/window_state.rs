//! One writer owns window placement; application settings have a separate file.
use eframe::egui;
use raw_window_handle::HasWindowHandle;
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use yamantaka_domain::{Failure, Result};
use yamantaka_storage::preferences::Preferences;
use yamantaka_windows::window_placement::Placement;
pub mod preview;

pub struct State {
    sender: Option<SyncSender<Placement>>,
    worker: Option<JoinHandle<Result<()>>>,
    completion: Arc<Mutex<Option<Result<Placement>>>>,
    saved: Option<Placement>,
    reported: Option<Placement>,
    last: Option<Placement>,
    pending: Option<Placement>,
    verification: Option<preview::Cycle>,
}
impl State {
    pub fn open(
        store: &Path,
        cc: &eframe::CreationContext<'_>,
        normalize_initial: bool,
        backup_placement: Option<&serde_json::Value>,
    ) -> Option<Self> {
        let handle = cc.window_handle().ok()?;
        if normalize_initial {
            Placement::capture(handle).ok()?.restore(handle).ok()?;
        }
        let path = store.join("window-state-v1.json");
        // Preserve unknown or malformed state instead of silently replacing it.
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) => {
                use std::os::windows::fs::MetadataExt;
                if !metadata.is_file()
                    || metadata.len() > 65_536
                    || metadata.file_attributes() & 0x400 != 0
                {
                    return None;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return None,
        }
        let preferences = match std::fs::File::open(&path) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(65_537).read_to_end(&mut bytes).ok()?;
                if bytes.len() > 65_536 {
                    return None;
                }
                let preferences: Preferences = serde_json::from_slice(&bytes).ok()?;
                if preferences.version != 1 {
                    return None;
                }
                preferences
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Preferences::default(),
            Err(_) => return None,
        };
        let last = match preferences.rust.get("nativePlacement").or(backup_placement) {
            Some(value) => Some(Placement::decode(value)?),
            None => None,
        };
        if let Some(saved) = last {
            saved.restore(handle).ok()?;
        }
        let (sender, requests) = mpsc::sync_channel(1);
        let completion = Arc::new(Mutex::new(None));
        let replies = completion.clone();
        let notify = cc.egui_ctx.clone();
        let worker = thread::Builder::new()
            .name("window-placement".into())
            .spawn(move || {
                let result = persist(path, preferences, requests, |placement| {
                    *replies.lock().unwrap_or_else(|error| error.into_inner()) =
                        Some(Ok(placement));
                    notify.request_repaint();
                });
                if result.is_err() {
                    *replies.lock().unwrap_or_else(|error| error.into_inner()) = Some(Err(
                        Failure::Storage("Window placement could not be saved".into()),
                    ));
                    notify.request_repaint();
                }
                result
            })
            .ok()?;
        Some(Self {
            sender: Some(sender),
            worker: Some(worker),
            completion,
            saved: last,
            reported: None,
            last,
            pending: None,
            verification: None,
        })
    }
    pub fn observe(&mut self, ctx: &egui::Context, frame: &eframe::Frame) -> Result<()> {
        if let Some(result) = self
            .completion
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
        {
            match result {
                Ok(placement) => self.saved = Some(placement),
                Err(error) => {
                    self.sender = None;
                    return Err(error);
                }
            }
        }
        if self.sender.is_none()
            || ctx.input(|input| {
                input.viewport().fullscreen.unwrap_or(false)
                    || input.viewport().minimized.unwrap_or(false)
            })
        {
            return Ok(());
        }
        let Some(handle) = frame.window_handle().ok() else {
            return Ok(());
        };
        let placement = Placement::capture(handle)?;
        if self.last == Some(placement) {
            self.pending = None;
            return Ok(());
        }
        if self.last != Some(placement) {
            self.pending = Some(placement);
            if let Some(sender) = &self.sender {
                match sender.try_send(placement) {
                    Ok(()) => {
                        self.last = Some(placement);
                        self.pending = None;
                    }
                    Err(TrySendError::Full(_)) => {
                        ctx.request_repaint_after(Duration::from_millis(50))
                    }
                    Err(TrySendError::Disconnected(_)) => self.sender = None,
                }
            }
        }
        Ok(())
    }
    pub fn take_snapshot(&mut self) -> Option<serde_json::Value> {
        if self.last == self.reported {
            return None;
        }
        self.reported = self.last;
        self.last.map(Placement::encode)
    }
    pub fn finish(&mut self) -> Result<()> {
        if let Some(sender) = self.sender.take()
            && let Some(pending) = self.pending.take()
        {
            let _ = sender.send(pending);
        }
        if let Some(worker) = self.worker.take() {
            return worker
                .join()
                .map_err(|_| Failure::Storage("Window placement worker stopped".into()))?;
        }
        Ok(())
    }
}
impl Drop for State {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
fn persist(
    path: PathBuf,
    mut preferences: Preferences,
    requests: Receiver<Placement>,
    saved: impl Fn(Placement),
) -> Result<()> {
    while let Ok(mut placement) = requests.recv() {
        loop {
            match requests.recv_timeout(Duration::from_millis(500)) {
                Ok(next) => placement = next,
                Err(reason) => {
                    preferences
                        .rust
                        .insert("nativePlacement".into(), placement.encode());
                    preferences.save(&path)?;
                    saved(placement);
                    if reason == RecvTimeoutError::Disconnected {
                        return Ok(());
                    }
                    break;
                }
            }
        }
    }
    Ok(())
}
