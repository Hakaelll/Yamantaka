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
        let direction = self.reading_direction();
        if let Err(error) = self.reader.rebuild_layout(direction) {
            self.error = Some(error.to_string());
        }
    }

    pub(super) fn step_reader(&mut self, forward: bool) {
        if self.reader.navigation.as_mut().is_some_and(|navigation| {
            if forward {
                navigation.next_page()
            } else {
                navigation.previous()
            }
        }) {
            self.apply_navigation();
        }
    }

    pub(super) fn reader_viewport(&mut self, ui: &mut egui::Ui, document: &Document, p: Palette) {
        let viewport = ui.available_size();
        let viewport_size = [
            f64::from(viewport.x.max(1.0)),
            f64::from(viewport.y.max(1.0)),
        ];
        self.apply_reader_destination(viewport_size);
        let (Some(layout), Some(preferences), Some(navigation)) = (
            self.reader.viewport.layout.clone(),
            self.reader.effective_preferences(),
            self.reader.navigation.as_ref(),
        ) else {
            ui.painter()
                .rect_filled(ui.available_rect_before_wrap(), 0, self.reader_canvas(p));
            return;
        };
        let anchor = navigation.position();
        ui.painter()
            .rect_filled(ui.available_rect_before_wrap(), 0, self.reader_canvas(p));
        let mut scrolling = egui::ScrollArea::both()
            .animated(!self.reduced_motion && !self.electronic_ink())
            .kinetic_scrolling(!self.electronic_ink())
            .id_salt(("pdf-viewport", &document.id))
            .max_height(viewport.y.max(1.0))
            .min_scrolled_height(0.0)
            .auto_shrink([false, false]);
        if self.reader.viewport.scroll_anchor {
            match layout.scroll_for_anchor(
                viewport_size,
                anchor,
                preferences.zoom_mode,
                f64::from(self.reader.zoom),
            ) {
                Ok([x, y]) => {
                    let restored = self.reader.restoration.scroll.take();
                    let y = if self.electronic_ink() {
                        y
                    } else {
                        restored.unwrap_or(y)
                    };
                    scrolling = scrolling.scroll_offset(egui::vec2(x as f32, y as f32));
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    return;
                }
            }
            self.reader.viewport.scroll_anchor = false;
        }
        let dpr = ui.ctx().pixels_per_point();
        let device_limit = ui.ctx().input(|input| input.max_texture_side).min(16384) as u32;
        let mut feedback = None;
        scrolling.show_viewport(ui, |ui, visible| {
            if let Some(preferences) = &mut self.reader.reader_preferences {
                preferences.scroll_offset = f64::from(visible.top()).max(0.0);
            }
            let frame = match layout.frame(
                viewport_size,
                anchor,
                preferences.zoom_mode,
                f64::from(self.reader.zoom),
                [f64::from(visible.left()), f64::from(visible.top())],
                0.0,
            ) {
                Ok(frame) => frame,
                Err(error) => {
                    self.error = Some(error.to_string());
                    return;
                }
            };
            self.reader.viewport.scale = frame.zoom as f32;
            ui.set_min_size(egui::vec2(
                frame.content_size[0] as f32,
                frame.content_size[1] as f32,
            ));
            // Only intersecting pages retain presentation leases. Cancelling a
            // consumer never cancels another ticket that shares its native render.
            self.reader.viewport.pages.retain(|page, slot| {
                let keep = frame.pages.iter().any(|visible| visible.page == *page);
                if !keep {
                    slot.cancel.cancel();
                }
                keep
            });
            let link_pages: Vec<u32> = frame.pages.iter().map(|visible| visible.page).collect();
            self.reader.links.request_visible(&self.runtime, &document.id, &link_pages);
            self.reader.annotation_paint.request(&self.runtime, &self.governor, &document.id, &link_pages);
            self.request_selection_pages(&document.id, &link_pages);
            let origin = ui.max_rect().min;
            let count = frame.pages.len().max(1);
            let preview_bytes = self.governor.limits().pages.min(14 * 1024 * 1024) / if self.reference.is_some() { 2 } else { 1 };
            let preview_budget = (preview_bytes/count).max(4);
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
                    generation: self.reader.viewport.generation,
                };
                let retained = self.reader.viewport.pages.get(&visible.page)
                    .is_some_and(|slot| slot.image_key.is_some());
                let paper = egui::Rect::from_min_size(
                    origin + egui::vec2(visible.rect.left as f32, visible.rect.top as f32),
                    displayed,
                );
                let label = format!("{} · {} {}", document.display_title(), page_label, visible.page + 1);
                match self.reader.viewport.paint_page(ui, &self.runtime, reader_surface::PagePaint { key: &key, paper, preferences: &preferences, label, limits: page_refinement::Limits { cap: device_limit, bytes: self.governor.limits().tiles / 6 / if self.reference.is_some() { 2 } else { 1 } / count } }) {
                    Ok(Some(request)) if self.capture.is_some() && self.capture_render_events.len() < 128 => {
                        self.capture_render_events.push(serde_json::json!({
                            "event":"submitted", "document":request.document_id,
                            "page":request.page,"generation":request.generation,"rotation":request.rotation,
                            "pixels":[request.physical_width,request.physical_height],
                            "target":[physical[0],physical[1]],"viewport":viewport_size,
                            "retainedImage":retained,
                        }));
                    }
                    Err(error) => self.error = Some(error.to_string()),
                    _ => {}
                }
                self.reader.annotation_paint.paint(ui, visible.page, paper, layout.rotation());
                if self.reference.as_ref().is_some_and(|reference| reference.differences.contains(&visible.page)) {
                    ui.painter().rect_stroke(paper, 1, egui::Stroke::new(2.0, p.accent), egui::StrokeKind::Inside);
                }
                self.reader.find.paint(ui, visible.page, paper, layout.rotation(), p);
                self.selection_page(ui, &document.id, visible.page, paper, layout.rotation(), p);
                if !self.reader.selection.enabled {
                    match self.reader.links.regions(ui, visible.page, paper, layout.rotation(), p, self.english) {
                        Ok(Some(destination)) => self.reader.reader_destination = Some((document.id.clone(), destination)),
                        Err(error) => self.error = Some(error),
                        _ => {}
                    }
                }
            }
            self.opening.observe_frame(&document.id, &frame, &self.reader.viewport);
            if let Some(reference)=&mut self.reference {
                reference.observe_primary_mount(&frame,&self.reader.viewport);
            }
            self.reader.loading = !self.reader.viewport.retry
                && self.reader.viewport.pages
                    .values()
                    .any(|slot| !slot.plan.ready(slot.image_key.as_ref()));
            self.reader.viewport.prefetch.plan(ui.ctx(), &self.runtime, page_prefetch::View {
                document:&document.id,layout:&layout,frame:&frame,preferences:&preferences,
                dpr,cap:device_limit,bytes:preview_bytes,
            }, self.reader.viewport.generation, !self.reader.viewport.retry && !self.reader.loading && self.reader.viewport.refinements_settled());
            feedback = layout.anchor_at_scroll(&frame, anchor).ok();
        });
        if let Some(position) = feedback {
            let changed_page = position.page() != self.reader.page;
            if let Some(navigation) = &mut self.reader.navigation {
                navigation.update_anchor(position);
                self.reader.page = navigation.position().page();
            }
            if changed_page && layout.layout() == PageLayout::Continuous {
                self.send(Action::Navigate(document.id.clone(), self.reader.page));
            }
        }
        if self.reader.panel && self.reader.reader_panel_tab == outline::ReaderPanelTab::Text {
            self.request_page_text(&document.id);
        } else {
            self.reader.text.clear();
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
