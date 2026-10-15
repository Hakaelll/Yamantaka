//! PDFium calls are owned by one worker; no PDFium handle crosses threads.
mod bindings;
mod cache;
mod cover_detection;
mod covers;
mod destination;
mod find;
mod links;
mod outline;
mod page_geometry;
mod source;
mod text_geometry;
mod worker;
pub use worker::{MemoryLimits, PdfWorker};
