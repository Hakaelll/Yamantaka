use pdfium_render::prelude::Pdfium;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use yamantaka_domain::{Failure, Result};

static LIBRARY: Mutex<Option<PathBuf>> = Mutex::new(None);

pub(crate) fn open(path: &Path) -> Result<Pdfium> {
    let path = path.canonicalize().map_err(|_| unavailable())?;
    let mut library = LIBRARY.lock().map_err(|_| unavailable())?;
    if let Some(loaded) = library.as_ref() {
        if loaded != &path {
            return Err(Failure::Pdf(
                "PDFium is already bound to a different library".into(),
            ));
        }
        // This branch is reachable only after our explicit bundled binding was
        // initialized. Default therefore reuses it; it cannot search the system.
        return Ok(Pdfium::default());
    }
    let bindings = Pdfium::bind_to_library(&path).map_err(|_| unavailable())?;
    let pdfium = Pdfium::new(bindings);
    *library = Some(path);
    Ok(pdfium)
}

fn unavailable() -> Failure {
    Failure::Pdf("Bundled PDFium could not be loaded. Verify the release bundle.".into())
}
