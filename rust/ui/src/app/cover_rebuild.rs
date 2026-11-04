use super::*;
use yamantaka_application::cover_rebuild::{Reply, Request};
#[derive(Default)]
pub(super) enum State {
    #[default]
    Idle,
    Pending {
        document: String,
        cancel: Cancellation,
    },
    Done {
        document: String,
        result: yamantaka_domain::Result<()>,
    },
}
impl YamantakaApp {
    pub(super) fn begin_cover_rebuild(&mut self, document: Document) {
        if matches!(self.cover_rebuild, State::Pending { .. }) {
            return;
        }
        let id = document.id.clone();
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::RebuildCover(Box::new(Request {
            document,
            automatic: None,
            governor: self.governor.clone(),
            cancel: cancel.clone(),
        }))) {
            Ok(()) => {
                self.cover_rebuild = State::Pending {
                    document: id,
                    cancel,
                }
            }
            Err(error) => {
                self.cover_rebuild = State::Done {
                    document: id,
                    result: Err(error),
                }
            }
        }
    }
    pub(super) fn accept_cover_rebuilt(&mut self, reply: Reply, ctx: &egui::Context) {
        if !matches!(&self.cover_rebuild,State::Pending{document,cancel} if document==&reply.document&&cancel.same_request(&reply.cancel))
        {
            return;
        }
        if reply.result.is_ok() {
            self.invalidate_library_cover(ctx, &reply.document);
        }
        self.cover_rebuild = State::Done {
            document: reply.document,
            result: reply.result,
        };
    }
    pub(super) fn cover_rebuild_control(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        p: Palette,
    ) {
        if let State::Pending {
            document: id,
            cancel,
        } = &self.cover_rebuild
            && id == &document.id
        {
            if theme::bounded_navigation(ui, self.tr("Cancelar regeneración", "Cancel rebuild"), p)
                .clicked()
            {
                cancel.cancel();
                self.runtime.wake();
            }
            return;
        }
        if ui
            .add_enabled_ui(
                !document.missing
                    && !self.startup_mode.is_safe()
                    && !matches!(self.cover_rebuild, State::Pending { .. }),
                |ui| {
                    theme::bounded_navigation(
                        ui,
                        self.tr("Regenerar portada", "Regenerate cover"),
                        p,
                    )
                },
            )
            .inner
            .clicked()
        {
            self.begin_cover_rebuild(document.clone());
        }
        if let State::Done {
            document: id,
            result,
        } = &self.cover_rebuild
            && id == &document.id
        {
            match result {
                Ok(()) => {
                    ui.label(self.tr("Portada actualizada", "Cover updated"));
                }
                Err(yamantaka_domain::Failure::Cancelled) => {}
                Err(yamantaka_domain::Failure::NativeSlotsBusy) => {
                    ui.add(
                        egui::Label::new(self.tr(
                            "Cierra una lectura y vuelve a intentarlo.",
                            "Close a reading and retry.",
                        ))
                        .wrap(),
                    );
                }
                Err(_) => {
                    ui.label(self.tr(
                        "No se pudo regenerar. Comprueba el archivo y reintenta.",
                        "Could not rebuild. Check the file and retry.",
                    ));
                }
            }
        }
    }
}

impl YamantakaApp {
    pub fn preview_inspector_actions(&mut self) {
        if self.demo && self.capture.is_some() {
            self.inspector_actions_preview = true;
        }
    }
    pub fn preview_cover_rebuild(&mut self) {
        if self.demo && self.capture.is_some() {
            self.cover_rebuild_preview = true;
        }
    }
    pub(super) fn advance_cover_rebuild_preview(&mut self) {
        if self.cover_rebuild_preview
            && matches!(self.cover_rebuild, State::Idle)
            && let Some(document) = self.documents.iter().find(|d| d.id == "demo-0").cloned()
        {
            self.begin_cover_rebuild(document);
        }
    }
    pub(super) fn cover_rebuild_capture_ready(&self) -> bool {
        !self.cover_rebuild_preview
            || self.password.as_ref().is_some_and(|p| {
                p.background_purpose()
                    == Some(yamantaka_application::background_password::Purpose::Cover)
            })
    }
    pub(super) fn cover_rebuild_summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":matches!(self.cover_rebuild,State::Pending{..}),"success":matches!(self.cover_rebuild,State::Done{result:Ok(()),..}),
            "password":self.password.as_ref().is_some_and(|p|p.background_purpose()==Some(yamantaka_application::background_password::Purpose::Cover))})
    }
}
