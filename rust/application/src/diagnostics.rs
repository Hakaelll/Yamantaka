use serde::Serialize;
use yamantaka_domain::{
    Cancellation, Result,
    diagnostics::*,
    memory::{GlobalMemoryLimits, MemoryGovernor, MemoryUsage},
};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Renderer {
    Wgpu,
    Glow,
}
#[derive(Serialize)]
pub struct MemoryCounters {
    total: usize,
    pages: usize,
    tiles: usize,
    text: usize,
    other: usize,
    cpu: usize,
    gpu: usize,
    transient: usize,
}
impl From<MemoryUsage> for MemoryCounters {
    fn from(value: MemoryUsage) -> Self {
        Self {
            total: value.total,
            pages: value.pages,
            tiles: value.tiles,
            text: value.text,
            other: value.other,
            cpu: value.cpu,
            gpu: value.gpu,
            transient: value.transient,
        }
    }
}
#[derive(Serialize)]
pub struct MemoryBudgets {
    total: usize,
    pages: usize,
    tiles: usize,
    text: usize,
}
impl From<GlobalMemoryLimits> for MemoryBudgets {
    fn from(value: GlobalMemoryLimits) -> Self {
        Self {
            total: value.total,
            pages: value.pages,
            tiles: value.tiles,
            text: value.text,
        }
    }
}
#[derive(Serialize)]
pub struct Report {
    startup_mode: yamantaka_domain::startup::StartupMode,
    format: &'static str,
    version: u32,
    application_version: &'static str,
    platform: &'static str,
    architecture: &'static str,
    pdf_engine: &'static str,
    renderer: Renderer,
    pub database: DatabaseDiagnostics,
    memory: MemoryCounters,
    memory_limits: MemoryBudgets,
    memory_scope: &'static str,
    contains_private_content: bool,
}
impl Report {
    pub fn collect(
        repository: &impl DiagnosticsRepository,
        renderer: Renderer,
        startup_mode: yamantaka_domain::startup::StartupMode,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Self> {
        let database = repository.diagnostics(cancel)?;
        let memory = governor.snapshot();
        cancel.check()?;
        Ok(Self {
            startup_mode,
            format: "yamantaka-rust-safe-diagnostics",
            version: 1,
            application_version: env!("CARGO_PKG_VERSION"),
            platform: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            pdf_engine: "PDFium via pdfium-render",
            renderer,
            database,
            memory: memory.usage.into(),
            memory_limits: memory.limits.into(),
            memory_scope: "Governed resources only. Opaque PDFium and driver allocations, UI atlas and some live text and Glow uploads are not covered. Memory and database snapshots are collected separately.",
            contains_private_content: false,
        })
    }
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|_| {
            yamantaka_domain::Failure::Invalid("Diagnostic report encoding failed".into())
        })
    }
}
