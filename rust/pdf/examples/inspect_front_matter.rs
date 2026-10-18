//! Read-only local diagnosis. Output can contain document text; keep it private.
use std::{path::PathBuf, sync::Arc};
use yamantaka_domain::{
    Cancellation, PdfEngine, front_matter::FrontMatter, pdf_text::TextGeometryPurpose,
    pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let library = PathBuf::from(args.next().ok_or("Expected PDFium library path")?);
    let path = PathBuf::from(args.next().ok_or("Expected PDF path")?).canonicalize()?;
    let worker = PdfWorker::start(&library, MemoryLimits::default())?;
    let cancel = Cancellation::default();
    let info = worker.open("inspect".into(), path.clone(), None, cancel.clone())?;
    println!(
        "Embedded title: {:?}\nEmbedded author: {:?}",
        info.title, info.author
    );
    let mut scan = FrontMatter::new(
        &path.file_name().unwrap_or_default().to_string_lossy(),
        info.title.as_deref(),
        info.author.as_deref(),
        &worker.memory_governor(),
    )?;
    for (page, geometry) in info.pages.iter().take(6).enumerate() {
        let text: Arc<_> = worker
            .try_text_geometry(
                "inspect".into(),
                page as u32,
                TextGeometryPurpose::FrontMatter,
                cancel.clone(),
            )?
            .wait()?;
        println!("Page {}: {:?}", page + 1, text.text());
        scan.observe(&text, geometry.coordinates(), &cancel)?;
    }
    let evidence = scan.finish(&cancel)?;
    println!("Title: {:?}\nAuthor: {:?}", evidence.title, evidence.author);
    worker.close("inspect".into())?;
    Ok(())
}
