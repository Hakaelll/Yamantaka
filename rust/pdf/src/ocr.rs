#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    io::{BufWriter, Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};
use yamantaka_domain::{
    Cancellation, Failure, RenderedPage, Result,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_find::{FIND_PAGE_SIZE, PdfFindQuery, PdfMatch, PdfPageMatches},
    pdf_geometry::NormalizedPageRect,
    pdf_text::{MAX_PAGE_TEXT_CHARACTERS, PdfPageText, TextCharacter},
    text_selection::{SelectionPosition, TextSelection},
};

pub(crate) fn recognize(
    helper: &Path,
    page: &RenderedPage,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<Arc<PdfPageText>> {
    let folder = tempfile::tempdir().map_err(io)?;
    let input = folder.path().join("page.gray");
    let output = folder.path().join("words.tsv");
    write_raster(&input, page, cancel)?;
    let stream = std::fs::File::create(&output).map_err(io)?;
    let mut command = Command::new(helper);
    command
        .arg(&input)
        .stdin(Stdio::null())
        .stdout(stream)
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let mut child = command.spawn().map_err(io)?;
    let started = Instant::now();
    loop {
        if cancel.is_cancelled() || started.elapsed() > Duration::from_secs(60) {
            let _ = child.kill();
            let _ = child.wait();
            cancel.check()?;
            return Err(Failure::Pdf("OCR timed out on this page".into()));
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) => {
                return Err(Failure::Pdf(
                    "OCR failed. Check the optional add-on and the Windows OCR language.".into(),
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(30)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io(error));
            }
        }
    }
    let mut report = String::new();
    std::fs::File::open(output)
        .map_err(io)?
        .take(2_000_001)
        .read_to_string(&mut report)
        .map_err(io)?;
    if report.len() > 2_000_000 {
        return Err(Failure::Invalid("OCR output is too large".into()));
    }
    parse(page.key.page, page.width, page.height, &report, governor)
}

fn io(_: std::io::Error) -> Failure {
    Failure::Pdf("The optional OCR add-on could not be started or read".into())
}

fn write_raster(path: &Path, page: &RenderedPage, cancel: &Cancellation) -> Result<()> {
    let mut image = BufWriter::new(std::fs::File::create(path).map_err(io)?);
    image.write_all(b"YOCR").map_err(io)?;
    image.write_all(&page.width.to_le_bytes()).map_err(io)?;
    image.write_all(&page.height.to_le_bytes()).map_err(io)?;
    let mut row = vec![0; page.width as usize];
    for source in page.pixels().chunks_exact(page.width as usize * 4) {
        cancel.check()?;
        for (pixel, output) in source.chunks_exact(4).zip(&mut row) {
            let gray =
                (u16::from(pixel[0]) * 30 + u16::from(pixel[1]) * 59 + u16::from(pixel[2]) * 11)
                    / 100;
            let alpha = u16::from(pixel[3]);
            *output = ((gray * alpha + (255 - alpha) * 255) / 255) as u8;
        }
        image.write_all(&row).map_err(io)?;
    }
    image.flush().map_err(io)
}
fn parse(
    page: u32,
    width: u32,
    height: u32,
    report: &str,
    governor: &MemoryGovernor,
) -> Result<Arc<PdfPageText>> {
    let invalid = || Failure::Invalid("Invalid OCR word geometry".into());
    let reserved = report.len().saturating_add(1).min(MAX_PAGE_TEXT_CHARACTERS);
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            reserved * (std::mem::size_of::<TextCharacter>() + 4),
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut text = String::new();
    let mut characters = Vec::new();
    let mut prior_line = None;
    for row in report.lines() {
        let mut fields = row.splitn(6, '\t');
        let mut dimension = || {
            fields
                .next()
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| v.is_finite())
                .ok_or_else(invalid)
        };
        let [x, y, w, h] = [dimension()?, dimension()?, dimension()?, dimension()?];
        let line = fields
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            .ok_or_else(invalid)?;
        let word = fields.next().ok_or_else(invalid)?.trim();
        let length = word.chars().count();
        if length == 0 {
            continue;
        }
        if characters.len() + length + 1 > MAX_PAGE_TEXT_CHARACTERS || w <= 0.0 || h <= 0.0 {
            return Err(invalid());
        }
        let [width, height] = [f64::from(width), f64::from(height)];
        if let Some(previous) = prior_line {
            let start = text.len() as u32;
            text.push(if line == previous { ' ' } else { '\n' });
            characters.push(TextCharacter {
                bytes: start..text.len() as u32,
                region: None,
            });
        }
        prior_line = Some(line);
        let step = w / length as f64;
        for (index, letter) in word.chars().enumerate() {
            let left = x + index as f64 * step;
            let region = NormalizedPageRect::new(
                [(left / width).clamp(0.0, 1.0), (y / height).clamp(0.0, 1.0)],
                [
                    ((left + step) / width).clamp(0.0, 1.0),
                    ((y + h) / height).clamp(0.0, 1.0),
                ],
            )
            .ok();
            let start = text.len() as u32;
            text.push(letter);
            characters.push(TextCharacter {
                bytes: start..text.len() as u32,
                region,
            });
        }
    }
    Ok(Arc::new(PdfPageText::from_reserved(
        page, text, characters, lease,
    )?))
}

