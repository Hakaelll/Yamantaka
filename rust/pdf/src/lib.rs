//! PDFium calls are owned by one worker; no PDFium handle crosses threads.
mod bindings;
mod cache;
mod destination;
mod find;
mod links;
mod outline;
mod page_geometry;
mod worker;
pub use worker::{MemoryLimits, PdfWorker};
