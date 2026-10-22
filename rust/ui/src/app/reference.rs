use super::*;
use yamantaka_domain::reader::ReaderPosition;
use yamantaka_domain::{PageLayout, reader::ReaderMode};
mod admission;
mod chooser;
mod password;
mod preview;
mod saved;
use admission::Admission;

pub(super) struct ReferenceView {
    admission: Admission,
    close_requested: bool,
    password_attempted: bool,
    selection: Option<Document>,
    pub(super) chooser: chooser::ReferenceChooser,
    pub pane: reader_pane::ReaderPane,
    pub follow_page: bool,
    pub follow_zoom: bool,
    pub follow_position: bool,
    followed_position: Option<ReaderPosition>,
    sync_preview: bool,
    ratio: f32,
    size: egui::Vec2,
    error: Option<String>,
}

impl Drop for ReferenceView {
    fn drop(&mut self) {
        if let Admission::Opening { cancel, .. } = &self.admission {
            cancel.cancel();
        }
    }
}

impl ReferenceView {
    pub(in crate::app) fn new(
        source: &reader_pane::ReaderPane,
        direction: ReadingDirection,
    ) -> yamantaka_domain::Result<Self> {
        let (Some(active), Some(mut preferences), Some(navigation)) = (
            source.active.clone(),
            source.reader_preferences.clone(),
            source.navigation.clone(),
        ) else {
            return Err(yamantaka_domain::Failure::Invalid(
                "Open a reading before comparing pages".into(),
            ));
        };
        preferences.page_layout = PageLayout::Single;
        preferences.view_mode = ReaderMode::Document;
        preferences.zoom_mode = ZoomMode::FitPage;
        preferences.scroll_offset = 0.0;
        let mut pane = reader_pane::ReaderPane::default();
        pane.active = Some(active);
        pane.navigation = Some(navigation);
        pane.reader_preferences = Some(preferences);
        if let Some(navigation) = &mut pane.navigation {
            navigation.go_to_page((source.page + 1).min(navigation.page_count() - 1));
        }
        pane.navigation_changed();
        pane.rebuild_layout(direction)?;
        Ok(Self {
            admission: Admission::Ready,
            close_requested: false,
            password_attempted: false,
            selection: None,
            chooser: Default::default(),
            pane,
            follow_page: false,
            follow_zoom: false,
            follow_position: false,
            followed_position: None,
            sync_preview: false,
            ratio: 0.5,
            size: egui::Vec2::ZERO,
            error: None,
        })
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        bytes: usize,
        english: bool,
        p: Palette,
        can_choose: bool,
    ) -> bool {
        let tr = |es, en| if english { en } else { es };
        let mut close = false;
        if !can_choose && let Some((document, _)) = &self.pane.active {
            ui.add(
                egui::Label::new(
                    RichText::new(document.display_title())
                        .color(p.secondary)
                        .size(12.0),
                )
                .truncate(),
            );
        }
        if self.busy() {
            ui.horizontal_wrapped(|ui| {
                if let Some(error) = self.error.clone() {
                    ui.label(error);
                    if theme::navigation(ui, tr("Reintentar", "Retry"), false, p).clicked()
                        && let Admission::Confirming {
                            blocked, submitted, ..
                        } = &mut self.admission
                    {
                        *blocked = false;
                        *submitted = false;
                        self.error = None;
                    }
                }
                close = theme::navigation(ui, tr("Cerrar referencia", "Close reference"), false, p)
                    .clicked();
            });
            self.viewport(ui, runtime, bytes, english);
            return close;
        }
        let count = self
            .pane
            .navigation
            .as_ref()
            .map_or(1, ReaderNavigation::page_count);
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(can_choose, |ui| {
                let button = theme::icon_button(
                    ui,
                    tr("Elegir libro de referencia", "Choose reference book"),
                    theme::Icon::Book,
                    p,
                );
                if button.clicked() {
                    self.chooser.reopen();
                }
                let mut popup = egui::Popup::menu(&button)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
                if self.chooser.preview {
                    popup = popup.open(true);
                }
                popup.show(|ui| {
                    self.selection = self.chooser.show(
                        ui,
                        runtime,
                        english,
                        p,
                        self.pane.active.as_ref().map(|(doc, _)| doc.id.as_str()),
                    );
                });
            });
            let page_label = ui.label(
                RichText::new(tr("Referencia", "Reference"))
                    .color(p.secondary)
                    .size(12.0),
            );
            let mut page = self.pane.page + 1;
            if ui
                .add(
                    egui::DragValue::new(&mut page)
                        .range(1..=count)
                        .suffix(format!(" / {count}")),
                )
                .labelled_by(page_label.id)
                .changed()
                && let Some(nav) = &mut self.pane.navigation
            {
                nav.go_to_page(page - 1);
                self.pane.navigation_changed();
                self.follow_page = false;
                self.follow_position = false;
            }
            let out = theme::icon_button(
                ui,
                tr("Alejar referencia", "Zoom reference out"),
                theme::Icon::Minus,
                p,
            )
            .clicked();
            let zoom_in = theme::icon_button(
                ui,
                tr("Acercar referencia", "Zoom reference in"),
                theme::Icon::Plus,
                p,
            )
            .clicked();
            let fit = theme::navigation(ui, tr("Ajustar", "Fit"), false, p).clicked();
            if out || zoom_in {
                let zoom = (self.pane.viewport.scale * if zoom_in { 1.2 } else { 1.0 / 1.2 })
                    .clamp(0.2, 8.0);
                self.set_zoom(ZoomMode::Custom, zoom);
            }
            if fit {
                self.set_zoom(ZoomMode::FitPage, self.pane.zoom);
            }
            let linked = theme::navigation(
                ui,
                tr("Vincular", "Link views"),
                self.follow_page || self.follow_zoom || self.follow_position,
                p,
            );
            let mut popup = egui::Popup::menu(&linked)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
            if self.sync_preview {
                popup = popup.open(true);
            }
            popup.show(|ui| {
                ui.label(
                    RichText::new(tr("Desde la lectura principal", "From the main reading"))
                        .color(p.secondary)
                        .size(12.0),
                );
                ui.checkbox(&mut self.follow_page, tr("Seguir página", "Follow page"));
                ui.checkbox(&mut self.follow_zoom, tr("Seguir zoom", "Follow zoom"));
                ui.checkbox(
                    &mut self.follow_position,
                    tr("Seguir posición", "Follow position"),
                );
            });
            close = theme::navigation(ui, tr("Cerrar referencia", "Close reference"), false, p)
                .clicked();
        });
        if let Some(error) = self.error.clone() {
            ui.horizontal_wrapped(|ui| {
                ui.label(error);
                if theme::navigation(ui, tr("Reintentar", "Retry"), false, p).clicked() {
                    self.pane.viewport.reset_requests();
                    self.error = None;
                }
            });
        }
        self.viewport(ui, runtime, bytes, english);
        close
    }

    pub fn fail_render(&mut self, token: &Cancellation, error: &yamantaka_domain::Failure) -> bool {
        if token.is_cancelled()
            || !self
                .pane
                .viewport
                .pages
                .values()
                .any(|slot| slot.cancel.same_request(token))
        {
            return false;
        }
        self.error = Some(error.to_string());
        self.pane.viewport.retry = true;
        self.pane.loading = false;
        true
    }

    pub(in crate::app) fn set_zoom(&mut self, mode: ZoomMode, zoom: f32) {
        self.follow_zoom = false;
        self.followed_position = None;
        if let Some(prefs) = &mut self.pane.reader_preferences {
            self.pane.zoom = zoom;
            prefs.zoom_mode = mode;
            prefs.zoom_level = f64::from(zoom);
            self.pane.viewport.reset_requests();
        }
    }

    fn follow_primary_zoom(&mut self, scale: f32) {
        if !self.follow_zoom
            || self.busy()
            || self.close_requested
            || !scale.is_finite()
            || scale <= 0.0
        {
            return;
        }
        if let Some(prefs) = &mut self.pane.reader_preferences
            && (prefs.zoom_mode != ZoomMode::Custom || (self.pane.zoom - scale).abs() > 0.0001)
        {
            self.pane.zoom = scale;
            prefs.zoom_mode = ZoomMode::Custom;
            prefs.zoom_level = f64::from(scale);
            self.pane.viewport.reset_requests();
            self.followed_position = None;
        }
    }

    fn follow_primary_position(&mut self, position: Option<ReaderPosition>) {
        if !self.follow_position || self.busy() || self.close_requested {
            self.followed_position = None;
            return;
        }
        let Some(position) = position else {
            return;
        };
        if self.followed_position == Some(position) {
            return;
        }
        self.followed_position = Some(position);
        if let Some(nav) = &mut self.pane.navigation {
            let page_changed = nav.position().page() != position.page().min(nav.page_count() - 1);
            if nav.update_anchor(position) {
                if page_changed {
                    self.pane.navigation_changed();
                } else {
                    self.pane.viewport.scroll_anchor = true;
                }
            }
        }
    }

    fn viewport(&mut self, ui: &mut egui::Ui, runtime: &Runtime, bytes: usize, english: bool) {
        let (Some((document, _)), Some(layout), Some(preferences), Some(navigation)) = (
            &self.pane.active,
            self.pane.viewport.layout.clone(),
            self.pane.reader_preferences.clone(),
            &self.pane.navigation,
        ) else {
            return;
        };
        let document = document.clone();
        let anchor = navigation.position();
        let size = ui.available_size();
        let viewport = [f64::from(size.x.max(1.0)), f64::from(size.y.max(1.0))];
        ui.painter().rect_filled(
            ui.available_rect_before_wrap(),
            0,
            YamantakaApp::reading_paper(preferences.reading_theme),
        );
        let mut scroll = egui::ScrollArea::both().id_salt("reference-pages");
        if self.pane.viewport.scroll_anchor {
            if let Ok([x, y]) = layout.scroll_for_anchor(
                viewport,
                anchor,
                preferences.zoom_mode,
                f64::from(self.pane.zoom),
            ) {
                scroll = scroll.scroll_offset(egui::vec2(x as f32, y as f32));
            }
            self.pane.viewport.scroll_anchor = false;
        }
        let dpr = ui.ctx().pixels_per_point();
        let cap = ui.ctx().input(|input| input.max_texture_side).min(16384) as u32;
        scroll.show_viewport(ui, |ui, visible| {
            let frame = match layout.frame(
                viewport,
                anchor,
                preferences.zoom_mode,
                f64::from(self.pane.zoom),
                [f64::from(visible.left()), f64::from(visible.top())],
                0.0,
            ) {
                Ok(frame) => frame,
                Err(error) => {
                    self.error = Some(error.to_string());
                    return;
                }
            };
            self.pane.viewport.scale = frame.zoom as f32;
            ui.set_min_size(egui::vec2(
                frame.content_size[0] as f32,
                frame.content_size[1] as f32,
            ));
            self.pane.viewport.pages.retain(|page, slot| {
                let keep = frame.pages.iter().any(|visible| visible.page == *page);
                if !keep {
                    slot.cancel.cancel();
                }
                keep
            });
            let origin = ui.max_rect().min;
            for page in &frame.pages {
                let size = egui::vec2(page.rect.width as f32, page.rect.height as f32);
                let physical =
                    preview_dimensions(size, dpr, cap, (bytes / frame.pages.len().max(1)).max(4));
                let key = RenderKey {
                    document_id: document.id.clone(),
                    page: page.page,
                    region: None,
                    physical_width: physical[0],
                    physical_height: physical[1],
                    rotation: layout.rotation().degrees(),
                    generation: self.pane.viewport.generation,
                };
                let paper = egui::Rect::from_min_size(
                    origin + egui::vec2(page.rect.left as f32, page.rect.top as f32),
                    size,
                );
                let label = format!(
                    "{} · {} {}",
                    document.display_title(),
                    if english {
                        "reference page"
                    } else {
                        "página de referencia"
                    },
                    page.page + 1
                );
                if let Err(error) =
                    self.pane
                        .viewport
                        .paint_page(ui, runtime, &key, paper, &preferences, label)
                {
                    self.error = Some(error.to_string());
                }
            }
            self.pane.loading = !self.pane.viewport.retry
                && self
                    .pane
                    .viewport
                    .pages
                    .values()
                    .any(|slot| !slot.plan.ready(slot.image_key.as_ref()));
            if let Ok(position) = layout.anchor_at_scroll(&frame, anchor)
                && let Some(nav) = &mut self.pane.navigation
            {
                nav.update_anchor(position);
                self.pane.page = nav.position().page();
            }
        });
    }
}

