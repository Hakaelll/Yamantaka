use pdfium_render::prelude::{PdfDocument, Pdfium};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek},
    path::{Path, PathBuf},
    time::SystemTime,
};
use yamantaka_domain::{Cancellation, Failure, Result};

pub(crate) struct LoadedPdf<'a> {
    pub purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose,
    pub document: PdfDocument<'a>,
    pub path: PathBuf,
    pub source: SourceStamp,
}

impl LoadedPdf<'_> {
    pub fn with_source<T: 'static>(
        &self,
        operation: impl FnOnce(&PdfDocument<'_>) -> Result<T>,
    ) -> Result<T> {
        self.source
            .with_source(&self.path, || operation(&self.document))
    }
}

#[derive(PartialEq, Eq)]
pub(crate) struct SourceStamp {
    canonical: PathBuf,
    size: u64,
    modified: SystemTime,
    head: [u8; 32],
    physical: yamantaka_windows::source_stamp::Stamp,
}

impl SourceStamp {
    pub fn snapshot(&self) -> yamantaka_domain::source_revision::SourceRevision {
        yamantaka_domain::source_revision::SourceRevision {
            canonical_path: self.canonical.clone(),
            size: self.size,
            modified: self.modified,
            head_sha256: self.head,
        }
    }
    pub fn read(path: &Path, cancel: &Cancellation) -> Result<Self> {
        Self::open(path, cancel).map(|(source, _)| source)
    }

    pub fn open(path: &Path, cancel: &Cancellation) -> Result<(Self, File)> {
        cancel.check()?;
        let canonical = path.canonicalize().map_err(source_error)?;
        let mut file = File::open(&canonical).map_err(source_error)?;
        let before = file.metadata().map_err(source_error)?;
        let physical =
            yamantaka_windows::source_stamp::Stamp::from_file(&file).ok_or_else(source_changed)?;
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
        }) || yamantaka_windows::source_stamp::Stamp::from_file(&file).as_ref()
            != Some(&physical)
            || yamantaka_windows::source_stamp::Stamp::read(&canonical).as_ref() != Some(&physical)
        {
            return Err(source_changed());
        }
        cancel.check()?;
        let source = Self {
            canonical,
            size: before.len(),
            modified,
            head: hash.finalize().into(),
            physical,
        };
        file.rewind().map_err(source_error)?;
        Ok((source, file))
    }

    pub fn lease(&self, path: &Path) -> Result<yamantaka_windows::source_stamp::Lease> {
        yamantaka_windows::source_stamp::Lease::acquire(path, self.physical.version())
            .map_err(|_| source_changed())
    }

    pub fn with_source<T: 'static>(
        &self,
        path: &Path,
        operation: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let lease = self.lease(path)?;
        let result = operation();
        lease.verify().map_err(|_| source_changed())?;
        result
    }

    pub fn open_pdf<'a>(
        &self,
        pdfium: &'a Pdfium,
        path: &Path,
        cancel: &Cancellation,
    ) -> Result<PdfDocument<'a>> {
        let (source, file) = Self::open(path, cancel)?;
        if source != *self {
            return Err(source_changed());
        }
        pdfium
            .load_pdf_from_reader(file, None)
            .map_err(crate::worker::pdf_failure)
    }

    pub fn verify(&self, path: &Path, cancel: &Cancellation) -> Result<()> {
        if Self::read(path, cancel)? != *self {
            return Err(source_changed());
        }
        Ok(())
    }
}

pub(crate) fn source_changed() -> Failure {
    Failure::PdfSourceChanged
}

fn source_error(error: std::io::Error) -> Failure {
    Failure::Pdf(format!("PDF source could not be inspected: {error}"))
}
