use super::*;
use shortcuts::{ReadingTarget, Shortcut};
mod actions;
mod catalog;
mod preview;
mod view;
use catalog::Command;
pub(in crate::app) use preview::PalettePreview;
use yamantaka_domain::PageLayout;
#[derive(Default)]
pub(in crate::app) struct PaletteState {
    pub open: bool,
    query: String,
    selected: usize,
    focus: bool,
    context: Option<(ReadingTarget, String)>,
    recents: Vec<Document>,
    pending: Option<Cancellation>,
    error: bool,
    info: bool,
    last: Option<String>,
}
impl PaletteState {
    pub fn owns(&self, cancel: &Cancellation) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(cancel))
    }
    pub fn accept(&mut self, documents: Vec<Document>, cancel: Cancellation) {
        self.pending = None;
        if self.open && !cancel.is_cancelled() {
            self.recents = documents.into_iter().take(8).collect();
        }
    }
    pub fn fail(&mut self, cancel: &Cancellation) -> bool {
        if !self.owns(cancel) {
            return false;
        }
        self.pending = None;
        self.error = !cancel.is_cancelled();
        true
    }
    fn close(&mut self) {
        self.open = false;
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
        self.recents.clear();
    }
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"open":self.open,"query":self.query,"recentCount":self.recents.len(),"pending":self.pending.is_some(),"info":self.info,"selected":self.selected,"last":self.last,"target":self.context.as_ref().map(|(target,_)|format!("{target:?}")),"document":self.context.as_ref().map(|(_,id)|id)})
    }
}
impl Drop for PaletteState {
    fn drop(&mut self) {
        self.close();
    }
}
impl YamantakaApp {
    pub(in crate::app) fn palette_document(&self) -> Option<&Document> {
        let (target, id) = self.command_palette.context.as_ref()?;
        if !matches!(self.surface, Surface::Reader) {
            return None;
        }
        let pane = if *target == ReadingTarget::Reference {
            &self
                .reference
                .as_ref()
                .filter(|reference| !reference.busy())?
                .pane
        } else {
            &self.reader
        };
        pane.active
            .as_ref()
            .filter(|(document, _)| document.id == *id)
            .map(|(document, _)| document)
    }
    pub(in crate::app) fn open_command_palette(&mut self, ctx: &egui::Context) {
        self.command_palette.close();
        let target = self.reading_focus.target;
        let pane = if target == ReadingTarget::Reference {
            self.reference
                .as_ref()
                .filter(|reference| !reference.busy())
                .map(|reference| &reference.pane)
        } else {
            Some(&self.reader)
        };
        let context = matches!(self.surface, Surface::Reader)
            .then(|| {
                pane.and_then(|pane| {
                    pane.active
                        .as_ref()
                        .map(|(document, _)| (target, document.id.clone()))
                })
            })
            .flatten();
        self.command_palette = PaletteState::default();
        self.command_palette.open = true;
        self.command_palette.focus = true;
        self.command_palette.context = context;
        egui::Popup::close_all(ctx);
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::Query(
            LibraryQuery {
                page_size: 8,
                sort: LibrarySort::LastOpened,
                sort_direction: SortDirection::Descending,
                ..Default::default()
            },
            cancel.clone(),
        )) {
            Ok(()) => self.command_palette.pending = Some(cancel),
            Err(_) => self.command_palette.error = true,
        }
        ctx.request_repaint();
    }
    fn close_command_palette(&mut self, ctx: &egui::Context) {
        let target = self
            .command_palette
            .context
            .as_ref()
            .map(|(target, _)| *target);
        self.command_palette.close();
        if let Some(target) = target {
            self.reading_focus.resume(
                ctx,
                if target == ReadingTarget::Reference && self.reference.is_none() {
                    ReadingTarget::Primary
                } else {
                    target
                },
            );
        }
    }
}
