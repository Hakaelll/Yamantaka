//! A failed wgpu startup gets one fresh-process attempt with the alternate compositor.
use std::{
    error::Error,
    ffi::OsString,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Default)]
struct Inner {
    fault: AtomicBool,
    closing: AtomicBool,
    closed: AtomicBool,
    checkpoint_failed: AtomicBool,
}

#[derive(Clone, Default)]
pub(super) struct State(Arc<Inner>);
impl State {
    pub fn install(&self, device: &eframe::wgpu::Device, ctx: &eframe::egui::Context) {
        let monitor = self.clone();
        let context = ctx.clone();
        device.set_device_lost_callback(move |reason, _| {
            if reason != eframe::wgpu::DeviceLostReason::Destroyed {
                monitor.signal(&context);
            }
        });
        let monitor = self.clone();
        let context = ctx.clone();
        device.on_uncaptured_error(Arc::new(move |_| monitor.signal(&context)));
    }
    pub fn signal(&self, ctx: &eframe::egui::Context) {
        if self.0.closing.load(Ordering::Acquire) {
            return;
        }
        if !self.0.fault.swap(true, Ordering::AcqRel) {
            ctx.request_repaint();
        }
    }
    pub fn faulted(&self) -> bool {
        self.0.fault.load(Ordering::Acquire)
    }
    pub fn checkpoint_failed(&self) {
        self.0.checkpoint_failed.store(true, Ordering::Release);
    }
    pub fn closed(&self) {
        self.0.closed.store(true, Ordering::Release);
    }
    pub fn begin_close(&self) {
        self.0.closing.store(true, Ordering::Release);
    }
    pub fn handover(&self) -> Result<(), Box<dyn Error>> {
        if self.0.closed.load(Ordering::Acquire)
            && !self.0.checkpoint_failed.load(Ordering::Acquire)
        {
            Ok(())
        } else {
            Err("Graphics recovery could not confirm a saved reading context; automatic restart stopped".into())
        }
    }
}

#[derive(Debug)]
pub(super) struct RuntimeFault {
    pub demo_store: Option<std::path::PathBuf>,
}
impl std::fmt::Display for RuntimeFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("The graphics device stopped during the reading session")
    }
}
impl Error for RuntimeFault {}

pub(super) fn runtime_fallback() -> bool {
    std::env::args_os().any(|arg| arg == "--graphics-runtime-fallback")
}

pub(super) fn continuity_resume() -> bool {
    runtime_fallback() || std::env::args_os().any(|arg| arg == "--continuity-resume")
}

pub(super) fn is_fallback() -> bool {
    std::env::args_os().any(|arg| arg == "--graphics-fallback")
}

pub(super) fn can_retry(error: &(dyn Error + 'static), args: &[OsString]) -> bool {
    !args
        .iter()
        .any(|arg| arg == "--software" || arg == "--opengl" || arg == "--graphics-fallback")
        && (error.is::<RuntimeFault>()
            || matches!(
                error.downcast_ref::<eframe::Error>(),
                Some(eframe::Error::Wgpu(_))
            ))
}

pub(super) fn restart(
    args: &[OsString],
    runtime: Option<&RuntimeFault>,
) -> std::io::Result<std::process::ExitStatus> {
    // run() has returned: its instance lock, database and native worker are gone.
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args(args)
        .args(["--software", "--graphics-fallback"]);
    if let Some(fault) = runtime {
        command.arg("--graphics-runtime-fallback");
        if let Some(store) = &fault.demo_store {
            let mut argument = OsString::from("--graphics-resume-demo=");
            argument.push(store);
            command.arg(argument);
        }
    }
    command.status()
}

pub(super) fn demo_store(args: &[OsString]) -> Result<Option<std::path::PathBuf>, Box<dyn Error>> {
    if !continuity_resume() {
        return Ok(None);
    }
    let Some(path) = args.iter().filter_map(|arg| arg.to_str()).find_map(|arg| {
        arg.strip_prefix("--graphics-resume-demo=")
            .or_else(|| arg.strip_prefix("--continuity-store="))
    }) else {
        return Ok(None);
    };
    let store = std::path::Path::new(path).canonicalize()?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build/demo-store/captures")
        .canonicalize()?;
    if store == root || !store.starts_with(root) {
        return Err("Graphics recovery demo store must be an isolated capture store".into());
    }
    Ok(Some(store))
}
