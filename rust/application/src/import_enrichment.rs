use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Failure, NewDocument,
    front_matter::{AnalysisFailure, FrontMatter, ImportAnalysis, MAX_FRONT_MATTER_PAGES},
    pdf_text::{PdfPageText, TextGeometryPurpose},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};

pub(crate) enum ImportStep {
    Pending(Box<ImportEnrichment>),
    Ready(Box<NewDocument>),
}

pub(crate) struct ImportEnrichment {
    pub id: String,
    document: Box<NewDocument>,
    page: u32,
    scan: FrontMatter,
    ticket: PdfTicket<Arc<PdfPageText>>,
    cancel: Cancellation,
}

fn unavailable(mut document: Box<NewDocument>, error: Failure) -> ImportStep {
    let reason = match error {
        Failure::MemoryBudget => AnalysisFailure::MemoryBudget,
        Failure::TextWorkLimit => AnalysisFailure::TextWorkLimit,
        _ => AnalysisFailure::Unavailable,
    };
    document.analysis = ImportAnalysis::Unavailable(reason);
    ImportStep::Ready(document)
}

impl ImportEnrichment {
    pub fn start(
        engine: &impl AsyncPdfEngine,
        id: String,
        document: Box<NewDocument>,
        cancel: Cancellation,
    ) -> ImportStep {
        let result = (|| {
            cancel.check()?;
            let governor = engine
                .memory_governor()
                .ok_or_else(|| Failure::Unsupported("Governed metadata analysis".into()))?;
            let file_name = document
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let scan = FrontMatter::new(
                &file_name,
                document.info.title.as_deref(),
                document.info.author.as_deref(),
                &governor,
            )?;
            let ticket = engine.try_text_geometry(
                id.clone(),
                0,
                TextGeometryPurpose::FrontMatter,
                cancel.clone(),
            )?;
            Ok((scan, ticket))
        })();
        match result {
            Ok((scan, ticket)) => ImportStep::Pending(Box::new(Self {
                id,
                document,
                page: 0,
                scan,
                ticket,
                cancel,
            })),
            Err(error) => unavailable(document, error),
        }
    }

    pub fn ticket(&self) -> &PdfTicket<Arc<PdfPageText>> {
        &self.ticket
    }

    pub fn advance(self, engine: &impl AsyncPdfEngine) -> ImportStep {
        let Self {
            id,
            mut document,
            page,
            mut scan,
            ticket,
            cancel,
        } = self;
        let observed = (|| {
            let text = ticket.wait()?;
            cancel.check()?;
            if text.page() != page {
                return Err(Failure::Invalid(
                    "Metadata text belongs to another page".into(),
                ));
            }
            let geometry =
                document.info.pages.get(page as usize).ok_or_else(|| {
                    Failure::Invalid("Metadata page is outside the document".into())
                })?;
            scan.observe(&text, geometry.coordinates(), &cancel)
        })();
        if let Err(error) = observed {
            return unavailable(document, error);
        }
        let next = page + 1;
        if next
            < document
                .info
                .pages
                .len()
                .min(MAX_FRONT_MATTER_PAGES as usize) as u32
        {
            match engine.try_text_geometry(
                id.clone(),
                next,
                TextGeometryPurpose::FrontMatter,
                cancel.clone(),
            ) {
                Ok(ticket) => ImportStep::Pending(Box::new(Self {
                    id,
                    document,
                    page: next,
                    scan,
                    ticket,
                    cancel,
                })),
                Err(error) => unavailable(document, error),
            }
        } else {
            match scan.finish(&cancel) {
                Ok(evidence) => {
                    document.analysis = ImportAnalysis::Complete(Box::new(evidence));
                    ImportStep::Ready(document)
                }
                Err(error) => unavailable(document, error),
            }
        }
    }
}
