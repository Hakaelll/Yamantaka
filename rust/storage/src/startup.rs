pub mod repair;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use yamantaka_domain::{Failure, Result, startup::StartupMode};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    version: u32,
    clean: bool,
    failures: u32,
    token: String,
}
pub struct StartupRun {
    path: PathBuf,
    state: Option<State>,
    mode: StartupMode,
}
fn read(path: &Path) -> Result<Option<State>> {
    match path.symlink_metadata() {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Ok(metadata) if metadata.is_file() && metadata.len() <= 4096 => {}
        _ => {
            return Err(Failure::Invalid(
                "Recovery state unavailable; file preserved".into(),
            ));
        }
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| file.take(4097).read_to_end(&mut bytes))
        .map_err(|_| Failure::Storage("Recovery state unavailable".into()))?;
    if bytes.len() > 4096 {
        return Err(Failure::Invalid(
            "Recovery state exceeds its limit; file preserved".into(),
        ));
    }
    let state: State = serde_json::from_slice(&bytes)
        .map_err(|_| Failure::Invalid("Recovery state unsupported; file preserved".into()))?;
    if state.version != 1 || uuid::Uuid::parse_str(&state.token).is_err() {
        return Err(Failure::Invalid(
            "Recovery state unsupported; file preserved".into(),
        ));
    }
    Ok(Some(state))
}
impl StartupRun {
    pub fn begin(root: &Path, requested: bool) -> Self {
        let path = root.join("rust-startup-v1.json");
        let previous = match read(&path) {
            Ok(previous) => previous,
            Err(_) => {
                return Self {
                    path,
                    state: None,
                    mode: StartupMode::StateUnavailable,
                };
            }
        };
        let failures = previous
            .filter(|state| !state.clean)
            .map_or(0, |state| state.failures.saturating_add(1));
        let state = State {
            version: 1,
            clean: false,
            failures,
            token: uuid::Uuid::new_v4().to_string(),
        };
        let mode = if requested {
            StartupMode::Requested
        } else if failures >= 3 {
            StartupMode::RepeatedUnclean { failures }
        } else {
            StartupMode::Normal
        };
        if crate::preferences::atomic_json(&path, &state).is_err() {
            return Self {
                path,
                state: None,
                mode: StartupMode::StateUnavailable,
            };
        }
        Self {
            path,
            state: Some(state),
            mode,
        }
    }
    pub fn mode(&self) -> StartupMode {
        self.mode
    }
    pub fn finish(self) -> Result<()> {
        let Some(mut state) = self.state else {
            return Ok(());
        };
        let current = read(&self.path)?
            .ok_or_else(|| Failure::Invalid("Recovery state changed; file preserved".into()))?;
        if current.token != state.token || current.clean || current.failures != state.failures {
            return Err(Failure::Invalid(
                "Recovery state changed; file preserved".into(),
            ));
        }
        state.clean = true;
        state.failures = 0;
        crate::preferences::atomic_json(&self.path, &state)
    }
}
