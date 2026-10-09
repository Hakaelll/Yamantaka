//! PDFium calls are owned by one worker; no PDFium handle crosses threads.
mod cache;
mod outline;
mod worker;
pub use worker::{MemoryLimits, PdfWorker};
