use super::*;
use yamantaka_domain::{Priority, RenderedPage};

const MAX_VISIBLE: usize = 16;
const MAX_PHYSICAL_SIDE: u32 = 256;

#[derive(Default)]
pub(super) struct ThumbnailState {
    document: Option<String>,
    rotation: u16,
    generation: u64,
    pages: HashMap<u32, ReaderPage>,
    error: Option<String>,
    active_page: Option<u32>,
}

impl Drop for ThumbnailState {
    fn drop(&mut self) {
        self.clear();
    }
}

impl ThumbnailState {
    pub(super) fn clear(&mut self) {
        for slot in self.pages.values() {
            slot.cancel.cancel();
        }
        self.pages.clear();
        self.document = None;
        self.error = None;
        self.active_page = None;
    }

    fn bind(&mut self, document: &str, rotation: u16) {
        if self.document.as_deref() != Some(document) || self.rotation != rotation {
            self.clear();
            self.document = Some(document.into());
            self.rotation = rotation;
            self.generation = self.generation.wrapping_add(1);
        }
    }

    fn retain_visible(&mut self, visible: &[u32]) {
        self.pages.retain(|page, slot| {
            if visible.contains(page) {
                true
            } else {
                slot.cancel.cancel();
                false
            }
        });
    }

    pub(super) fn resident_count(&self) -> usize {
        self.pages.len()
    }

    pub(super) fn ready(&self) -> bool {
        !self.pages.is_empty()
            && self
                .pages
                .values()
                .all(|slot| slot.image.is_some() && slot.image_key == slot.requested)
    }

    pub(super) fn accepts(&self, page: &RenderedPage, cancel: &Cancellation) -> bool {
        self.pages
            .get(&page.key.page)
            .is_some_and(|slot| slot.accepts(page, cancel))
    }

    pub(super) fn fail(&mut self, cancel: &Cancellation, error: String) -> bool {
        if cancel.is_cancelled() {
            return false;
        }
        if self
            .pages
            .values()
            .any(|slot| slot.cancel.same_request(cancel))
        {
            self.error = Some(error);
            return true;
        }
        false
    }

