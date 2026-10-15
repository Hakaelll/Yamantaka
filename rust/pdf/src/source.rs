use pdfium_render::prelude::PdfDocument;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};
use yamantaka_domain::{Cancellation, Failure, Result};

pub(crate) struct LoadedPdf<'a> {
    pub document: PdfDocument<'a>,
    pub path: PathBuf,
    pub source: SourceStamp,
}

#[derive(PartialEq, Eq)]
pub(crate) struct SourceStamp {
    canonical: PathBuf,
    size: u64,
    modified: SystemTime,
    head: [u8; 32],
}

impl SourceStamp {
    pub fn read(path: &Path, cancel: &Cancellation) -> Result<Self> {
        cancel.check()?;
        let canonical = path.canonicalize().map_err(source_error)?;
        let mut file = File::open(&canonical).map_err(source_error)?;
        let before = file.metadata().map_err(source_error)?;
        if !before.is_file() {
            return Err(Failure::Invalid("PDF source is not a regular file".into()));
        }
        let modified = before.modified().map_err(source_error)?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 4096];
        let mut remaining = 64 * 1024;
        while remaining > 0 {
            cancel.check()?;
            let amount = remaining.min(buffer.len());
            let read = file.read(&mut buffer[..amount]).map_err(source_error)?;
            if read == 0 {
                break;
            }
            hash.update(&buffer[..read]);
            remaining -= read;
        }
        let after = file.metadata().map_err(source_error)?;
        let current = std::fs::metadata(&canonical).map_err(source_error)?;
        if [after, current].iter().any(|metadata| {
            metadata.len() != before.len() || metadata.modified().ok() != Some(modified)
        }) {
            return Err(source_changed());
        }
        cancel.check()?;
        Ok(Self {
            canonical,
            size: before.len(),
            modified,
            head: hash.finalize().into(),
        })
    }

    pub fn verify(&self, path: &Path, cancel: &Cancellation) -> Result<()> {
        if Self::read(path, cancel)? != *self {
            return Err(source_changed());
        }
        Ok(())
    }
}

pub(crate) fn source_changed() -> Failure {
    Failure::Invalid("PDF source changed; reopen the document before requesting its cover".into())
}

fn source_error(error: std::io::Error) -> Failure {
    Failure::Pdf(format!("PDF source could not be inspected: {error}"))
}
