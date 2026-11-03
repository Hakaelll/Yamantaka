//! Synthetic source-coherence diagnosis. Never accepts a user's PDF as input.
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf};
use yamantaka_domain::{
    Cancellation, Failure, PdfEngine, Priority, RenderKey, pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

fn pdf(label: &str) -> Vec<u8> {
    let content = format!("BT /F1 24 Tf 60 700 Td ({label}) Tj ET\n");
    let padding = " ".repeat(70_000);
    let objects=[
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [4 0 R] /Count 1 >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{padding}\nendstream",padding.len()),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 6 0 R >> >> /Contents 5 0 R >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{content}endstream",content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
    ];
    let mut bytes = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
    }
    let xref = bytes.len();
    bytes.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!("trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    bytes
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let library = PathBuf::from(args.first().ok_or("Expected PDFium library")?);
    let report = PathBuf::from(args.get(1).ok_or("Expected report path")?);
    if report
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("json"))
    {
        return Err("Expected a new JSON report path".into());
    }
    let mut report_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&report)?;
    let observe = args.iter().any(|arg| arg == "--observe");
    let temporary = tempfile::tempdir_in(report.parent().ok_or("Expected report parent")?)?;
    let path = temporary.path().join("synthetic-source.pdf");
    let original = pdf("Original source");
    let replacement = pdf("Replaced source");
    assert_eq!(original.len(), replacement.len());
    assert_eq!(&original[..65536], &replacement[..65536]);
    fs::write(&path, &original)?;
    let modified = fs::metadata(&path)?.modified()?;
    let worker = PdfWorker::start(&library, MemoryLimits::default())?;
    let cancel = Cancellation::default();
    worker.open("source".into(), path.clone(), None, cancel.clone())?;
    let key = RenderKey {
        document_id: "source".into(),
        page: 0,
        region: None,
        physical_width: 320,
        physical_height: 420,
        rotation: 0,
        generation: 1,
    };
    let old = worker.render(key.clone(), Priority::Visible, cancel.clone())?;
    let old_digest = Sha256::digest(old.pixels());
    assert!(
        worker
            .text("source".into(), 0, cancel.clone())?
            .contains("Original source")
    );
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(&replacement)?;
        file.set_modified(modified)?;
    }
    assert_eq!(fs::metadata(&path)?.modified()?, modified);
    let cached_render = worker
        .render(key.clone(), Priority::Visible, cancel.clone())
        .is_err_and(|error| matches!(error, Failure::PdfSourceChanged));
    let cached_text = worker
        .text("source".into(), 0, cancel.clone())
        .is_err_and(|error| matches!(error, Failure::PdfSourceChanged));
    let outline = worker
        .try_outline("source".into(), cancel.clone())?
        .wait()
        .is_err_and(|error| matches!(error, Failure::PdfSourceChanged));
    let geometry = worker
        .try_text_geometry(
            "source".into(),
            0,
            yamantaka_domain::pdf_text::TextGeometryPurpose::FrontMatter,
            cancel.clone(),
        )?
        .wait()
        .is_err_and(|error| matches!(error, Failure::PdfSourceChanged));
    let links = worker
        .try_page_links("source".into(), 0, cancel.clone())?
        .wait()
        .is_err_and(|error| matches!(error, Failure::PdfSourceChanged));
    let find = worker
        .try_find_page(
            "source".into(),
            0,
            yamantaka_domain::pdf_find::PdfFindQuery::new("source", false, false)?,
            0,
            cancel.clone(),
        )?
        .wait()
        .is_err_and(|error| matches!(error, Failure::PdfSourceChanged));
    worker.open("source".into(), path.clone(), None, cancel.clone())?;
    assert!(
        worker
            .text("source".into(), 0, cancel.clone())?
            .contains("Replaced source")
    );
    let new = worker.render(key, Priority::Visible, cancel.clone())?;
    assert_ne!(old.pixels(), new.pixels());
    assert_eq!(Sha256::digest(old.pixels()), old_digest);
    fs::rename(&path, temporary.path().join("previous-source.pdf"))?;
    fs::write(&path, &replacement)?;
    fs::OpenOptions::new()
        .write(true)
        .open(&path)?
        .set_modified(modified)?;
    let replaced = worker
        .text("source".into(), 0, cancel)
        .is_err_and(|error| matches!(error, Failure::PdfSourceChanged));
    let version = yamantaka_windows::source_stamp::Stamp::read(&path)
        .ok_or("Missing physical source stamp")?
        .version();
    let lease = yamantaka_windows::source_stamp::Lease::acquire(&path, version)?;
    let write_blocked = fs::OpenOptions::new().write(true).open(&path).is_err();
    lease.verify()?;
    drop(lease);
    let write_released = fs::OpenOptions::new().write(true).open(&path).is_ok();
    assert!(write_blocked && write_released);
    worker.close("source".into())?;
    let text = format!(
        "{{\"synthetic\":true,\"legacyPrefixUnchanged\":true,\"sizeAndModifiedUnchanged\":true,\"inPlaceWriteBetweenJobsAllowed\":true,\"cachedRenderRejected\":{cached_render},\"cachedTextRejected\":{cached_text},\"outlineRejected\":{outline},\"textGeometryRejected\":{geometry},\"linksRejected\":{links},\"findRejected\":{find},\"reopenUsesNewContent\":true,\"oldPixelsUnchanged\":true,\"pathReplacementRejected\":{replaced},\"leaseBlocksWrites\":{write_blocked},\"leaseReleasesWrites\":{write_released}}}\n"
    );
    report_file.write_all(text.as_bytes())?;
    if !observe
        && ![
            cached_render,
            cached_text,
            outline,
            geometry,
            links,
            find,
            replaced,
        ]
        .into_iter()
        .all(|value| value)
    {
        return Err("A changed source was accepted".into());
    }
    Ok(())
}
