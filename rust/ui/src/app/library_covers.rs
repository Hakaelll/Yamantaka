use super::*;

#[derive(Default)]
pub(super) struct LibraryCovers {
    slots: HashMap<String, CoverSlot>,
    generation: u64,
}
struct CoverSlot {
    path: std::path::PathBuf,
    fingerprint: String,
    generation: u64,
    cancel: Cancellation,
    image: Option<PageImage>,
    finished: bool,
}
impl LibraryCovers {
    pub(super) fn slots_ready(&self) -> bool {
        self.slots.values().all(|slot| slot.finished)
    }
}

impl YamantakaApp {
    pub(super) fn pump_library_covers(&mut self, ctx: &egui::Context) {
        let requested = ctx.data_mut(|data| {
            data.remove_temp::<Vec<Document>>(egui::Id::new("pdf-cover-requests"))
                .unwrap_or_default()
        });
        self.library_covers.slots.retain(|id, slot| {
            let keep = requested.iter().any(|doc| {
                &doc.id == id && doc.path == slot.path && doc.fingerprint == slot.fingerprint
            });
            if !keep {
                slot.cancel.cancel();
                ctx.data_mut(|data| data.remove::<book::PdfCover>(book::cover_id(id)));
            }
            keep
        });
        for document in requested {
            if document.missing || self.library_covers.slots.contains_key(&document.id) {
                continue;
            }
            let Some(generation) = self.library_covers.generation.checked_add(1) else {
                continue;
            };
            let cancel = Cancellation::default();
            match self.runtime.submit(Action::Cover(
                document.id.clone(),
                document.path.clone(),
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
                            generation,
                            cancel,
                            image: None,
                            finished: false,
                        },
                    );
                }
                Err(yamantaka_domain::Failure::QueueFull) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100))
                }
                Err(_) => {}
            }
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
        if page.key.document_id != id
            || page.key.generation != slot.generation
            || page.key.page != 0
        {
            return;
        }
        // The native presenter reserves CPU staging and GPU payload before upload.
        // Keep a static jacket on the alternate compositor until it has governed uploads.
        let Some(presenter) = &mut self.presenter else {
            return;
        };
        if let Ok(texture) = presenter.upload(&page) {
            ctx.data_mut(|data| {
                data.insert_temp(
                    book::cover_id(id),
                    book::PdfCover {
                        texture: texture.id(),
                        size: [page.width, page.height],
                        path: slot.path.clone(),
                        fingerprint: slot.fingerprint.clone(),
                    },
                )
            });
            slot.image = Some(PageImage::Native(texture));
        }
    }
    pub(super) fn cover_summary(&self) -> serde_json::Value {
        serde_json::json!({"resident": self.library_covers.slots.len(),
            "ready":self.library_covers.slots.values().filter(|slot| slot.image.is_some()).count(),
            "pending":self.library_covers.slots.values().filter(|slot| !slot.finished).count(),
            "source":"PDF page 1","maximumVisible":16})
    }
}
