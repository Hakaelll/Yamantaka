use super::*;
use yamantaka_domain::reader::ReaderPosition;
use yamantaka_domain::workspaces::ComparisonLayout;
use yamantaka_domain::{PageLayout, reader::ReaderMode};
mod admission;
mod bookmark_preview;
mod differences;
mod mount;
mod navigator;
mod options;
mod password;
mod preparation;
mod preview;
mod saved;
mod sync;
mod text_preview;
mod toolbar;
use admission::Admission;
pub(super) use mount::Mount;
pub(super) use preparation::Preview as PreparationPreview;

struct ReferenceTools<'a> {
    runtime: &'a Runtime,
    governor: &'a MemoryGovernor,
    export: &'a mut annotations::export::ExportState,
    reduced_motion: bool,
}
pub(super) struct ReferenceView {
    pub(in crate::app) deferred: bool,
    primary_stage: Option<mount::PrimaryStage>,
    provisional_paints: u64,
    opening_probe: Option<preparation::Probe>,
    retry_open: Option<preparation::OpenRequest>,
    admission: Admission,
    close_requested: bool,
    password_attempted: bool,
    selection: Option<Document>,
    pub(super) chooser: catalogue_chooser::CatalogueChooser,
    pub pane: reader_pane::ReaderPane,
    pub(in crate::app) input_layers: std::collections::HashSet<egui::LayerId>,
    pub(in crate::app) navigator_requested: bool,
    pub(in crate::app) copy_requested: bool,
    pub(in crate::app) notes_action: Option<annotations::NotesAction>,
    pub(in crate::app) notes_preview: bool,
    pub(in crate::app) notes_scroll_preview: bool,
    pub(in crate::app) selection_preview: selection::SelectionPreview,
    pub(in crate::app) selection_keyboard_preview: bool,
    pub(in crate::app) selection_tools_preview: bool,
    pub follow_page: bool,
    pub follow_zoom: bool,
    pub follow_position: bool,
    pub(in crate::app) bidirectional: bool,
    pub(in crate::app) swapped: bool,
    pub(in crate::app) search_visible: bool,
    pub(in crate::app) search_preview: bool,
    pub(in crate::app) outline_preview: bool,
    pub(in crate::app) thumbnails_preview: bool,
    pub(in crate::app) bookmarks_preview: bool,
    pub(in crate::app) text_preview: bool,
    pending_primary_page: Option<u32>,
    pending_primary_zoom: bool,
    pub(in crate::app) differences: std::collections::BTreeSet<u32>,
    difference_target: Option<u32>,
    followed_position: Option<ReaderPosition>,
    pub(in crate::app) layout: ComparisonLayout,
    pub(in crate::app) stacked: bool,
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
        if let Admission::Preparing(candidate) = &self.admission {
            candidate.cancel();
        }
    }
}

