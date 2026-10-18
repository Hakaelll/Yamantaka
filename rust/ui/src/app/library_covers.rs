use super::*;
use yamantaka_application::cover_source::CoverSource;

#[derive(Default)]
pub(super) struct LibraryCovers {
    slots: HashMap<String, CoverSlot>,
    generation: u64,
    capacity_preview: u8,
    blocked_before_release: usize,
}

struct CoverSlot {
    path: std::path::PathBuf,
    fingerprint: String,
    page_count: u32,
    generation: u64,
    cancel: Cancellation,
    image: Option<PageImage>,
    finished: bool,
    source_page: Option<u32>,
}
impl LibraryCovers {
    pub(super) fn slots_ready(&self) -> bool {
        self.slots.values().all(|slot| slot.finished) && matches!(self.capacity_preview, 0 | 3)
    }
}
impl YamantakaApp {
    pub fn preview_cover_capacity(&mut self) {
        if self.demo && self.capture.is_some() {
            self.library_covers.capacity_preview = 1;
        }
    }
    pub(super) fn pump_library_covers(&mut self, ctx: &egui::Context) {
        if self.cover_picker.is_some() {
            return;
        }
        let requested = ctx.data_mut(|data| {
            data.remove_temp::<Vec<Document>>(egui::Id::new("pdf-cover-requests"))
                .unwrap_or_default()
        });
        let mut cancelled = false;
        self.library_covers.slots.retain(|id, slot| {
            let keep = requested.iter().any(|doc| {
                &doc.id == id
                    && doc.path == slot.path
                    && doc.fingerprint == slot.fingerprint
                    && doc.page_count == slot.page_count
                    && !doc.missing
            });
            if !keep {
                slot.cancel.cancel();
                cancelled = true;
                ctx.data_mut(|data| data.remove::<book::PdfCover>(book::cover_id(id)));
            }
            keep
        });
        if cancelled {
            self.runtime.wake();
        }
        for document in requested {
            if document.missing || self.library_covers.slots.contains_key(&document.id) {
                continue;
            }
            let Some(generation) = self.library_covers.generation.checked_add(1) else {
                continue;
            };
            let cancel = Cancellation::default();
            match self.runtime.submit(Action::Cover(
                CoverSource::capture(&document),
                generation,
                cancel.clone(),
            )) {
                Ok(()) => {
                    self.library_covers.generation = generation;
                    self.library_covers.slots.insert(
                        document.id,
                        CoverSlot {
                            path: document.path,
                            fingerprint: document.fingerprint,
                            page_count: document.page_count,
                            generation,
                            cancel,
                            image: None,
                            finished: false,
                            source_page: None,
                        },
                    );
                }
                Err(yamantaka_domain::Failure::QueueFull) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100))
                }
                Err(_) => {}
            }
        }
        if self.library_covers.capacity_preview == 1
            && !self.library_covers.slots.is_empty()
            && self.runtime.waiting_covers() == self.library_covers.slots.len()
        {
            self.library_covers.blocked_before_release = self.library_covers.slots.len();
            self.library_covers.capacity_preview = 2;
            self.send(Action::VisibleDocuments(Vec::new()));
        } else if self.library_covers.capacity_preview == 2
            && self
                .library_covers
                .slots
                .values()
                .all(|slot| slot.image.is_some())
        {
            self.library_covers.capacity_preview = 3;
        }
    }
    pub(super) fn accept_library_cover(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        result: yamantaka_domain::Result<Arc<yamantaka_domain::RenderedPage>>,
        cancel: &Cancellation,
    ) {
        let Some(slot) = self.library_covers.slots.get_mut(id) else {
            return;
        };
        if cancel.is_cancelled() || !slot.cancel.same_request(cancel) || slot.finished {
            return;
        }
        slot.finished = true;
        let Ok(page) = result else {
            return;
        };
        if page.key.document_id != id || page.key.generation != slot.generation {
            return;
        }
        // The native presenter reserves CPU staging and GPU payload before upload.
        // Keep a static jacket on the alternate compositor until it has governed uploads.
        let Some(presenter) = &mut self.presenter else {
            return;
        };
        if let Ok(texture) = presenter.upload(&page) {
            slot.source_page = Some(page.key.page);
            ctx.data_mut(|data| {
                data.insert_temp(
                    book::cover_id(id),
                    book::PdfCover {
                        texture: texture.id(),
                        size: [page.width, page.height],
                        path: slot.path.clone(),
                        fingerprint: slot.fingerprint.clone(),
                        page_count: slot.page_count,
                    },
                )
            });
            slot.image = Some(PageImage::Native(texture));
        }
    }
    pub(super) fn cover_summary(&self) -> serde_json::Value {
        serde_json::json!({"resident": self.library_covers.slots.len(),
            "blockedBeforeRelease":self.library_covers.blocked_before_release,"capacityPreview":self.library_covers.capacity_preview,"capacityRevision":self.runtime.pdf_capacity_revision(),
            "ready":self.library_covers.slots.values().filter(|slot| slot.image.is_some()).count(),
            "pending":self.library_covers.slots.values().filter(|slot| !slot.finished).count(),
            "source":"PDF selected page","sourcePages":self.library_covers.slots.iter().filter_map(|(id,slot)| slot.source_page.map(|page|(id.clone(),page))).collect::<std::collections::BTreeMap<_,_>>(),"maximumVisible":16})
    }
    pub(super) fn install_selected_cover(
        &mut self,
        ctx: &egui::Context,
        document: &Document,
        page: u32,
        aspect: f64,
        image: PageImage,
    ) {
        if let Some(old) = self.library_covers.slots.remove(&document.id) {
            old.cancel.cancel();
            self.runtime.wake();
        }
        let dimensions = if aspect > 1.0 {
            [512, (512.0 / aspect).round() as u32]
        } else {
            [(512.0 * aspect).round() as u32, 512]
        };
        ctx.data_mut(|data| {
            data.insert_temp(
                book::cover_id(&document.id),
                book::PdfCover {
                    texture: image.id(),
                    size: dimensions,
                    path: document.path.clone(),
                    fingerprint: document.fingerprint.clone(),
                    page_count: document.page_count,
                },
            )
        });
        self.library_covers.generation = self.library_covers.generation.saturating_add(1);
        self.library_covers.slots.insert(
            document.id.clone(),
            CoverSlot {
                path: document.path.clone(),
                fingerprint: document.fingerprint.clone(),
                page_count: document.page_count,
                generation: self.library_covers.generation,
                cancel: Cancellation::default(),
                image: Some(image),
                finished: true,
                source_page: Some(page),
            },
        );
    }
    pub(super) fn invalidate_library_cover(&mut self, ctx: &egui::Context, id: &str) {
        if let Some(slot) = self.library_covers.slots.remove(id) {
            slot.cancel.cancel();
            self.runtime.wake();
        }
        ctx.data_mut(|data| data.remove::<book::PdfCover>(book::cover_id(id)));
    }
}
