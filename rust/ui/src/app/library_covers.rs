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
    source_page: Option<u32>,
}
impl LibraryCovers {
    pub(super) fn slots_ready(&self) -> bool {
        self.slots.values().all(|slot| slot.finished)
    }
}

impl YamantakaApp {
    pub(super) fn pump_library_covers(&mut self, ctx: &egui::Context) {
        if self.cover_picker.is_some() {
            return;
        }
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
                },
            )
        });
        self.library_covers.generation = self.library_covers.generation.saturating_add(1);
        self.library_covers.slots.insert(
            document.id.clone(),
            CoverSlot {
                path: document.path.clone(),
                fingerprint: document.fingerprint.clone(),
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
        }
        ctx.data_mut(|data| data.remove::<book::PdfCover>(book::cover_id(id)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_application::manual_cover::ManualCoverRequest;
    use yamantaka_domain::{LibraryRepository, cover_selection::ManualCover};
    use yamantaka_pdf::{MemoryLimits, PdfWorker};
    use yamantaka_storage::Database;

    #[test]
    fn native_cover_preview_cancel_and_save_round_trip_keep_scopes_and_original_pdf() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = root
            .join("build/rust-fixtures/textual.pdf")
            .canonicalize()
            .unwrap();
        let original = std::fs::read(&path).unwrap();
        let db = Database::in_memory_at(11).unwrap();
        let escaped = path.to_string_lossy().replace('\'', "''");
        db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('picker','{escaped}','synthetic','synthetic.pdf',2,1,'now','now','now','synthetic');")).unwrap();
        let document = db.get("picker").unwrap().unwrap();
        let engine = PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap();
        let runtime = Runtime::start(db, engine, || {}).unwrap();
        let preview = Cancellation::default();
        runtime
            .submit(Action::PreviewCover(
                "picker".into(),
                path.clone(),
                1,
                19,
                preview.clone(),
            ))
            .unwrap();
        assert!(
            matches!(runtime.updates.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
            Update::Cover(id,Ok(page),token) if id=="picker" && page.key.page==1 && page.key.generation==19 && token.same_request(&preview))
        );
        let cancelled = Cancellation::default();
        cancelled.cancel();
        runtime
            .submit(Action::SaveManualCover(Box::new(ManualCoverRequest {
                choice: ManualCover::new(&document, 1, 4.0 / 3.0).unwrap(),
                cancel: cancelled.clone(),
            })))
            .unwrap();
        assert!(
            matches!(runtime.updates.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
            Update::ManualCover(reply) if reply.document=="picker" && matches!(reply.result,Err(yamantaka_domain::Failure::Cancelled)) && reply.cancel.same_request(&cancelled))
        );
        let lookup = Cancellation::default();
        runtime
            .submit(Action::CoverSourcePage("picker".into(), lookup.clone()))
            .unwrap();
        assert!(
            matches!(runtime.updates.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
            Update::CoverSourcePage(id,Ok(None),token) if id=="picker" && token.same_request(&lookup))
        );
        let save = Cancellation::default();
        runtime
            .submit(Action::SaveManualCover(Box::new(ManualCoverRequest {
                choice: ManualCover::new(&document, 1, 4.0 / 3.0).unwrap(),
                cancel: save.clone(),
            })))
            .unwrap();
        let Update::ManualCover(reply) = runtime
            .updates
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap()
        else {
            panic!("Expected a scoped manual cover reply")
        };
        save.cancel();
        assert_eq!(reply.document, "picker");
        assert!(reply.result.is_ok());
        assert!(reply.cancel.same_request(&save));
        runtime
            .submit(Action::Cover(
                "picker".into(),
                path.clone(),
                20,
                Cancellation::default(),
            ))
            .unwrap();
        assert!(
            matches!(runtime.updates.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
            Update::Cover(id,Ok(page),_) if id=="picker" && page.key.page==1 && page.width==512 && page.height==384)
        );
        drop(runtime);
        assert_eq!(std::fs::read(path).unwrap(), original);
    }

    #[test]
    fn native_actor_renders_the_saved_manual_page_without_a_reader_open() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = root
            .join("build/rust-fixtures/textual.pdf")
            .canonicalize()
            .unwrap();
        let before = std::fs::read(&path).unwrap();
        let db = Database::in_memory_at(11).unwrap();
        db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('saved','synthetic.pdf','synthetic','synthetic.pdf',2,1,'now','now','now','synthetic');
            INSERT INTO document_covers VALUES('saved',1,'preserved-small','preserved-large','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z',1,'manual',4,'contain',0.75);").unwrap();
        let worker = PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap();
        let runtime = Runtime::start(db, worker, || {}).unwrap();
        let cancel = Cancellation::default();
        runtime
            .submit(Action::Cover(
                "saved".into(),
                path.clone(),
                41,
                cancel.clone(),
            ))
            .unwrap();
        assert!(
            matches!(runtime.updates.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
            Update::Cover(id,Ok(page),token) if id=="saved" && page.key.page==1
                && page.key.generation==41 && page.width==512 && page.height==384
                && token.same_request(&cancel))
        );
        drop(runtime);
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}
