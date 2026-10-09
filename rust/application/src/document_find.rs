//! Document traversal stays on the actor; presentation never drives page scanning.
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    pdf_find::{FIND_PAGE_SIZE, PdfFindQuery, PdfMatch, PdfPageMatches},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FindCursor {
    pub page: u32,
    pub ordinal: u32,
}

#[derive(Clone, Default)]
pub struct FindProgress(Arc<AtomicU32>);
impl FindProgress {
    pub fn scanned_pages(&self) -> u32 {
        self.0.load(Ordering::Acquire)
    }
}

pub struct FindRequest {
    pub document_id: String,
    query: PdfFindQuery,
    page_count: u32,
    cursor: FindCursor,
    pub cancellation: Cancellation,
    pub progress: FindProgress,
}
impl FindRequest {
    pub fn new(
        document_id: String,
        query: PdfFindQuery,
        page_count: u32,
        cursor: FindCursor,
        cancellation: Cancellation,
    ) -> Result<Self> {
        if document_id.is_empty() || page_count == 0 || cursor.page >= page_count {
            return Err(Failure::Invalid(
                "PDF document search range is invalid".into(),
            ));
        }
        cancellation.check()?;
        Ok(Self {
            document_id,
            query,
            page_count,
            cursor,
            cancellation,
            progress: FindProgress(Arc::new(AtomicU32::new(cursor.page))),
        })
    }
}

struct MatchPage {
    payload: Arc<PdfPageMatches>,
    visible: usize,
}
/// Holds at most eighty page payloads and exposes at most eighty matches.
/// Payloads keep their existing governor leases; snippets and geometry are not copied.
pub struct FindBatch {
    pages: Vec<MatchPage>,
    pub next: Option<FindCursor>,
    pub scanned_pages: u32,
}
impl FindBatch {
    pub fn matches(&self) -> impl Iterator<Item = (u32, &PdfMatch)> {
        self.pages.iter().flat_map(|page| {
            page.payload
                .matches()
                .iter()
                .take(page.visible)
                .map(|item| (page.payload.page(), item))
        })
    }
}

pub(crate) struct FindScan {
    pub request: FindRequest,
    cursor: FindCursor,
    pages: Vec<MatchPage>,
    count: usize,
    ticket: PdfTicket<Arc<PdfPageMatches>>,
}
pub(crate) enum ScanStep {
    Continue(FindScan),
    Finished(FindBatch),
}
impl FindScan {
    pub fn start(request: FindRequest, engine: &impl AsyncPdfEngine) -> Result<Self> {
        let cursor = request.cursor;
        let ticket = engine.try_find_page(
            request.document_id.clone(),
            cursor.page,
            request.query.clone(),
            cursor.ordinal,
            request.cancellation.clone(),
        )?;
        Ok(Self {
            request,
            cursor,
            pages: Vec::new(),
            count: 0,
            ticket,
        })
    }
    pub fn ticket(&self) -> &PdfTicket<Arc<PdfPageMatches>> {
        &self.ticket
    }
    pub fn advance(self, engine: &impl AsyncPdfEngine, notify: &impl Fn()) -> Result<ScanStep> {
        let Self {
            request,
            cursor,
            mut pages,
            mut count,
            ticket,
        } = self;
        request.cancellation.check()?;
        let payload = ticket.wait()?;
        request.cancellation.check()?;
        if payload.page() != cursor.page
            || payload
                .matches()
                .first()
                .is_some_and(|item| item.ordinal != cursor.ordinal)
        {
            return Err(Failure::Pdf(
                "PDF search returned a different cursor".into(),
            ));
        }
        let visible = payload.matches().len().min(FIND_PAGE_SIZE - count);
        let next = if let Some(omitted) = payload.matches().get(visible) {
            Some(FindCursor {
                page: cursor.page,
                ordinal: omitted.ordinal,
            })
        } else if let Some(ordinal) = payload.next_match() {
            Some(FindCursor {
                page: cursor.page,
                ordinal,
            })
        } else if cursor.page + 1 < request.page_count {
            Some(FindCursor {
                page: cursor.page + 1,
                ordinal: 0,
            })
        } else {
            None
        };
        let scanned = next.map_or(request.page_count, |next| next.page);
        request.progress.0.store(scanned, Ordering::Release);
        if scanned > cursor.page && scanned.is_multiple_of(16) {
            notify();
        }
        count += visible;
        if visible > 0 {
            pages.push(MatchPage { payload, visible });
        }
        if count == FIND_PAGE_SIZE || next.is_none() {
            return Ok(ScanStep::Finished(FindBatch {
                pages,
                next,
                scanned_pages: scanned,
            }));
        }
        let cursor = next.expect("unfinished search has a continuation");
        let ticket = engine.try_find_page(
            request.document_id.clone(),
            cursor.page,
            request.query.clone(),
            cursor.ordinal,
            request.cancellation.clone(),
        )?;
        Ok(ScanStep::Continue(Self {
            request,
            cursor,
            pages,
            count,
            ticket,
        }))
    }
}
