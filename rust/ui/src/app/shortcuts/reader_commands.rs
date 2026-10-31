use super::{ReadingTarget, Shortcut};
use crate::app::*;
pub(in crate::app) struct PageJump {
    document: String,
    target: ReadingTarget,
    pub(super) value: String,
    count: u32,
    focus: bool,
}
impl YamantakaApp {
    pub(in crate::app) fn toggle_immersive_reader(&mut self, ctx: &egui::Context) {
        self.reader_focus = !self.reader_focus;
        self.reader_chrome_hidden = self.reader_focus;
        egui::Popup::close_all(ctx);
        if self.reader_focus {
            self.reader.panel = false;
            self.reader.find.stop();
            if let Some(reference) = &mut self.reference {
                reference.stop_navigator_requests();
                reference.pane.panel = false;
            }
        }
        ctx.request_repaint();
    }
    pub(in crate::app) fn dispatch_legacy_reader_command(
        &mut self,
        ctx: &egui::Context,
        command: Shortcut,
    ) -> bool {
        if !matches!(
            command,
            Shortcut::GoToPage
                | Shortcut::Navigator
                | Shortcut::TogglePanel
                | Shortcut::ToggleBookmark
                | Shortcut::ToggleChrome
                | Shortcut::ToggleImmersive
        ) {
            return false;
        }
        if !matches!(self.surface, Surface::Reader) {
            return true;
        }
        if command == Shortcut::ToggleImmersive {
            self.toggle_immersive_reader(ctx);
            return true;
        }
        if command == Shortcut::ToggleChrome {
            self.reader_chrome_hidden = !self.reader_chrome_hidden;
            egui::Popup::close_all(ctx);
            if let Some(reference) = &mut self.reference {
                reference.stop_navigator_requests();
                reference.pane.panel = false;
            }
            return true;
        }
        let reference = self.reading_focus.reference();
        if reference && self.reference.as_ref().is_none_or(|view| view.busy()) {
            return true;
        }
        let pane = if reference {
            &mut self.reference.as_mut().unwrap().pane
        } else {
            &mut self.reader
        };
        let Some((document, info)) = &pane.active else {
            return true;
        };
        match command {
            Shortcut::GoToPage => {
                self.page_jump = Some(PageJump {
                    document: document.id.clone(),
                    target: if reference {
                        ReadingTarget::Reference
                    } else {
                        ReadingTarget::Primary
                    },
                    value: pane.page.saturating_add(1).to_string(),
                    count: info.pages.len() as u32,
                    focus: true,
                });
                egui::Popup::close_all(ctx);
                self.reading_focus.cancel_canvas_request();
            }
            Shortcut::ToggleBookmark => {
                pane.bookmarks
                    .toggle(&self.runtime, &document.id, pane.page)
            }
            Shortcut::Navigator | Shortcut::TogglePanel => {
                self.reader_focus = false;
                self.reader_chrome_hidden = false;
                if reference {
                    let reference = self.reference.as_mut().unwrap();
                    if command == Shortcut::TogglePanel && reference.pane.panel {
                        egui::Popup::close_all(ctx);
                        reference.stop_navigator_requests();
                        reference.pane.panel = false;
                    } else {
                        reference.pane.reader_panel_tab = outline::ReaderPanelTab::Thumbnails;
                        reference.navigator_requested = true;
                    }
                } else if command == Shortcut::Navigator {
                    self.reader.panel = true;
                    self.reader.reader_panel_tab = outline::ReaderPanelTab::Thumbnails;
                } else {
                    self.toggle_reader_panel();
                }
            }
            _ => unreachable!(),
        }
        ctx.request_repaint();
        true
    }
    pub(in crate::app) fn page_jump_dialog(&mut self, ctx: &egui::Context) {
        let Some(jump) = &mut self.page_jump else {
            return;
        };
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let p = Palette::new(self.dark);
        let mut submit = false;
        let mut cancel = false;
        let preview =
            self.demo && self.capture.is_some() && self.reference_shortcuts_preview.phase() == 50;
        let modal = egui::Modal::new(egui::Id::new("reader-page-jump")).show(ctx, |ui| {
            ui.set_width((ctx.content_rect().width() - 48.0).clamp(120.0, 360.0));
            egui::ScrollArea::vertical()
                .max_height((ctx.content_rect().height() - 48.0).max(80.0))
                .show(ui, |ui| {
                    theme::display(ui, tr("Ir a una página", "Go to page"), 24.0);
                    ui.label(format!(
                        "{} · 1–{}",
                        if jump.target == ReadingTarget::Reference {
                            tr("Referencia", "Reference")
                        } else {
                            tr("Lectura principal", "Main reading")
                        },
                        jump.count
                    ));
                    let field = ui.add(
                        egui::TextEdit::singleline(&mut jump.value)
                            .char_limit(10)
                            .desired_width(f32::INFINITY),
                    );
                    field.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::TextEdit,
                            ui.is_enabled(),
                            tr("Página", "Page"),
                        )
                    });
                    if jump.focus {
                        field.request_focus();
                        jump.focus = false;
                    }
                    submit =
                        field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    ui.add_space(8.0);
                    let valid = jump
                        .value
                        .trim()
                        .parse::<u32>()
                        .ok()
                        .is_some_and(|page| page > 0 && page <= jump.count);
                    if !valid {
                        ui.label(tr(
                            "Introduce una página dentro del intervalo.",
                            "Enter a page within the range.",
                        ));
                    }
                    let actions = ui.horizontal_wrapped(|ui| {
                        submit |= ui
                            .add_enabled_ui(valid, |ui| {
                                theme::navigation(ui, tr("Ir a la página", "Go to page"), false, p)
                            })
                            .inner
                            .clicked();
                        if theme::navigation(ui, tr("Cancelar", "Cancel"), false, p).clicked() {
                            cancel = true;
                        }
                    });
                    if preview {
                        actions.response.scroll_to_me(Some(egui::Align::Center));
                    }
                    if submit && !valid {
                        jump.focus = true;
                    }
                    submit &= valid;
                });
        });
        if cancel || modal.should_close() {
            self.page_jump = None;
        } else if submit {
            self.confirm_page_jump();
        }
    }
    pub(in crate::app) fn confirm_page_jump(&mut self) {
        if !matches!(self.surface, Surface::Reader) {
            self.page_jump = None;
            return;
        }
        let Some(jump) = self.page_jump.take() else {
            return;
        };
        let Some(page) = jump
            .value
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|page| *page > 0 && *page <= jump.count)
        else {
            return;
        };
        if jump.target == ReadingTarget::Reference {
            if let Some(reference) = &mut self.reference
                && !reference.busy()
                && reference
                    .pane
                    .active
                    .as_ref()
                    .is_some_and(|(document, info)| {
                        document.id == jump.document && page as usize <= info.pages.len()
                    })
            {
                reference.set_page(page - 1);
            }
        } else if self.reader.active.as_ref().is_some_and(|(document, info)| {
            document.id == jump.document && page as usize <= info.pages.len()
        }) {
            self.navigate(page - 1);
        }
    }
}
