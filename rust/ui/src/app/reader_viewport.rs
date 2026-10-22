use super::*;
use yamantaka_domain::PageLayout;

impl YamantakaApp {
    pub(super) fn reading_direction(&self) -> ReadingDirection {
        if self
            .preferences
            .get("readingDirection")
            .and_then(serde_json::Value::as_str)
            == Some("rightToLeft")
        {
            ReadingDirection::Rtl
        } else {
            ReadingDirection::Ltr
        }
    }

    pub(super) fn rebuild_reader_layout(&mut self) {
        let (Some((_, info)), Some(preferences)) = (&self.active, &self.reader_preferences) else {
            return;
        };
        let direction = self.reading_direction();
        match ReaderLayout::new(
            info.pages.iter().map(|page| page.size()),
            preferences,
            direction,
            24.0,
        ) {
            Ok(layout) => {
                if let Some(navigation) = &mut self.navigation {
                    navigation.apply_preferences(preferences, direction);
                }
                self.viewport.layout = Some(Arc::new(layout));
                self.viewport.scroll_anchor = true;
            }
            Err(error) => {
                self.viewport.layout = None;
                self.error = Some(error.to_string());
            }
        }
    }

    pub(super) fn step_reader(&mut self, forward: bool) {
        if self.navigation.as_mut().is_some_and(|navigation| {
            if forward {
                navigation.next_page()
            } else {
                navigation.previous()
            }
        }) {
            self.apply_navigation();
        }
    }

