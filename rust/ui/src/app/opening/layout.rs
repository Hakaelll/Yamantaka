use super::preparation::Step;
use super::*;
use yamantaka_domain::reader::ReaderPosition;

impl YamantakaApp {
    pub(in crate::app) fn measure_opening_view(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let direction = self.reading_direction();
        let Some(pending) = self.opening.pending.as_mut() else {
            return;
        };
        let Some(candidate) = pending.candidate.as_mut() else {
            return;
        };
        let Step::Ready(preferences) = &candidate.preferences else {
            return;
        };
        let preferences = preferences.clone();
        let result = (|| {
            let mut effective = preferences;
            let mut panel = self.reader.panel;
            let mut restored_scroll = None;
            if let Some(context) = &pending.context {
                effective = context.apply(&effective)?;
                panel = context.panel().is_some();
                restored_scroll = Some(effective.scroll_offset);
            }
            if let Some(tab) = self.tabs.opening_tab(&pending.document, &pending.open) {
                effective = tab.preferences().clone();
                panel = tab.panel().is_some();
                restored_scroll = Some(effective.scroll_offset);
            }
            effective = self.reader.visual_profile.effective(&effective);
            if self.reader.visual_profile == reader_profile::VisualProfile::ElectronicInk {
                restored_scroll = None;
            }
            if candidate
                .geometry
                .as_ref()
                .is_none_or(|(prefs, reading, _)| prefs != &effective || *reading != direction)
            {
                let layout = ReaderLayout::new(
                    candidate.info.pages.iter().map(|page| page.size()),
                    &effective,
                    direction,
                    24.0,
                )?;
                candidate.geometry = Some((effective.clone(), direction, Arc::new(layout)));
            }
            let layout = &candidate.geometry.as_ref().unwrap().2;
            // A child UI measures the same native containers without advancing or painting the live UI.
            let mut measuring = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("opening-geometry")
                    .max_rect(rect)
                    .invisible(),
            );
            if !self.reader_chrome_hidden {
                reader_controls::toolbar_frame(&mut measuring, |ui| {
                    ui.allocate_space(egui::vec2(0.0, 40.0));
                });
                measuring.add_space(4.0);
            }
            if panel && !self.reader_focus {
                if ui.ctx().content_rect().width() < 620.0 {
                    return candidate.plan_pages(
                        vec![candidate.page],
                        (ui.input(|input| input.max_texture_side) as u32).min(768),
                        (self.governor.limits().pages / 6).clamp(4, 2 * 1024 * 1024),
                    );
                }
                let state = (!self.reader_panel_restore)
                    .then(|| {
                        egui::containers::panel::PanelState::load(
                            ui.ctx(),
                            egui::Id::new("reader-panel"),
                        )
                    })
                    .flatten();
                ui.ctx().data_mut(|data| {
                    let id = egui::Id::new("opening-reader-panel");
                    data.remove::<egui::containers::panel::PanelState>(id);
                    if let Some(state) = state {
                        data.insert_persisted(id, state);
                    }
                });
                bookmarks::navigator_frame("opening-reader-panel", self.reader_panel_width)
                    .resizable(false)
                    .show(&mut measuring, |ui| {
                        bookmarks::navigator_content(ui, |_| {})
                    });
            }
            let viewport = measuring.available_size();
            let viewport = [
                f64::from(viewport.x.max(1.0)),
                f64::from(viewport.y.max(1.0)),
            ];
            let anchor = ReaderPosition::at(
                candidate.page,
                effective.page_rotation.unrotate_point([0.5, 0.0]),
            )?;
            let zoom = f64::from(effective.zoom_level as f32);
            let [x, y] = layout.scroll_for_anchor(viewport, anchor, effective.zoom_mode, zoom)?;
            let mut frame = None;
            egui::ScrollArea::both()
                .id_salt("opening-pages")
                .max_height(viewport[1] as f32)
                .min_scrolled_height(0.0)
                .auto_shrink([false, false])
                .scroll_offset(egui::vec2(x as f32, restored_scroll.unwrap_or(y) as f32))
                .show_viewport(&mut measuring, |ui, visible| {
                    frame = Some(
                        layout
                            .frame(
                                viewport,
                                anchor,
                                effective.zoom_mode,
                                zoom,
                                [f64::from(visible.left()), f64::from(visible.top())],
                                0.0,
                            )
                            .inspect(|frame| {
                                ui.set_min_size(egui::vec2(
                                    frame.content_size[0] as f32,
                                    frame.content_size[1] as f32,
                                ));
                            }),
                    );
                });
            let frame = frame.expect("The scroll-area closure runs synchronously")?;
            self.opening
                .first_raster_probe
                .record_plan(&pending.document, &frame);
            let mut pages: Vec<_> = frame.pages.iter().map(|page| page.page).collect();
            if pages.is_empty() {
                pages.push(candidate.page);
            }
            candidate.plan_pages(
                pages,
                (ui.input(|input| input.max_texture_side) as u32).min(768),
                (self.governor.limits().pages / 6).clamp(4, 2 * 1024 * 1024),
            )?;
            if self
                .opening
                .first_raster_probe
                .inject_failure(&pending.document)
            {
                candidate.inject_failure();
            }
            yamantaka_domain::Result::Ok(())
        })();
        if let Err(error) = result {
            self.reject_open_preparation(error);
        }
    }
}
