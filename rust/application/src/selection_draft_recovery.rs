//! Rebuild selection geometry through the serialized PDF worker, one page at a time.
use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Failure, Result,
    annotations::selection_note_draft::SelectionNoteDraft,
    memory::{ByteLease, Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_text::{PdfPageText, TextGeometryPurpose},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
    text_selection::snapshot::TextSelectionSnapshot,
};

pub struct Request {
    pub draft: SelectionNoteDraft,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct Reply {
    pub draft: SelectionNoteDraft,
    pub result: Result<TextSelectionSnapshot>,
    pub cancel: Cancellation,
}
pub(crate) struct Scan {
    pub request: Request,
    ticket: Option<PdfTicket<Arc<PdfPageText>>>,
    pages: Vec<Arc<PdfPageText>>,
    _lease: ByteLease,
}
pub(crate) enum Step {
    Continue(Box<Scan>),
    Finished(Box<Reply>),
}
impl Request {
    fn reply(self, result: Result<TextSelectionSnapshot>) -> Box<Reply> {
        Box::new(Reply {
            draft: self.draft,
            result,
            cancel: self.cancel,
        })
    }
}
impl Scan {
    pub fn start(
        request: Request,
        engine: &impl AsyncPdfEngine,
    ) -> std::result::Result<Self, Box<Reply>> {
        let prepared = (|| {
            request.cancel.check()?;
            request.draft.validate()?;
            let range = request.draft.selection()?;
            let count = usize::try_from(
                u64::from(range.end().page()) - u64::from(range.start().page()) + 1,
            )
            .map_err(|_| Failure::MemoryBudget)?;
            let bytes = count
                .checked_mul(std::mem::size_of::<Arc<PdfPageText>>())
                .and_then(|n| n.checked_add(std::mem::size_of::<Self>()))
                .ok_or(Failure::MemoryBudget)?;
            let mut lease = request
                .governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Text,
                    MemoryRepresentation::Cpu,
                    bytes,
                ))
                .map_err(|_| Failure::MemoryBudget)?;
            let mut pages = Vec::new();
            pages
                .try_reserve_exact(count)
                .map_err(|_| Failure::MemoryBudget)?;
            lease
                .try_resize(
                    pages
                        .capacity()
                        .checked_mul(std::mem::size_of::<Arc<PdfPageText>>())
                        .and_then(|n| n.checked_add(std::mem::size_of::<Self>()))
                        .ok_or(Failure::MemoryBudget)?,
                )
                .map_err(|_| Failure::MemoryBudget)?;
            let ticket = engine.try_text_geometry(
                request.draft.document.clone(),
                range.start().page(),
                TextGeometryPurpose::Recovery,
                request.cancel.clone(),
            )?;
            Ok((ticket, pages, lease))
        })();
        match prepared {
            Ok((ticket, pages, lease)) => Ok(Self {
                request,
                ticket: Some(ticket),
                pages,
                _lease: lease,
            }),
            Err(error) => Err(request.reply(Err(error))),
        }
    }
    pub fn ticket(&self) -> &PdfTicket<Arc<PdfPageText>> {
        self.ticket
            .as_ref()
            .expect("pending recovery has one PDF ticket")
    }
    /// The coordinator calls this only after ticket readiness, never on the UI thread.
    pub fn advance(mut self: Box<Self>, engine: &impl AsyncPdfEngine) -> Step {
        let result = (|| {
            self.request.cancel.check()?;
            let page = self.ticket.take().unwrap().wait()?;
            self.request.cancel.check()?;
            let expected = u64::from(self.request.draft.start[0]) + self.pages.len() as u64;
            if u64::from(page.page()) != expected {
                return Err(Failure::Invalid(
                    "Recovered selection page identity changed".into(),
                ));
            }
            let last = page.page() == self.request.draft.end[0];
            self.pages.push(page);
            if !last {
                self.ticket = Some(engine.try_text_geometry(
                    self.request.draft.document.clone(),
                    u32::try_from(expected + 1).map_err(|_| Failure::MemoryBudget)?,
                    TextGeometryPurpose::Recovery,
                    self.request.cancel.clone(),
                )?);
            }
            Ok(last)
        })();
        match result {
            Ok(false) => Step::Continue(self),
            Ok(true) => {
                let result = (|| {
                    let snapshot = TextSelectionSnapshot::new(
                        self.request.draft.selection()?,
                        self.pages.drain(..).map(Ok),
                        &self.request.governor,
                    )?;
                    if snapshot.text() != self.request.draft.passage {
                        return Err(Failure::Invalid(
                            "Recovered selection passage changed".into(),
                        ));
                    }
                    self.request.cancel.check()?;
                    Ok(snapshot)
                })();
                Step::Finished(self.request.reply(result))
            }
            Err(error) => Step::Finished(self.request.reply(Err(error))),
        }
    }
}