impl YamantakaApp {
    pub fn preview_reference(&mut self) {
        self.preview_reference = self.demo;
    }
    pub(super) fn toggle_reference(&mut self) {
        self.reader.viewport.scroll_anchor = true;
        if self.reference.is_some() {
            self.request_reference_close();
            return;
        }
        if self.presenter.is_none() {
            return;
        }
        match ReferenceView::new(&self.reader, self.reading_direction()) {
            Ok(reference) => self.reference = Some(reference),
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub(super) fn comparison_viewport(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        info: &PdfInfo,
        p: Palette,
    ) {
        let Some(reference) = &mut self.reference else {
            return;
        };
        if !reference.busy()
            && !reference.close_requested
            && reference.follow_page
            && !reference.follow_position
            && let Some(nav) = &mut reference.pane.navigation
            && nav.position().page() != self.reader.page.min(nav.page_count() - 1)
            && nav.go_to_page(self.reader.page.min(nav.page_count() - 1))
        {
            reference.pane.navigation_changed();
        }
        let rect = ui.available_rect_before_wrap();
        if reference.size != rect.size() {
            reference.size = rect.size();
            reference.followed_position = None;
            self.reader.viewport.scroll_anchor = true;
            reference.pane.viewport.scroll_anchor = true;
        }
        let stacked = rect.width() < 960.0;
        let extent = if stacked { rect.height() } else { rect.width() };
        let split = extent * reference.ratio;
        let mut first = rect;
        let mut second = rect;
        if stacked {
            first.max.y = rect.top() + split - 4.0;
            second.min.y = first.bottom() + 8.0;
        } else {
            first.max.x = rect.left() + split - 4.0;
            second.min.x = first.right() + 8.0;
        }
        let divider = if stacked {
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), first.bottom()),
                egui::pos2(rect.right(), second.top()),
            )
        } else {
            egui::Rect::from_min_max(
                egui::pos2(first.right(), rect.top()),
                egui::pos2(second.left(), rect.bottom()),
            )
        };
        let drag = ui.interact(
            divider,
            ui.id().with("reference-divider"),
            egui::Sense::drag(),
        );
        drag.clone().on_hover_cursor(if stacked {
            egui::CursorIcon::ResizeVertical
        } else {
            egui::CursorIcon::ResizeHorizontal
        });
        if drag.dragged() {
            reference.set_ratio(
                reference.ratio
                    + if stacked {
                        drag.drag_delta().y
                    } else {
                        drag.drag_delta().x
                    } / extent.max(1.0),
            );
        }
        ui.painter().line_segment(
            if stacked {
                [
                    egui::pos2(rect.left(), divider.center().y),
                    egui::pos2(rect.right(), divider.center().y),
                ]
            } else {
                [
                    egui::pos2(divider.center().x, rect.top()),
                    egui::pos2(divider.center().x, rect.bottom()),
                ]
            },
            egui::Stroke::new(1.0, p.line),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(first)
                .id_salt("reading-pane"),
            |ui| {
                ui.set_clip_rect(first.intersect(ui.clip_rect()));
                self.reader_viewport(ui, document, info, p);
            },
        );
        let bytes = self.governor.limits().pages.min(14 * 1024 * 1024) / 2;
        let mut close = false;
        let mut selected = None;
        let can_choose = self.reference.as_ref().is_some_and(|reference| {
            reference.pane.active.as_ref().map(|(doc, _)| &doc.id)
                == self.reader.active.as_ref().map(|(doc, _)| &doc.id)
        });
        if let Some(reference) = &mut self.reference {
            reference.follow_primary_zoom(self.reader.viewport.scale);
            reference
                .follow_primary_position(self.reader.navigation.as_ref().map(|nav| nav.position()));
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(second)
                    .id_salt("reference-pane"),
                |ui| {
                    ui.set_clip_rect(second.intersect(ui.clip_rect()));
                    close = reference.show(ui, &self.runtime, bytes, self.english, p, can_choose);
                    selected = reference.selection.take();
                },
            );
        }
        if close {
            self.request_reference_close();
        } else if drag.dragged() {
            self.reader.viewport.scroll_anchor = true;
            if let Some(reference) = &mut self.reference {
                reference.pane.viewport.scroll_anchor = true;
                reference.followed_position = None;
            }
        }
        ui.allocate_rect(rect, egui::Sense::hover());
        if let Some(document) = selected {
            self.open_reference(document);
        }
    }
}
