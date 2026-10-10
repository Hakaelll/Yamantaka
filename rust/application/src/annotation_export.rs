use std::io::Write;
use yamantaka_domain::{
    Cancellation, Failure, Result,
    annotations::{AnnotationColor, AnnotationKind, repository::AnnotationRecord},
};

#[derive(Clone, Copy)]
pub enum AnnotationExportFormat {
    Markdown,
    Json,
}
pub struct AnnotationExportMetadata<'a> {
    pub document: &'a str,
    pub title: &'a str,
    pub author: Option<&'a str>,
    pub progress: f64,
    pub exported_at: &'a str,
}
pub struct AnnotationExporter<'a, W: Write> {
    writer: W,
    document: &'a str,
    format: AnnotationExportFormat,
    cancel: Cancellation,
    previous_page: Option<u32>,
    count: u64,
    failed: bool,
}
fn utc(value: &str) -> Result<String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|time| {
            time.with_timezone(&chrono::Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
        })
        .map_err(|_| Failure::Invalid("Annotation export timestamp is invalid".into()))
}
fn write_error(_: impl std::fmt::Display) -> Failure {
    Failure::Storage("Annotation export could not be written".into())
}
fn text(writer: &mut impl Write, value: &str, quote: bool) -> Result<()> {
    for character in value.chars() {
        match character {
            '\n' if quote => writer.write_all(b"\n> "),
            '\r' => continue,
            '&' => writer.write_all(b"&amp;"),
            '<' => writer.write_all(b"&lt;"),
            '>' => writer.write_all(b"&gt;"),
            '\\' | '#' | '*' | '_' | '[' | ']' | '`' => write!(writer, "\\{character}"),
            _ => write!(writer, "{character}"),
        }
        .map_err(write_error)?;
    }
    Ok(())
}
impl<'a, W: Write> AnnotationExporter<'a, W> {
    pub fn new(
        mut writer: W,
        format: AnnotationExportFormat,
        metadata: AnnotationExportMetadata<'a>,
        cancel: Cancellation,
    ) -> Result<Self> {
        cancel.check()?;
        if metadata.document.is_empty()
            || !metadata.progress.is_finite()
            || metadata.exported_at.is_empty()
        {
            return Err(Failure::Invalid(
                "Annotation export metadata is invalid".into(),
            ));
        }
        let exported_at = utc(metadata.exported_at)?;
        match format {
            AnnotationExportFormat::Json => {
                writer
                    .write_all(
                        b"{\"format\":\"yamantaka-annotations\",\"version\":1,\"exportedAt\":",
                    )
                    .map_err(write_error)?;
                serde_json::to_writer(&mut writer, &exported_at).map_err(write_error)?;
                writer
                    .write_all(b",\"metadata\":{\"documentId\":")
                    .map_err(write_error)?;
                serde_json::to_writer(&mut writer, metadata.document).map_err(write_error)?;
                writer.write_all(b",\"title\":").map_err(write_error)?;
                serde_json::to_writer(&mut writer, metadata.title).map_err(write_error)?;
                if let Some(author) = metadata.author {
                    writer.write_all(b",\"author\":").map_err(write_error)?;
                    serde_json::to_writer(&mut writer, author).map_err(write_error)?;
                }
                write!(writer,",\"progress\":{}}},\"configuration\":{{\"coordinates\":\"normalized-page-v1\"}},\"annotations\":[",metadata.progress.clamp(0.0,1.0)).map_err(write_error)?;
            }
            AnnotationExportFormat::Markdown => {
                writer.write_all(b"# ").map_err(write_error)?;
                text(&mut writer, metadata.title, false)?;
                writer
                    .write_all(b"\n\n- Exportado: ")
                    .map_err(write_error)?;
                text(&mut writer, &exported_at, false)?;
                write!(
                    writer,
                    "\n- Progreso: {:.0} %\n",
                    metadata.progress.clamp(0.0, 1.0) * 100.0
                )
                .map_err(write_error)?;
                if let Some(author) = metadata.author.filter(|value| !value.trim().is_empty()) {
                    writer.write_all(b"- Autor: ").map_err(write_error)?;
                    text(&mut writer, author, false)?;
                    writer.write_all(b"\n").map_err(write_error)?;
                }
            }
        }
        Ok(Self {
            writer,
            document: metadata.document,
            format,
            cancel,
            previous_page: None,
            count: 0,
            failed: false,
        })
    }
    pub fn push(&mut self, record: &AnnotationRecord) -> Result<()> {
        if self.failed {
            return Err(Failure::Invalid(
                "Annotation export has already failed".into(),
            ));
        }
        let result = self.write_record(record);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn write_record(&mut self, record: &AnnotationRecord) -> Result<()> {
        self.cancel.check()?;
        let data = record.data();
        let values = &data.values;
        if values.document_id != self.document
            || self
                .previous_page
                .is_some_and(|page| page > values.page_index)
        {
            return Err(Failure::Invalid(
                "Annotation export source identity or page order is invalid".into(),
            ));
        }
        let created_at = utc(&data.created_at)?;
        let updated_at = utc(&data.updated_at)?;
        match self.format {
            AnnotationExportFormat::Json => {
                if self.count != 0 {
                    self.writer.write_all(b",").map_err(write_error)?;
                }
                self.writer.write_all(b"{\"id\":").map_err(write_error)?;
                serde_json::to_writer(&mut self.writer, &data.id).map_err(write_error)?;
                write!(
                    self.writer,
                    ",\"pageIndex\":{},\"type\":",
                    values.page_index
                )
                .map_err(write_error)?;
                serde_json::to_writer(&mut self.writer, &values.kind).map_err(write_error)?;
                self.writer.write_all(b",\"color\":").map_err(write_error)?;
                serde_json::to_writer(&mut self.writer, &values.color).map_err(write_error)?;
                for (name, value) in [
                    ("note", &values.note),
                    ("selectedText", &values.selected_text),
                    ("contextBefore", &values.context_before),
                    ("contextAfter", &values.context_after),
                    ("originDocumentId", &data.origin_document_id),
                    ("originFingerprint", &data.origin_fingerprint),
                ] {
                    if let Some(value) = value {
                        write!(self.writer, ",\"{name}\":").map_err(write_error)?;
                        serde_json::to_writer(&mut self.writer, value).map_err(write_error)?;
                    }
                }
                self.writer.write_all(b",\"rects\":").map_err(write_error)?;
                serde_json::to_writer(&mut self.writer, &values.rects).map_err(write_error)?;
                write!(
                    self.writer,
                    ",\"pageRotation\":{},\"documentFingerprint\":",
                    values.page_rotation
                )
                .map_err(write_error)?;
                serde_json::to_writer(&mut self.writer, &values.document_fingerprint)
                    .map_err(write_error)?;
                self.writer
                    .write_all(b",\"status\":")
                    .map_err(write_error)?;
                serde_json::to_writer(&mut self.writer, &values.status).map_err(write_error)?;
                self.writer
                    .write_all(b",\"reviewState\":")
                    .map_err(write_error)?;
                serde_json::to_writer(&mut self.writer, &values.review_state)
                    .map_err(write_error)?;
                self.writer.write_all(b",\"tags\":").map_err(write_error)?;
                serde_json::to_writer(&mut self.writer, &data.tags).map_err(write_error)?;
                for (name, value) in [("createdAt", &created_at), ("updatedAt", &updated_at)] {
                    write!(self.writer, ",\"{name}\":").map_err(write_error)?;
                    serde_json::to_writer(&mut self.writer, value).map_err(write_error)?;
                }
                self.writer.write_all(b"}").map_err(write_error)?;
            }
            AnnotationExportFormat::Markdown => {
                let kind = match values.kind {
                    AnnotationKind::Highlight => "Resaltado",
                    AnnotationKind::Underline => "Subrayado",
                    AnnotationKind::Strikeout => "Tachado",
                    AnnotationKind::TextNote => "Nota de texto",
                    AnnotationKind::PageMarker => "Marcador visual",
                    AnnotationKind::PageComment => "Comentario de página",
                };
                let color = match values.color {
                    AnnotationColor::Yellow => "Amarillo suave",
                    AnnotationColor::Green => "Verde suave",
                    AnnotationColor::Blue => "Azul suave",
                    AnnotationColor::Rose => "Rosa suave",
                    AnnotationColor::Violet => "Violeta suave",
                };
                let page = u64::from(values.page_index) + 1;
                if self.previous_page != Some(values.page_index) {
                    write!(self.writer, "\n<a id=\"page-{page}\"></a>\n").map_err(write_error)?;
                }
                write!(
                    self.writer,
                    "\n## Página {page} — {kind}\n\n- Color: {color}\n"
                )
                .map_err(write_error)?;
                if !data.tags.is_empty() {
                    self.writer
                        .write_all(b"- Etiquetas: ")
                        .map_err(write_error)?;
                    for (index, tag) in data.tags.iter().enumerate() {
                        if index != 0 {
                            self.writer.write_all(b", ").map_err(write_error)?;
                        }
                        text(&mut self.writer, tag, false)?;
                    }
                    self.writer.write_all(b"\n").map_err(write_error)?;
                }
                if let Some(passage) = &values.selected_text {
                    self.writer.write_all(b"\n> ").map_err(write_error)?;
                    text(&mut self.writer, passage, true)?;
                    self.writer.write_all(b"\n").map_err(write_error)?;
                }
                if let Some(note) = &values.note {
                    self.writer.write_all(b"\n").map_err(write_error)?;
                    text(&mut self.writer, note, false)?;
                    self.writer.write_all(b"\n").map_err(write_error)?;
                }
                write!(self.writer, "\n[Abrir página {page}](#page-{page})\n")
                    .map_err(write_error)?;
            }
        }
        self.previous_page = Some(values.page_index);
        self.count = self.count.checked_add(1).ok_or(Failure::MemoryBudget)?;
        Ok(())
    }
    pub fn finish(mut self) -> Result<W> {
        self.cancel.check()?;
        if self.failed {
            return Err(Failure::Invalid(
                "Annotation export has already failed".into(),
            ));
        }
        if matches!(self.format, AnnotationExportFormat::Json) {
            self.writer.write_all(b"]}\n").map_err(write_error)?;
        }
        self.writer.flush().map_err(write_error)?;
        Ok(self.writer)
    }
}