    pub(super) fn install(
        &mut self,
        page: &RenderedPage,
        cancel: &Cancellation,
        result: yamantaka_domain::Result<PageImage>,
    ) {
        if !self.accepts(page, cancel) {
            return;
        }
        match result {
            Ok(image) => {
                let slot = self.pages.get_mut(&page.key.page).unwrap();
                slot.image = Some(image);
                slot.image_key = Some(page.key.clone());
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }
}

fn physical_dimensions(
    size: [f32; 2],
    rotation: u16,
    logical_height: f32,
    dpr: f32,
    device_limit: u32,
) -> [u32; 2] {
    let [width, height] = if rotation == 90 || rotation == 270 {
        [size[1], size[0]]
    } else {
        size
    };
    let side = (logical_height * dpr).clamp(1.0, MAX_PHYSICAL_SIDE.min(device_limit).max(1) as f32);
    let scale = side / width.max(height);
    [
        (width * scale).round().max(1.0) as u32,
        (height * scale).round().max(1.0) as u32,
    ]
}

impl YamantakaApp {
    pub fn preview_thumbnails(&mut self, page: Option<u32>) {
        if self.demo && self.capture.is_some() {
            self.reader.panel = true;
            self.reader.reader_panel_tab = outline::ReaderPanelTab::Thumbnails;
            self.thumbnail_preview_page = page;
        }
    }

    fn choose_thumbnail_page(&mut self, page: u32, compact: bool) {
        self.navigate(page);
        if compact {
            self.reader.panel = false;
        }
    }

    pub(super) fn thumbnails_panel_content(
        &mut self,
        ui: &mut egui::Ui,
        info: &PdfInfo,
        p: Palette,
    ) {
        let Some((document, _)) = &self.reader.active else {
            return;
        };
        let Some(preferences) = &self.reader.reader_preferences else {
            return;
        };
        let document = document.id.clone();
        let rotation = preferences.page_rotation.degrees();
        if let Some(page) = self.thumbnail_preview_page.take() {
            self.choose_thumbnail_page(page.min(info.pages.len().saturating_sub(1) as u32), false);
        }
        let compact = ui.ctx().content_rect().width() < 620.0;
        if !compact {
            ui.label(RichText::new(self.tr("Encuentra tu página", "Find your page")).size(22.0));
            ui.add_space(6.0);
            ui.label(
                RichText::new(self.tr(
                    "Recorre el documento de un vistazo.",
                    "Browse your document at a glance.",
                ))
                .color(p.secondary),
            );
            ui.add_space(14.0);
        }
        let jump = self.reader.thumbnails.show(
            ui,
            &self.runtime,
            ThumbnailContext {
                document: &document,
                info,
                page: self.reader.page,
                rotation,
                compact,
                english: self.english,
                reduced_motion: self.reduced_motion,
                max_height: None,
            },
            p,
        );
        if let Some(page) = jump {
            self.choose_thumbnail_page(page, compact);
        }
    }
}

pub(super) struct ThumbnailContext<'a> {
    pub document: &'a str,
    pub info: &'a PdfInfo,
    pub page: u32,
    pub rotation: u16,
    pub compact: bool,
    pub english: bool,
    pub reduced_motion: bool,
    pub max_height: Option<f32>,
}

impl ThumbnailState {
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        page_context: ThumbnailContext<'_>,
        p: Palette,
    ) -> Option<u32> {
        let ThumbnailContext {
            document,
            info,
            rotation,
            compact,
            english,
            ..
        } = page_context;
        let tr = |es, en| if english { en } else { es };
        self.bind(document, rotation);
        if let Some(error) = self.error.clone() {
            ui.label(tr(
                "No se pudieron preparar las páginas.",
                "Pages could not be prepared.",
            ));
            ui.collapsing(tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
            if theme::navigation(ui, tr("Volver a intentar", "Try again"), false, p).clicked() {
                self.clear();
            }
            return None;
        }
        let row_height = if compact { 48.0 } else { 172.0 };
        let scroll_to = if self.active_page != Some(page_context.page) {
            self.active_page = Some(page_context.page);
            Some(page_context.page as f32 * (row_height + ui.spacing().item_spacing.y))
        } else {
            None
        };
        let device_limit = ui
            .ctx()
            .input(|input| input.max_texture_side)
            .min(MAX_PHYSICAL_SIDE as usize) as u32;
        let dpr = ui.ctx().pixels_per_point();
        let mut visible = Vec::with_capacity(MAX_VISIBLE);
        let mut jump = None;
        let mut scrolling = egui::ScrollArea::vertical().id_salt(("page-thumbnails", &document));
        if let Some(offset) = scroll_to {
            scrolling = scrolling.vertical_scroll_offset(offset);
        }
        if let Some(height) = page_context.max_height {
            scrolling = scrolling
                .max_height(height)
                .min_scrolled_height(height.min(320.0));
        }
        scrolling.show_rows(ui, row_height, info.pages.len(), |ui, range| {
            visible.extend(range.clone().take(MAX_VISIBLE).map(|index| index as u32));
            self.retain_visible(&visible);
            for index in range {
                let page = index as u32;
                let size = info.pages[index].size();
                let dimensions = physical_dimensions(
                    size,
                    rotation,
                    row_height - if compact { 16.0 } else { 36.0 },
                    dpr,
                    device_limit,
                );
                let key = RenderKey {
                    document_id: document.into(),
                    page,
                    region: None,
                    physical_width: dimensions[0],
                    physical_height: dimensions[1],
                    rotation,
                    generation: self.generation,
                };
                if visible.contains(&page) {
                    let slot = self.pages.entry(page).or_insert_with(|| ReaderPage {
                        requested: None,
                        cancel: Cancellation::default(),
                        image: None,
                        image_key: None,
                        plan: Default::default(),
                        refinement: Default::default(),
                    });
                    if slot.requested.as_ref() != Some(&key) {
                        slot.cancel.cancel();
                        slot.cancel = Cancellation::default();
                        match runtime.submit(Action::Render(
                            key.clone(),
                            Priority::Thumbnail,
                            slot.cancel.clone(),
                        )) {
                            Ok(()) => slot.requested = Some(key),
                            Err(error) => self.error = Some(error.to_string()),
                        }
                    }
                }
                ui.push_id(page, |ui| {
                    let label = format!("{} {}", tr("Página", "Page"), u64::from(page) + 1);
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width().max(40.0), row_height),
                        egui::Sense::click(),
                    );
                    let selected = page_context.page == page;
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::Button,
                            ui.is_enabled(),
                            selected,
                            &label,
                        )
                    });
                    let hover = if page_context.reduced_motion {
                        f32::from(response.hovered())
                    } else {
                        ui.ctx().animate_bool_with_time(
                            response.id.with("thumbnail-hover"),
                            response.hovered(),
                            0.14,
                        )
                    };
                    let fill = if selected {
                        p.raised
                    } else {
                        p.surface.lerp_to_gamma(p.raised, hover)
                    };
                    ui.painter().rect(
                        rect,
                        8,
                        fill,
                        egui::Stroke::new(
                            if selected || response.has_focus() {
                                2.0
                            } else {
                                1.0
                            },
                            if selected || response.has_focus() {
                                p.accent
                            } else {
                                p.line
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    let available = if compact {
                        egui::vec2(52.0, row_height - 16.0)
                    } else {
                        egui::vec2((rect.width() - 24.0).max(1.0), row_height - 36.0)
                    };
                    let aspect = dimensions[0] as f32 / dimensions[1] as f32;
                    let height = available.y.min(available.x / aspect);
                    let image_size = egui::vec2(height * aspect, height);
                    let center = if compact {
                        egui::pos2(rect.left() + 38.0, rect.center().y)
                    } else {
                        egui::pos2(rect.center().x, rect.top() + 8.0 + available.y / 2.0)
                    };
                    let image_rect = egui::Rect::from_center_size(center, image_size);
                    if let Some(image) = self.pages.get(&page).and_then(|slot| slot.image.as_ref())
                    {
                        ui.painter().image(
                            image.id(),
                            image_rect,
                            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    } else {
                        ui.painter().rect_filled(image_rect, 1, p.raised);
                    }
                    let position = if compact {
                        egui::pos2(rect.left() + 80.0, rect.center().y)
                    } else {
                        egui::pos2(rect.center().x, rect.bottom() - 15.0)
                    };
                    ui.painter().text(
                        position,
                        if compact {
                            egui::Align2::LEFT_CENTER
                        } else {
                            egui::Align2::CENTER_CENTER
                        },
                        &label,
                        egui::FontId::proportional(14.0),
                        if selected { p.ink } else { p.secondary },
                    );
                    theme::record_control_bounds(ui, &label, rect);
                    if response.clicked() {
                        jump = Some(page);
                    }
                });
            }
        });
        self.retain_visible(&visible);
        jump
    }
}
