use super::*;
use std::collections::BTreeMap;
use yamantaka_domain::{
    Failure,
    pdf_find::{PdfMatch, PdfPageMatches},
};

#[derive(Default)]
pub(super) struct Highlights {
    identity: Option<(String, PdfFindQuery)>,
    pages: BTreeMap<u32, Page>,
}
struct Page {
    chunks: Vec<Arc<PdfPageMatches>>,
    next: Option<u32>,
    pending: Option<Cancellation>,
    retry_at: Option<Instant>,
    failed: bool,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            next: Some(0),
            pending: None,
            retry_at: None,
            failed: false,
        }
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}
impl Highlights {
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
    pub(super) fn matches(&self, page: u32) -> impl Iterator<Item = &PdfMatch> {
        self.pages
            .get(&page)
            .into_iter()
            .flat_map(|page| page.chunks.iter().flat_map(|chunk| chunk.matches()))
    }
    pub(super) fn contains(&self, page: u32, ordinal: u32) -> bool {
        self.pages.get(&page).is_some_and(|page| {
            page.chunks.iter().any(|chunk| {
                chunk
                    .matches()
                    .binary_search_by_key(&ordinal, |item| item.ordinal)
                    .is_ok()
            })
        })
    }
}

impl FindState {
    pub(in crate::app) fn clear_highlights(&mut self) {
        self.highlights.clear();
    }
    pub(in crate::app) fn request_highlights(
        &mut self,
        runtime: &Runtime,
        document: &str,
        visible: &[u32],
        ctx: &egui::Context,
    ) {
        self.highlights
            .pages
            .retain(|page, _| visible.contains(page));
        let Some(query) = &self.query else { return };
        if self.document.as_deref() != Some(document)
            || !matches!(self.phase, FindPhase::Ready { .. })
        {
            return;
        }
        if self
            .highlights
            .identity
            .as_ref()
            .is_none_or(|(id, previous)| id != document || previous != query)
        {
            self.highlights.clear();
            self.highlights.identity = Some((document.to_owned(), query.clone()));
        }
        let mut available = 2_usize.saturating_sub(
            self.highlights
                .pages
                .values()
                .filter(|page| page.pending.is_some())
                .count(),
        );
        for &page in visible {
            if available == 0 {
                break;
            }
            let slot = self.highlights.pages.entry(page).or_default();
            if slot.failed || slot.pending.is_some() {
                continue;
            }
            let Some(offset) = slot.next else { continue };
            if let Some(at) = slot.retry_at
                && at > Instant::now()
            {
                ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
                continue;
            }
            let cancel = Cancellation::default();
            match runtime.submit(Action::FindPage(
                document.to_owned(),
                page,
                query.clone(),
                offset,
                cancel.clone(),
            )) {
                Ok(()) => {
                    slot.pending = Some(cancel);
                    slot.retry_at = None;
                    available -= 1;
                }
                Err(Failure::QueueFull) => {
                    slot.retry_at = Some(Instant::now() + Duration::from_millis(100));
                    ctx.request_repaint_after(Duration::from_millis(100));
                    break;
                }
                Err(_) => slot.failed = true,
            }
        }
    }

    pub(in crate::app) fn accept_highlights(
        &mut self,
        document: &str,
        payload: Arc<PdfPageMatches>,
        cancel: &Cancellation,
    ) {
        if cancel.is_cancelled()
            || self
                .highlights
                .identity
                .as_ref()
                .is_none_or(|(id, _)| id != document)
        {
            return;
        }
        let Some(slot) = self.highlights.pages.get_mut(&payload.page()) else {
            return;
        };
        if !slot
            .pending
            .as_ref()
            .is_some_and(|current| current.same_request(cancel))
        {
            return;
        }
        slot.pending = None;
        let offset = slot.next.unwrap_or(0);
        if payload
            .matches()
            .first()
            .is_some_and(|item| item.ordinal < offset)
            || payload.next_match().is_some_and(|next| next <= offset)
        {
            slot.failed = true;
            return;
        }
        slot.next = payload.next_match();
        slot.chunks.push(payload);
    }

    pub(in crate::app) fn fail_highlights(
        &mut self,
        document: &str,
        error: &Failure,
        cancel: &Cancellation,
    ) {
        if cancel.is_cancelled()
            || self
                .highlights
                .identity
                .as_ref()
                .is_none_or(|(id, _)| id != document)
        {
            return;
        }
        let Some(slot) = self.highlights.pages.values_mut().find(|slot| {
            slot.pending
                .as_ref()
                .is_some_and(|current| current.same_request(cancel))
        }) else {
            return;
        };
        slot.pending = None;
        if matches!(error, Failure::QueueFull) {
            slot.retry_at = Some(Instant::now() + Duration::from_millis(100));
        } else {
            slot.failed = true;
        }
    }

    pub(in crate::app) fn highlight_notice(&self, ui: &mut egui::Ui, english: bool, p: Palette) {
        if self.highlights.pages.values().any(|page| page.failed) {
            ui.add(egui::Label::new(RichText::new(if english {
                "Some page highlights are unavailable. Run the search again to retry."
            } else {
                "Algunos resaltados no están disponibles. Repite la búsqueda para reintentarlo."
            }).color(p.secondary)).wrap());
        }
    }
}
