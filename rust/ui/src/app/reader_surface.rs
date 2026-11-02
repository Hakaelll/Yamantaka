use super::*;

pub(super) struct ReaderSurface {
    pub prefetch: page_prefetch::State,
    pub empty_paints: u64,
    pub regions_enabled: bool,
    pub scale: f32,
    pub pages: HashMap<u32, ReaderPage>,
    pub layout: Option<Arc<ReaderLayout>>,
    pub scroll_anchor: bool,
    pub generation: u64,
    pub requested: Option<RenderKey>,
    pub retry: bool,
    pub cancel: Cancellation,
}

impl Default for ReaderSurface {
    fn default() -> Self {
        Self {
            prefetch: Default::default(),
            empty_paints: 0,
            regions_enabled: false,
            scale: 1.0,
            pages: HashMap::new(),
            layout: None,
            scroll_anchor: true,
            generation: 0,
            requested: None,
            retry: false,
            cancel: Cancellation::default(),
        }
    }
}

pub(super) struct PagePaint<'a> {
    pub key: &'a RenderKey,
    pub paper: egui::Rect,
    pub preferences: &'a ReaderPreferences,
    pub label: String,
    pub limits: page_refinement::Limits,
}

impl ReaderSurface {
    pub fn paint_page(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        paint: PagePaint<'_>,
    ) -> yamantaka_domain::Result<Option<RenderKey>> {
        let PagePaint {
            key,
            paper,
            preferences,
            label,
            limits,
        } = paint;
        let region = self
            .regions_enabled
            .then(|| {
                page_refinement::target(
                    key,
                    paper,
                    ui.clip_rect(),
                    ui.ctx().pixels_per_point(),
                    limits,
                )
            })
            .flatten();
        let prepared = if self
            .pages
            .get(&key.page)
            .is_none_or(|slot| slot.image.is_none())
        {
            self.prefetch.take(&render_plan::preview_key(key))
        } else {
            None
        };
        let slot = self.pages.entry(key.page).or_insert_with(|| ReaderPage {
            requested: None,
            cancel: Cancellation::default(),
            image: None,
            image_key: None,
            plan: Default::default(),
            refinement: Default::default(),
        });
        if let Some((image_key, image)) = prepared {
            slot.cancel.cancel();
            slot.requested = None;
            slot.image = Some(image);
            slot.image_key = Some(image_key);
        }
        let base = if region.is_some() && !slot.refinement.blocked() {
            render_plan::preview_key(key)
        } else {
            key.clone()
        };
        let decision = slot.plan.next(
            &base,
            slot.image_key.as_ref(),
            slot.requested.as_ref(),
            Instant::now(),
        );
        let mut submitted = None;
        let mut failure = None;
        if !self.retry {
            if let Some(delay) = decision.wake_after {
                ui.ctx().request_repaint_after(delay);
            }
            if let Some(request) = decision.request {
                slot.cancel.cancel();
                slot.cancel = Cancellation::default();
                match runtime.submit(Action::Render(
                    request.clone(),
                    yamantaka_domain::Priority::Visible,
                    slot.cancel.clone(),
                )) {
                    Ok(()) => {
                        slot.requested = Some(request.clone());
                        self.requested = Some(request.clone());
                        submitted = Some(request);
                    }
                    Err(error) => {
                        self.retry = true;
                        failure = Some(error);
                    }
                }
            }
        }
        let response = ui.allocate_rect(paper, egui::Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Image, ui.is_enabled(), &label)
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
                .map_or(key.rotation, |key| key.rotation);
            ui.painter().add(render_plan::page_mesh(
                image.id(),
                paper,
                rotation,
                preferences.page_rotation,
            ));
        } else {
            self.empty_paints += 1;
            ui.painter().rect_filled(paper, 1, Color32::WHITE);
        }
        slot.refinement.paint(
            ui,
            runtime,
            region,
            paper,
            key.rotation,
            slot.plan.ready(slot.image_key.as_ref()),
        );
        match failure {
            Some(error) => Err(error),
            None => Ok(submitted),
        }
    }

    pub fn accepts(&self, page: &yamantaka_domain::RenderedPage, cancel: &Cancellation) -> bool {
        self.pages
            .get(&page.key.page)
            .is_some_and(|slot| slot.accepts(page, cancel))
    }

    pub fn install(
        &mut self,
        page: &yamantaka_domain::RenderedPage,
        cancel: &Cancellation,
        image: PageImage,
    ) -> Option<bool> {
        let slot = self.pages.get_mut(&page.key.page)?;
        if !slot.accepts(page, cancel) {
            return None;
        }
        if slot.refinement.accepts(page, cancel) {
            return Some(slot.refinement.install(page, image));
        }
        let replaced = slot.image_key.is_some();
        slot.image = Some(image);
        slot.image_key = Some(page.key.clone());
        Some(replaced)
    }

    pub fn fail_refinement(&mut self, cancel: &Cancellation) -> bool {
        self.pages
            .values_mut()
            .any(|slot| slot.refinement.fail(cancel))
    }
    pub fn refinements_settled(&self) -> bool {
        self.pages.values().all(|slot| slot.refinement.settled())
    }
    pub fn refinement_summary(&self) -> serde_json::Value {
        serde_json::Value::Array(
            self.pages
                .values()
                .map(|slot| slot.refinement.summary())
                .collect(),
        )
    }

    pub fn reset_requests(&mut self) {
        self.prefetch.cancel_pending();
        self.requested = None;
        self.retry = false;
        self.scroll_anchor = true;
        for page in self.pages.values_mut() {
            page.cancel.cancel();
            page.requested = None;
            page.refinement.reset();
        }
    }

    pub fn clear(&mut self) {
        self.prefetch.clear();
        self.cancel.cancel();
        for page in self.pages.values() {
            page.cancel.cancel();
        }
        self.pages.clear();
        self.layout = None;
        self.cancel = Cancellation::default();
        self.reset_requests();
    }
}

impl Drop for ReaderSurface {
    fn drop(&mut self) {
        self.cancel.cancel();
        for page in self.pages.values() {
            page.cancel.cancel();
        }
    }
}