impl ReferenceView {
    fn retry_error(&mut self) {
        if let Some(request) = &self.retry_open {
            self.selection = Some(request.document.clone());
        } else {
            self.pane.viewport.reset_requests();
        }
        self.error = None;
    }
    pub(in crate::app) fn opening_summary(&self) -> Option<serde_json::Value> {
        self.opening_probe.as_ref().map(|probe| {
            let mut summary = probe.summary();
            summary["provisionalPaints"] = self.provisional_paints.into();
            summary
        })
    }
    pub(in crate::app) fn release_deferred(
        &mut self,
        source: Cancellation,
    ) -> yamantaka_domain::Result<()> {
        if self.deferred
            && !self.busy()
            && self
                .pane
                .viewport
                .pages
                .get(&self.pane.page)
                .is_some_and(|slot| slot.image.is_some())
        {
            self.primary_stage = Some(mount::PrimaryStage::new(source));
        }
        if self.deferred && self.primary_stage.is_none() {
            Err(yamantaka_domain::Failure::Invalid(
                "The saved reference image is not ready".into(),
            ))
        } else {
            Ok(())
        }
    }
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
        pane.visual_profile = source.visual_profile;
        pane.active = Some(active);
        pane.reader_panel_tab = outline::ReaderPanelTab::Outline;
        pane.navigation = Some(navigation);
        pane.reader_preferences = Some(preferences);
        if let Some(navigation) = &mut pane.navigation {
            navigation.go_to_page((source.page + 1).min(navigation.page_count() - 1));
        }
        pane.navigation_changed();
        pane.rebuild_layout(direction)?;
        Ok(Self {
            deferred: false,
            primary_stage: None,
            provisional_paints: 0,
            opening_probe: None,
            retry_open: None,
            admission: Admission::Ready,
            close_requested: false,
            password_attempted: false,
            selection: None,
            chooser: Default::default(),
            pane,
            input_layers: Default::default(),
            navigator_requested: false,
            copy_requested: false,
            notes_action: None,
            notes_preview: false,
            notes_scroll_preview: false,
            selection_preview: Default::default(),
            selection_keyboard_preview: false,
            selection_tools_preview: false,
            follow_page: false,
            follow_zoom: false,
            follow_position: false,
            bidirectional: false,
            swapped: false,
            search_visible: false,
            search_preview: false,
            outline_preview: false,
            thumbnails_preview: false,
            bookmarks_preview: false,
            text_preview: false,
            pending_primary_page: None,
            pending_primary_zoom: false,
            differences: Default::default(),
            difference_target: None,
            followed_position: None,
            layout: ComparisonLayout::Automatic,
            stacked: false,
            sync_preview: false,
            ratio: 0.5,
            size: egui::Vec2::ZERO,
            error: None,
        })
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        tools: &mut ReferenceTools<'_>,
        english: bool,
        p: Palette,
        primary: &reader_pane::ReaderPane,
        mut policy: reader_chrome::Policy,
    ) -> bool {
        policy.blocked |= self.busy() || self.pane.panel || self.error.is_some();
        policy.manual_hidden &= !self.busy();
        let mut state = std::mem::take(&mut self.pane.chrome);
        state.bind(
            self.pane
                .active
                .as_ref()
                .map_or("reference-pending", |(document, _)| document.id.as_str()),
        );
        let mut close = false;
        if let Some(layer) = state.show(
            ui,
            "reference-reading-controls",
            policy,
            if english {
                "Show reference controls"
            } else {
                "Mostrar controles de referencia"
            },
            p,
            |ui| {
                let result = self.overlay_controls(ui, tools, english, p, primary);
                close = result.0;
                result.1
            },
        ) {
            self.input_layers.insert(layer);
        }
        self.pane.chrome = state;
        self.viewport(ui, tools, english, p);
        close
    }

    fn overlay_controls(
        &mut self,
        ui: &mut egui::Ui,
        tools: &mut ReferenceTools<'_>,
        english: bool,
        p: Palette,
        primary: &reader_pane::ReaderPane,
    ) -> (bool, Option<egui::Id>) {
        let runtime = tools.runtime;
        let tr = |es, en| if english { en } else { es };
        let mut close = false;
        let mut first = None;
        let can_choose = self.pane.active.as_ref().map(|(doc, _)| &doc.id)
            == primary.active.as_ref().map(|(doc, _)| &doc.id);
        let notes_blocked = primary.annotations.writing();
        let primary = primary.navigation.as_ref();
        let compact = ui.available_width() < 560.0 || ui.available_height() < 180.0;
        if !can_choose
            && !compact
            && let Some((document, _)) = &self.pane.active
        {
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
            return (close, None);
        }
        let count = self
            .pane
            .navigation
            .as_ref()
            .map_or(1, ReaderNavigation::page_count);
        ui.horizontal_wrapped(|ui| {
            first = Some(self.navigator(ui, tools, notes_blocked, english, p));
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
            {
                self.set_page(page - 1);
            }
            if compact {
                let button = theme::icon_button(
                    ui,
                    tr("Opciones de referencia", "Reference options"),
                    theme::Icon::Menu,
                    p,
                );
                let mut popup = egui::Popup::menu(&button)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
                if self.sync_preview || self.chooser.preview || self.selection_tools_preview {
                    popup = popup.open(true);
                }
                popup.show(|ui| {
                    self.input_layers.insert(ui.layer_id());
                    ui.set_width((ui.ctx().content_rect().width() - 40.0).clamp(120.0, 420.0));
                    ui.horizontal_wrapped(|ui| {
                        self.toolbar_tools(ui, runtime, primary, can_choose, english, p)
                    });
                });
            } else {
                self.toolbar_tools(ui, runtime, primary, can_choose, english, p);
            }
            close = theme::icon_button(
                ui,
                tr("Cerrar referencia", "Close reference"),
                theme::Icon::Close,
                p,
            )
            .clicked();
        });
        if let Some(error) = self.error.clone().or_else(|| {
            self.pane
                .viewport
                .source_changed
                .then(|| reader_surface::source_change_message(english).into())
        }) {
            ui.horizontal_wrapped(|ui| {
                ui.label(error);
                if self.pane.viewport.source_changed {
                    if theme::navigation(ui, tr("Cerrar referencia", "Close reference"), false, p)
                        .clicked()
                    {
                        close = true;
                    }
                } else if theme::navigation(ui, tr("Reintentar", "Retry"), false, p).clicked() {
                    self.retry_error();
                }
            });
        }
        if let Some(error) = self.pane.annotation_paint.error().map(str::to_owned) {
            ui.horizontal_wrapped(|ui| {
                ui.label(tr(
                    "No se pudieron cargar las marcas de página.",
                    "Page marks could not be loaded.",
                ))
                .on_hover_text(error);
                if theme::navigation(ui, tr("Recargar marcas", "Reload marks"), false, p).clicked()
                {
                    self.pane.annotation_paint.invalidate_document(None);
                }
            });
        }
        (close, first)
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

    fn follow_primary_page(&mut self, page: u32) {
        if !self.busy()
            && !self.close_requested
            && self.follow_page
            && self.pending_primary_page.is_none()
            && !self.follow_position
            && let Some(nav) = &mut self.pane.navigation
            && nav.position().page() != page.min(nav.page_count() - 1)
            && nav.go_to_page(page.min(nav.page_count() - 1))
        {
            self.pane.navigation_changed();
        }
    }

    fn follow_primary_zoom(&mut self, scale: f32) {
        if !self.follow_zoom
            || self.pending_primary_zoom
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

    fn viewport(
        &mut self,
        ui: &mut egui::Ui,
        tools: &ReferenceTools<'_>,
        english: bool,
        p: Palette,
    ) {
        let runtime = tools.runtime;
        let governor = tools.governor;
        if self.deferred {
            self.provisional_paints += 1;
        }
        if self.pane.reader_destination.is_some() {
            let size = ui.available_size();
            let previous_zoom = self.pane.viewport.scale;
            match self
                .pane
                .resolve_destination([f64::from(size.x.max(1.0)), f64::from(size.y.max(1.0))])
            {
                Ok(true) => {
                    self.reference_navigation_changed();
                    if (self.pane.zoom - previous_zoom).abs() > 0.0001 {
                        if self.bidirectional && self.follow_zoom {
                            self.pending_primary_zoom = true;
                        } else {
                            self.follow_zoom = false;
                        }
                    }
                }
                Err(error) => self.error = Some(error),
                _ => {}
            }
        }
        let (Some((document, _)), Some(layout), Some(preferences), Some(navigation)) = (
            &self.pane.active,
            self.pane.viewport.layout.clone(),
            self.pane.effective_preferences(),
            &self.pane.navigation,
        ) else {
            return;
        };
        let document = document.clone();
        let bytes = governor.limits().pages.min(14 * 1024 * 1024) / 2;
        let anchor = navigation.position();
        let size = ui.available_size();
        let viewport = [f64::from(size.x.max(1.0)), f64::from(size.y.max(1.0))];
        ui.painter().rect_filled(
            ui.available_rect_before_wrap(),
            0,
            if self.pane.visual_profile == reader_profile::VisualProfile::ElectronicInk {
                Color32::WHITE
            } else {
                YamantakaApp::reading_paper(preferences.reading_theme)
            },
        );
        ui.spacing_mut().scroll.fade.strength = 0.0;
        let mut scroll = egui::ScrollArea::both()
            .animated(
                !tools.reduced_motion
                    && self.pane.visual_profile != reader_profile::VisualProfile::ElectronicInk,
            )
            .kinetic_scrolling(
                !tools.reduced_motion
                    && self.pane.visual_profile != reader_profile::VisualProfile::ElectronicInk,
            )
            .id_salt("reference-pages")
            .max_height(size.y.max(1.0))
            .min_scrolled_height(0.0)
            .auto_shrink([false, false]);
        if self.pane.viewport.scroll_anchor {
            if let Ok([x, y]) = layout.scroll_for_anchor(
                viewport,
                anchor,
                preferences.zoom_mode,
                f64::from(self.pane.zoom),
            ) {
                let restored = self.pane.restoration.scroll.take();
                let y = if self.pane.visual_profile == reader_profile::VisualProfile::ElectronicInk
                {
                    y
                } else {
                    restored.unwrap_or(y)
                };
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
            self.pane
                .chrome
                .record_frame(ui.ctx(), &frame, self.pane.viewport.generation);
            if !self.busy() && !self.close_requested {
                let pages: Vec<u32> = frame.pages.iter().map(|page| page.page).collect();
                self.pane
                    .links
                    .request_visible(runtime, &document.id, &pages);
                self.pane
                    .annotation_paint
                    .request(runtime, governor, &document.id, &pages);
            }
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
            let selection_pages: Vec<u32> = frame.pages.iter().map(|page| page.page).collect();
            if !self.busy() && !self.close_requested {
                if self.pane.panel && self.pane.reader_panel_tab == outline::ReaderPanelTab::Find {
                    self.pane.find.request_highlights(
                        runtime,
                        &document.id,
                        &selection_pages,
                        ui.ctx(),
                    );
                } else {
                    self.pane.find.clear_highlights();
                }
                self.pane
                    .selection
                    .request(runtime, &document.id, &selection_pages);
            }
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
                if let Err(error) = self.pane.viewport.paint_page(
                    ui,
                    runtime,
                    reader_surface::PagePaint {
                        key: &key,
                        paper,
                        preferences: &preferences,
                        label,
                        limits: page_refinement::Limits {
                            cap,
                            bytes: governor.limits().tiles / 12 / frame.pages.len().max(1),
                        },
                    },
                ) {
                    self.error = Some(error.to_string());
                }
                self.pane
                    .annotation_paint
                    .paint(ui, page.page, paper, layout.rotation());
                if self.differences.contains(&page.page) {
                    ui.painter().rect_stroke(
                        paper,
                        1,
                        egui::Stroke::new(2.0, p.accent),
                        egui::StrokeKind::Inside,
                    );
                }
                self.pane
                    .find
                    .paint(ui, page.page, paper, layout.rotation(), p);
                if !self.busy() && !self.close_requested {
                    self.pane.selection.show_page(
                        ui,
                        selection::SelectionPage {
                            document: &document.id,
                            page: page.page,
                            paper,
                            rotation: layout.rotation(),
                            english,
                        },
                        p,
                        Some(&mut self.selection_preview),
                    );
                }
                if !self.busy() && !self.close_requested && !self.pane.selection.enabled {
                    match self.pane.links.regions(
                        ui,
                        page.page,
                        paper,
                        layout.rotation(),
                        p,
                        english,
                    ) {
                        Ok(Some(destination)) => {
                            self.pane.reader_destination = Some((document.id.clone(), destination))
                        }
                        Err(error) => self.error = Some(error),
                        _ => {}
                    }
                }
            }
            if !self.busy() && !self.close_requested {
                self.pane
                    .selection
                    .drag_viewport(ui, &frame, origin, layout.rotation());
            }
            self.pane.loading = !self.pane.viewport.retry
                && self
                    .pane
                    .viewport
                    .pages
                    .values()
                    .any(|slot| !slot.plan.ready(slot.image_key.as_ref()));
            if let Some(probe) = &mut self.opening_probe {
                probe.observe(&self.pane.viewport);
            }
            self.pane.viewport.prefetch.plan(
                ui.ctx(),
                runtime,
                page_prefetch::View {
                    document: &document.id,
                    layout: &layout,
                    frame: &frame,
                    preferences: &preferences,
                    dpr,
                    cap,
                    bytes,
                },
                self.pane.viewport.generation,
                !self.pane.viewport.retry
                    && !self.pane.loading
                    && self.pane.viewport.refinements_settled(),
            );
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
    pub(super) fn toggle_reference(&mut self, ctx: &egui::Context) {
        if self.reference_mount.take().is_some() {
            return;
        }
        self.reader.viewport.scroll_anchor = true;
        if self.reference.is_some() {
            self.request_reference_close();
            return;
        }
        if self.opening.busy()
            || self.tabs.busy()
            || self.reader.reader_preferences_loading
            || self.open_confirmation.is_some()
        {
            return;
        }
        match Mount::new(
            &self.reader,
            self.reading_direction(),
            ctx,
            self.governor.limits().pages,
            self.capture.is_some(),
        ) {
            Ok(mut mount) => {
                if self.reference_mount_preview == PreparationPreview::FailOnce {
                    mount.inject_failure();
                    self.reference_mount_preview = PreparationPreview::Off;
                }
                self.cancel_continuity_comparison_restore();
                self.finish_continuity_comparison(false);
                self.reference_mount = Some(mount);
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub(super) fn comparison_viewport(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        p: Palette,
    ) {
        let Some(reference) = &mut self.reference else {
            return;
        };
        reference.follow_primary_page(self.reader.page);
        let rect = ui.available_rect_before_wrap();
        let geometry = comparison_rects(rect, reference.layout, reference.ratio, reference.swapped);
        let stacked = geometry.stacked;
        if reference.size != rect.size() || reference.stacked != stacked {
            reference.size = rect.size();
            reference.stacked = stacked;
            reference.followed_position = None;
            self.reader.viewport.scroll_anchor = true;
            reference.pane.viewport.scroll_anchor = true;
        }
        let extent = if stacked { rect.height() } else { rect.width() };
        let divider = geometry.divider;
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
            let delta = if stacked {
                drag.drag_delta().y
            } else {
                drag.drag_delta().x
            };
            reference.set_ratio(
                reference.ratio
                    + delta * if reference.swapped { -1.0 } else { 1.0 } / extent.max(1.0),
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
        let primary_rect = geometry.primary;
        let reference_rect = geometry.reference;
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(primary_rect)
                .id_salt("reading-pane"),
            |ui| {
                ui.set_clip_rect(primary_rect.intersect(ui.clip_rect()));
                self.reader_viewport(ui, document, p);
            },
        );
        let mut close = false;
        let mut selected = None;
        let mut difference = None;
        if let Some(reference) = &mut self.reference {
            reference.follow_primary_zoom(self.reader.viewport.scale);
            reference
                .follow_primary_position(self.reader.navigation.as_ref().map(|nav| nav.position()));
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(reference_rect)
                    .id_salt("reference-pane"),
                |ui| {
                    ui.set_clip_rect(reference_rect.intersect(ui.clip_rect()));
                    close = reference.show(
                        ui,
                        &mut ReferenceTools {
                            runtime: &self.runtime,
                            governor: &self.governor,
                            export: &mut self.annotation_export,
                            reduced_motion: self.reduced_motion,
                        },
                        self.english,
                        p,
                        &self.reader,
                        reader_chrome::Policy {
                            auto_hide: reference
                                .pane
                                .reader_preferences
                                .as_ref()
                                .is_some_and(|prefs| prefs.controls_auto_hide),
                            blocked: self.reader_chrome_blocked,
                            manual_hidden: self.reader_chrome_hidden,
                        },
                    );
                    selected = reference.selection.take();
                    difference = reference.difference_target.take();
                },
            );
        }
        self.reading_focus.register(
            ui,
            primary_rect,
            shortcuts::ReadingTarget::Primary,
            &format!(
                "{} · {} {}",
                self.tr("Lectura principal", "Main reading"),
                shortcuts::bindings::caption(
                    self.shortcut_bindings
                        .token(shortcuts::Shortcut::SwitchPane)
                ),
                self.tr("cambia de panel", "switches panes")
            ),
            p,
        );
        self.reading_focus.register(
            ui,
            reference_rect,
            shortcuts::ReadingTarget::Reference,
            &format!(
                "{} · {} {}",
                self.tr("Lectura de referencia", "Reference reading"),
                shortcuts::bindings::caption(
                    self.shortcut_bindings
                        .token(shortcuts::Shortcut::SwitchPane)
                ),
                self.tr("cambia de panel", "switches panes")
            ),
            p,
        );
        let notes_action = self
            .reference
            .as_mut()
            .and_then(|view| view.notes_action.take());
        if let Some(action) = notes_action {
            self.apply_notes_action(true, action);
        }
        let copy = self
            .reference
            .as_mut()
            .is_some_and(|reference| std::mem::take(&mut reference.copy_requested));
        if copy {
            self.copy_selection(ui.ctx(), true);
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
        if !close {
            if self.apply_reference_sync() {
                ui.ctx().request_repaint();
            }
            if let Some(page) = difference {
                self.navigate(page);
                ui.ctx().request_repaint();
            }
        }
        ui.allocate_rect(rect, egui::Sense::hover());
        if let Some(document) = selected {
            self.open_reference(document);
        }
    }
}

struct ComparisonRects {
    primary: egui::Rect,
    reference: egui::Rect,
    divider: egui::Rect,
    stacked: bool,
}
fn comparison_rects(
    rect: egui::Rect,
    layout: ComparisonLayout,
    ratio: f32,
    swapped: bool,
) -> ComparisonRects {
    let stacked = match layout {
        ComparisonLayout::Automatic => rect.width() < 960.0,
        ComparisonLayout::SideBySide => false,
        ComparisonLayout::Stacked => true,
    };
    let extent = if stacked { rect.height() } else { rect.width() };
    let split = extent * if swapped { 1.0 - ratio } else { ratio };
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
    let (primary, reference) = if swapped {
        (second, first)
    } else {
        (first, second)
    };
    ComparisonRects {
        primary,
        reference,
        divider,
        stacked,
    }
}