    pub(super) fn reader_viewport(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        info: &PdfInfo,
        p: Palette,
    ) {
        let viewport = ui.available_size();
        let viewport_size = [
            f64::from(viewport.x.max(1.0)),
            f64::from(viewport.y.max(1.0)),
        ];
        self.apply_reader_destination(document, info, viewport_size);
        let (Some(layout), Some(preferences), Some(navigation)) = (
            self.viewport.layout.clone(),
            self.reader_preferences.clone(),
            self.navigation.as_ref(),
        ) else {
            ui.label(
                RichText::new(self.tr("Preparando el documento…", "Preparing your document…"))
                    .color(p.secondary),
            );
            return;
        };
        let anchor = navigation.position();
        ui.painter()
            .rect_filled(ui.available_rect_before_wrap(), 0, self.reader_canvas(p));
        let mut scrolling = egui::ScrollArea::both().id_salt(("pdf-viewport", &document.id));
        if self.viewport.scroll_anchor {
            match layout.scroll_for_anchor(
                viewport_size,
                anchor,
                preferences.zoom_mode,
                f64::from(self.zoom),
            ) {
                Ok([x, y]) => {
                    let y = self.restoration.scroll.take().unwrap_or(y);
                    scrolling = scrolling.scroll_offset(egui::vec2(x as f32, y as f32));
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    return;
                }
            }
            self.viewport.scroll_anchor = false;
        }
        let dpr = ui.ctx().pixels_per_point();
        let device_limit = ui.ctx().input(|input| input.max_texture_side).min(16384) as u32;
        let mut feedback = None;
        scrolling.show_viewport(ui, |ui, visible| {
            if let Some(preferences) = &mut self.reader_preferences {
                preferences.scroll_offset = f64::from(visible.top()).max(0.0);
            }
            let frame = match layout.frame(
                viewport_size,
                anchor,
                preferences.zoom_mode,
                f64::from(self.zoom),
                [f64::from(visible.left()), f64::from(visible.top())],
                0.0,
            ) {
                Ok(frame) => frame,
                Err(error) => {
                    self.error = Some(error.to_string());
                    return;
                }
            };
            self.viewport.scale = frame.zoom as f32;
            ui.set_min_size(egui::vec2(
                frame.content_size[0] as f32,
                frame.content_size[1] as f32,
            ));
            // Only intersecting pages retain presentation leases. Cancelling a
            // consumer never cancels another ticket that shares its native render.
            self.viewport.pages.retain(|page, slot| {
                let keep = frame.pages.iter().any(|visible| visible.page == *page);
                if !keep {
                    slot.cancel.cancel();
                }
                keep
            });
            let link_pages: Vec<u32> = frame.pages.iter().map(|visible| visible.page).collect();
            self.request_visible_links(&document.id, &link_pages);
            self.request_annotation_marks(&document.id, &link_pages);
            self.selection.retain_visible(&document.id, &link_pages);
            self.request_selection_pages(&document.id, &link_pages);
            let origin = ui.max_rect().min;
            let count = frame.pages.len().max(1);
            let preview_budget =
                (self.governor.limits().pages.min(14 * 1024 * 1024) / count).max(4);
            let page_label = self.tr("página", "page");
            for visible in &frame.pages {
                let displayed = egui::vec2(visible.rect.width as f32, visible.rect.height as f32);
                let physical = preview_dimensions(displayed, dpr, device_limit, preview_budget);
                let key = RenderKey {
                    document_id: document.id.clone(),
                    page: visible.page,
                    region: None,
                    physical_width: physical[0],
                    physical_height: physical[1],
                    rotation: layout.rotation().degrees(),
                    generation: self.viewport.generation,
                };
                let slot = self.viewport.pages
                    .entry(visible.page)
                    .or_insert_with(|| ReaderPage {
                        requested: None,
                        cancel: Cancellation::default(),
                        image: None,
                        image_key: None,
                        plan: Default::default(),
                    });
                let decision = slot.plan.next(
                    &key,
                    slot.image_key.as_ref(),
                    slot.requested.as_ref(),
                    Instant::now(),
                );
                if !self.viewport.retry
                    && let Some(delay) = decision.wake_after
                {
                    ui.ctx().request_repaint_after(delay);
                }
                if !self.viewport.retry
                    && let Some(key) = decision.request
                {
                    slot.cancel.cancel();
                    slot.cancel = Cancellation::default();
                    match self.runtime.submit(Action::Render(
                        key.clone(),
                        yamantaka_domain::Priority::Visible,
                        slot.cancel.clone(),
                    )) {
                        Ok(()) => {
                            if self.capture.is_some() && self.capture_render_events.len() < 128 {
                                self.capture_render_events.push(serde_json::json!({
                                    "event":"submitted", "document":key.document_id,
                                    "page":key.page,"generation":key.generation,"rotation":key.rotation,
                                    "pixels":[key.physical_width,key.physical_height],
                                    "target":[physical[0],physical[1]],"viewport":viewport_size,
                                    "retainedImage":slot.image_key.is_some(),
                                }));
                            }
                            slot.requested = Some(key.clone());
                            self.viewport.requested = Some(key);
                        }
                        Err(error) => {
                            self.error = Some(error.to_string());
                            self.viewport.retry = true;
                        }
                    }
                }
                let paper = egui::Rect::from_min_size(
                    origin + egui::vec2(visible.rect.left as f32, visible.rect.top as f32),
                    displayed,
                );
                let response = ui.allocate_rect(paper, egui::Sense::hover());
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Image,
                        ui.is_enabled(),
                        format!(
                            "{} · {} {}",
                            document.display_title(),
                            page_label,
                            visible.page + 1
                        ),
                    )
                });
                if !matches!(
                    preferences.reading_theme,
                    ReaderTheme::Dark | ReaderTheme::Black
                ) {
                    ui.painter().add(
                        egui::epaint::Shadow {
                            offset: [0, 3],
                            blur: 12,
                            spread: 0,
                            color: Color32::from_black_alpha(32),
                        }
                        .as_shape(paper, 1),
                    );
                }
                if let Some(image) = &slot.image {
                    let rotation = slot
                        .image_key
                        .as_ref()
                        .map_or(layout.rotation().degrees(), |key| key.rotation);
                    ui.painter().add(render_plan::page_mesh(
                        image.id(),
                        paper,
                        rotation,
                        layout.rotation(),
                    ));
                } else {
                    ui.painter().rect_filled(paper, 1, Color32::WHITE);
                }
                self.paint_annotations(ui, visible.page, paper, layout.rotation());
                self.paint_find_matches(ui, visible.page, paper, layout.rotation(), p);
                self.selection_page(ui, &document.id, visible.page, paper, layout.rotation(), p);
                if !self.selection.enabled {
                    self.reader_link_regions(
                        ui,
                        &document.id,
                        visible.page,
                        paper,
                        layout.rotation(),
                        p,
                    );
                }
            }
            self.loading = !self.viewport.retry
                && self.viewport.pages
                    .values()
                    .any(|slot| !slot.plan.ready(slot.image_key.as_ref()));
            feedback = layout.anchor_at_scroll(&frame, anchor).ok();
        });
        if let Some(position) = feedback {
            let changed_page = position.page() != self.page;
            if let Some(navigation) = &mut self.navigation {
                navigation.update_anchor(position);
                self.page = navigation.position().page();
            }
            if changed_page && layout.layout() == PageLayout::Continuous {
                self.send(Action::Navigate(document.id.clone(), self.page));
            }
        }
        if self.panel && self.reader_panel_tab == outline::ReaderPanelTab::Text {
            self.request_page_text(&document.id);
        } else {
            self.text.clear();
        }
    }
}

pub(super) fn page_region(
    paper: egui::Rect,
    rotation: yamantaka_domain::reader_preferences::PageRotation,
    bounds: yamantaka_domain::pdf_geometry::NormalizedPageRect,
) -> egui::Rect {
    let bounds = bounds.rotated(rotation);
    let position = |point: [f64; 2]| {
        paper.min
            + egui::vec2(
                point[0] as f32 * paper.width(),
                point[1] as f32 * paper.height(),
            )
    };
    egui::Rect::from_min_max(position(bounds.min()), position(bounds.max()))
}
