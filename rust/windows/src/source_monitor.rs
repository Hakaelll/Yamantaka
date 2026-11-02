//! Observe only explicitly supplied files; never enumerate or import a directory.
use crate::source_stamp::Stamp;
use std::{
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};
use yamantaka_domain::{Failure, Result};

const INTERVAL: Duration = Duration::from_secs(1);
const MAX_FILES: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub document: String,
    pub path: PathBuf,
    pub generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileState {
    Unavailable,
    Present(Stamp),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub target: Target,
    pub state: FileState,
}
#[derive(Default)]
struct Shared {
    revision: u64,
    targets: Arc<[Target]>,
    completion: Option<Vec<Observation>>,
    stopping: bool,
}

pub struct Monitor {
    shared: Arc<(Mutex<Shared>, Condvar)>,
    worker: Option<JoinHandle<()>>,
}
impl Monitor {
    pub fn start(notify: impl Fn() + Send + 'static) -> Result<Self> {
        let shared = Arc::new((Mutex::new(Shared::default()), Condvar::new()));
        let state = shared.clone();
        let worker = thread::Builder::new()
            .name("visible-source-observer".into())
            .spawn(move || observe(state, notify))
            .map_err(|_| Failure::Storage("Visible source observer could not start".into()))?;
        Ok(Self {
            shared,
            worker: Some(worker),
        })
    }
    pub fn set_targets(&self, targets: Vec<Target>) -> Result<()> {
        if targets.len() > MAX_FILES {
            return Err(Failure::Invalid(
                "Visible source observation is bounded to sixteen explicit files".into(),
            ));
        }
        let (state, wake) = &*self.shared;
        let mut state = state.lock().unwrap_or_else(|error| error.into_inner());
        if state.targets.as_ref() != targets.as_slice() {
            state.revision = state.revision.checked_add(1).ok_or_else(|| {
                Failure::Invalid("Source observation generation exhausted".into())
            })?;
            state.targets = targets.into();
            state.completion = None;
            wake.notify_one();
        }
        Ok(())
    }
    pub fn take(&self) -> Option<Vec<Observation>> {
        self.shared
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .completion
            .take()
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        let (state, wake) = &*self.shared;
        state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .stopping = true;
        wake.notify_one();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn observe(shared: Arc<(Mutex<Shared>, Condvar)>, notify: impl Fn()) {
    let (state, wake) = &*shared;
    let mut seen = 0;
    let mut previous = Vec::new();
    loop {
        let mut scope = state.lock().unwrap_or_else(|error| error.into_inner());
        if scope.stopping {
            return;
        }
        if scope.revision == seen {
            scope = if scope.targets.is_empty() {
                wake.wait(scope).unwrap_or_else(|error| error.into_inner())
            } else {
                wake.wait_timeout(scope, INTERVAL)
                    .unwrap_or_else(|error| error.into_inner())
                    .0
            };
        }
        if scope.stopping {
            return;
        }
        let revision = scope.revision;
        let targets = scope.targets.clone();
        drop(scope);
        let mut observed = Vec::with_capacity(targets.len());
        for target in targets.iter() {
            let scope = state.lock().unwrap_or_else(|error| error.into_inner());
            if scope.stopping {
                return;
            }
            if scope.revision != revision {
                break;
            }
            drop(scope);
            observed.push(Observation {
                target: target.clone(),
                state: Stamp::read(&target.path)
                    .map(FileState::Present)
                    .unwrap_or(FileState::Unavailable),
            });
        }
        let mut scope = state.lock().unwrap_or_else(|error| error.into_inner());
        if scope.stopping {
            return;
        }
        if scope.revision != revision {
            continue;
        }
        let changed = revision != seen || observed != previous;
        seen = revision;
        if changed {
            scope.completion = Some(observed.clone());
        }
        drop(scope);
        previous = observed;
        if changed {
            notify();
        }
    }
}
