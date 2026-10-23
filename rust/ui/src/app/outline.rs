use super::*;
mod reference;
mod view;
use std::collections::HashSet;
use yamantaka_domain::outline::PdfOutline;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReaderPanelTab {
    Outline,
    Thumbnails,
    Find,
    Bookmarks,
    Annotations,
    Text,
}

#[derive(Clone, Copy)]
struct Row {
    entry: usize,
    depth: usize,
    children: bool,
}

#[derive(Default)]
pub(super) struct OutlineState {
    document: Option<String>,
    pending: Option<Cancellation>,
    outline: Option<Arc<PdfOutline>>,
    error: Option<String>,
    collapsed: HashSet<usize>,
    rows: Vec<Row>,
}

impl Drop for OutlineState {
    fn drop(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}

impl OutlineState {
    pub(super) fn owns(&self, document: &str, cancel: &Cancellation) -> bool {
        self.document.as_deref() == Some(document)
            && !cancel.is_cancelled()
            && self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.same_request(cancel))
    }
    pub(super) fn cancel(&mut self) {
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
    }
    pub(super) fn destination(
        &self,
        index: usize,
    ) -> Option<yamantaka_domain::pdf_destination::NavigationDestination> {
        self.outline.as_ref()?.entries().get(index)?.destination
    }
    pub(super) fn toggle(&mut self, index: usize) {
        if !self.collapsed.remove(&index) {
            self.collapsed.insert(index);
        }
        self.rebuild_rows();
    }
    pub(super) fn capture_summary(&self) -> serde_json::Value {
        serde_json::json!({"entries":self.outline.as_ref().map_or(0, |outline| outline.entries().len()), "rows":self.rows.len(), "pending":self.pending.is_some()})
    }
    pub(super) fn ready(&self) -> bool {
        self.outline.is_some() && self.pending.is_none()
    }

    pub(super) fn accept(
        &mut self,
        document: &str,
        cancel: &Cancellation,
        result: Result<Arc<PdfOutline>, String>,
    ) {
        if cancel.is_cancelled()
            || self.document.as_deref() != Some(document)
            || !self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.same_request(cancel))
        {
            return;
        }
        self.pending = None;
        match result {
            Ok(outline) => {
                self.outline = Some(outline);
                self.error = None;
                self.collapsed.clear();
                self.rebuild_rows();
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn rebuild_rows(&mut self) {
        self.rows.clear();
        let Some(outline) = &self.outline else { return };
        let entries = outline.entries();
        let mut depths = vec![0usize; entries.len()];
        let mut hidden = vec![false; entries.len()];
        let mut children = vec![false; entries.len()];
        for entry in entries {
            if let Some(parent) = entry.parent {
                children[parent] = true;
            }
        }
        for (index, entry) in entries.iter().enumerate() {
            if let Some(parent) = entry.parent {
                depths[index] = depths[parent] + 1;
                hidden[index] = hidden[parent] || self.collapsed.contains(&parent);
            }
            if !hidden[index] {
                self.rows.push(Row {
                    entry: index,
                    depth: depths[index],
                    children: children[index],
                });
            }
        }
    }
}

impl YamantakaApp {
    pub fn preview_outline(&mut self, target: Option<usize>) {
        if self.demo && self.capture.is_some() {
            self.reader.panel = true;
            self.reader.reader_panel_tab = ReaderPanelTab::Outline;
            self.outline_preview_target = target;
        }
    }
    pub(super) fn apply_outline_preview(&mut self) {
        if !self.reader.outline.ready() {
            return;
        }
        if let Some(index) = self.outline_preview_target.take()
            && let Some(destination) = self
                .reader
                .outline
                .outline
                .as_ref()
                .and_then(|outline| outline.entries().get(index))
                .and_then(|entry| entry.destination)
        {
            self.choose_reader_destination(destination, false);
        }
    }

    fn choose_reader_destination(
        &mut self,
        destination: yamantaka_domain::pdf_destination::NavigationDestination,
        compact: bool,
    ) {
        if let Some(document) = self.reader.outline.document.clone() {
            self.reader.reader_destination = Some((document, destination));
            if compact {
                self.reader.panel = false;
            }
        }
    }

    pub(super) fn outline_panel_content(&mut self, ui: &mut egui::Ui, p: Palette) {
        let compact = ui.ctx().content_rect().width() < 620.0;
        if let Some(destination) =
            self.reader
                .outline
                .show(ui, self.reader.page, p, self.english, compact, None)
        {
            self.choose_reader_destination(destination, compact);
        }
    }
}
