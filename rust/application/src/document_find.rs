//! Document traversal stays on the actor; presentation never drives page scanning.
use std::collections::VecDeque;
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

enum FindRange {
    Forward,
    Before(Option<FindCursor>),
}

pub struct FindRequest {
    pub document_id: String,
    query: PdfFindQuery,
    page_count: u32,
    cursor: FindCursor,
    range: FindRange,
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
            range: FindRange::Forward,
            cancellation,
            progress: FindProgress(Arc::new(AtomicU32::new(cursor.page))),
        })
    }
    /// Returns the last batch strictly before `boundary`; None searches to the end.
    pub fn previous(
        document_id: String,
        query: PdfFindQuery,
        page_count: u32,
        boundary: Option<FindCursor>,
        cancellation: Cancellation,
    ) -> Result<Self> {
        if boundary.is_some_and(|cursor| cursor.page >= page_count) {
            return Err(Failure::Invalid(
                "PDF previous-search boundary is invalid".into(),
            ));
        }
        let mut request = Self::new(
            document_id,
            query,
            page_count,
            FindCursor::default(),
            cancellation,
        )?;
        request.range = FindRange::Before(boundary);
        Ok(request)
    }
}

struct MatchPage {
    payload: Arc<PdfPageMatches>,
    start: usize,
    visible: usize,
}
/// Holds at most eighty page payloads and exposes at most eighty matches.
/// Payloads keep their existing governor leases; snippets and geometry are not copied.
pub struct FindBatch {
    pages: VecDeque<MatchPage>,
    pub start: Option<FindCursor>,
    pub next: Option<FindCursor>,
    pub scanned_pages: u32,
}
impl FindBatch {
    pub fn matches(&self) -> impl Iterator<Item = (u32, &PdfMatch)> {
        self.pages.iter().flat_map(|page| {
            page.payload
                .matches()
                .iter()
                .skip(page.start)
                .take(page.visible)
                .map(|item| (page.payload.page(), item))
        })
    }
}

pub(crate) struct FindScan {
    pub request: FindRequest,
    cursor: FindCursor,
    pages: VecDeque<MatchPage>,
    count: usize,
    ticket: PdfTicket<Arc<PdfPageMatches>>,
}
pub(crate) enum ScanStep {
    Continue(FindScan),
    Finished(FindBatch),
}
impl FindScan {
    pub fn start(request: FindRequest, engine: &impl AsyncPdfEngine) -> Result<Self> {
        request.cancellation.check()?;
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
            pages: VecDeque::new(),
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
        let backward = matches!(request.range, FindRange::Before(_));
        let boundary = match request.range {
            FindRange::Forward => None,
            FindRange::Before(boundary) => boundary,
        };
        let within_range = payload
            .matches()
            .iter()
            .take_while(|item| {
                boundary.is_none_or(|end| cursor.page < end.page || item.ordinal < end.ordinal)
            })
            .count();
        let reached_boundary = boundary.is_some_and(|end| cursor.page == end.page)
            && (within_range < payload.matches().len()
                || payload.next_match().is_none()
                || payload
                    .next_match()
                    .is_some_and(|next| next >= boundary.unwrap().ordinal));
        let visible = if backward {
            within_range
        } else {
            within_range.min(FIND_PAGE_SIZE - count)
        };
        let next = if reached_boundary {
            None
        } else if let Some(omitted) = payload.matches().get(visible) {
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
        let scanned = if reached_boundary {
            cursor.page
        } else {
            next.map_or(request.page_count, |next| next.page)
        };
        request.progress.0.store(scanned, Ordering::Release);
        if scanned > cursor.page && scanned.is_multiple_of(16) {
            notify();
        }
        count += visible;
        if visible > 0 {
            pages.push_back(MatchPage {
                payload,
                start: 0,
                visible,
            });
        } else {
            drop(payload);
        }
        while backward && count > FIND_PAGE_SIZE {
            let remove = (count - FIND_PAGE_SIZE).min(pages.front().unwrap().visible);
            let front = pages.front_mut().unwrap();
            front.start += remove;
            front.visible -= remove;
            count -= remove;
            if front.visible == 0 {
                pages.pop_front();
            }
        }
        if (!backward && count == FIND_PAGE_SIZE) || next.is_none() {
            return Ok(ScanStep::Finished(FindBatch {
                start: pages.front().map(|page| FindCursor {
                    page: page.payload.page(),
                    ordinal: page.payload.matches()[page.start].ordinal,
                }),
                pages,
                next: if backward { boundary } else { next },
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
