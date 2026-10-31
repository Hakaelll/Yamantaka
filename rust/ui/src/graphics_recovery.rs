//! A failed wgpu startup gets one fresh-process attempt with the alternate compositor.
use std::{error::Error, ffi::OsString, process::Command};

pub(super) fn is_fallback() -> bool {
    std::env::args_os().any(|arg| arg == "--graphics-fallback")
}

pub(super) fn can_retry(error: &(dyn Error + 'static), args: &[OsString]) -> bool {
    !args
        .iter()
        .any(|arg| arg == "--software" || arg == "--opengl" || arg == "--graphics-fallback")
        && matches!(
            error.downcast_ref::<eframe::Error>(),
            Some(eframe::Error::Wgpu(_))
        )
}

pub(super) fn restart(args: &[OsString]) -> std::io::Result<std::process::ExitStatus> {
    // run() has returned: its instance lock, database and native worker are gone.
    Command::new(std::env::current_exe()?)
        .args(args)
        .args(["--software", "--graphics-fallback"])
        .status()
}