pub(crate) fn find(
    page: &PdfPageText,
    query: &PdfFindQuery,
    offset: u32,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<Arc<PdfPageMatches>> {
    let matches_case = query.match_case();
    let pattern = if matches_case {
        query.text().into()
    } else {
        query.text().to_lowercase()
    };
    let mut folded = String::new();
    let mut offsets = Vec::with_capacity(page.characters().len() + 1);
    for letter in page.text().chars() {
        offsets.push(folded.len());
        if matches_case {
            folded.push(letter);
        } else {
            folded.extend(letter.to_lowercase());
        }
    }
    offsets.push(folded.len());
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            64 + offsets.capacity() * std::mem::size_of::<usize>() + folded.capacity(),
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut matches = Vec::new();
    let mut ordinal = 0;
    let mut next = None;
    for (byte, value) in folded.match_indices(&pattern) {
        cancel.check()?;
        if query.whole_words()
            && (folded[..byte]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
                || folded[byte + value.len()..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_'))
        {
            continue;
        }
        let index = ordinal;
        ordinal += 1;
        if index < offset {
            continue;
        }
        if matches.len() == FIND_PAGE_SIZE {
            next = Some(index);
            break;
        }
        let first = offsets.partition_point(|b| *b <= byte).saturating_sub(1);
        let last = offsets.partition_point(|b| *b < byte + value.len());
        let selection = TextSelection::new(
            SelectionPosition::new(page.page(), first)?,
            SelectionPosition::new(page.page(), last)?,
        );
        if let Some(part) = selection.page(page)? {
            matches.push(PdfMatch {
                ordinal: index,
                snippet: part.text().chars().take(160).collect(),
                rectangles: part.ink_regions().take(128).collect(),
            });
        }
    }
    Ok(Arc::new(PdfPageMatches::from_reserved(
        page.page(),
        matches,
        next,
        !page.text().trim().is_empty(),
        lease,
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognized_words_support_selection_and_document_search() {
        let governor = MemoryGovernor::new(Default::default());
        let page = parse(
            0,
            200,
            100,
            "12\t15\t45\t13\t0\tÁrbol\n65\t15\t70\t13\t0\tverde",
            &governor,
        )
        .unwrap();
        assert_eq!(page.text(), "Árbol verde");
        let query = PdfFindQuery::new("árbol", false, true).unwrap();
        let result = find(&page, &query, 0, &governor, &Cancellation::default()).unwrap();
        assert_eq!(result.matches()[0].snippet, "Árbol");
        assert!(result.matches()[0].position(0).is_some());
    }
}
