use super::*;
mod capture;
mod probe;
pub(super) use capture::Capture;
pub(crate) use probe::verify;

#[derive(Default)]
pub(super) struct State {
    document: String,
    last_activity: Option<Instant>,
    hidden: bool,
    settled: bool,
    layer: Option<egui::LayerId>,
    focus_first: bool,
    rect: Option<egui::Rect>,
    geometry: Option<serde_json::Value>,
}

pub(super) struct Policy {
    pub auto_hide: bool,
    pub blocked: bool,
    pub manual_hidden: bool,
}

impl State {
    pub(super) fn bind(&mut self, document: &str) {
        if self.document != document {
            self.document = document.to_owned();
            self.reveal();
        }
    }

    pub(super) fn reveal(&mut self) {
        self.hidden = false;
        self.last_activity = Some(Instant::now());
    }

    pub(super) fn hidden(&self) -> bool {
        self.hidden
    }
    pub(super) fn settled(&self) -> bool {
        self.settled
    }

    fn advance(&mut self, ctx: &egui::Context, region: egui::Rect, policy: &Policy) {
        let now = Instant::now();
        let hotzone = egui::Rect::from_min_size(region.min, egui::vec2(region.width(), 56.0));
        let pointer = ctx.input(|input| {
            input.pointer.hover_pos().is_some_and(|pos| {
                hotzone.contains(pos) || self.rect.is_some_and(|rect| rect.contains(pos))
            })
        });
        let focused = !self.hidden
            && ctx
                .memory(|memory| memory.focused())
                .and_then(|id| ctx.read_response(id))
                .is_some_and(|response| Some(response.layer_id) == self.layer);
        let blocked = policy.blocked
            || focused
            || ctx.text_edit_focused()
            || egui::Popup::is_any_open(ctx)
            || ctx.input(|input| input.pointer.any_down());
        self.settled = policy.manual_hidden || !policy.auto_hide || blocked || pointer;
        if policy.manual_hidden {
            return;
        }
        if !policy.auto_hide || pointer {
            self.reveal();
            return;
        }
        if blocked {
            self.last_activity = Some(now);
            return;
        }
        if !self.hidden
            && ctx.input(|input| {
                input.events.iter().any(|event| {
                    matches!(
                        event,
                        egui::Event::PointerMoved(_)
                            | egui::Event::MouseWheel { .. }
                            | egui::Event::PointerButton { .. }
                            | egui::Event::Key { pressed: true, .. }
                            | egui::Event::Text(_)
                            | egui::Event::Touch { .. }
                    )
                })
            })
        {
            self.last_activity = Some(now);
        }
        let delay = if ctx.input(|input| input.viewport().fullscreen.unwrap_or(false)) {
            Duration::from_secs(2)
        } else {
            Duration::from_secs(3)
        };
        let elapsed = now.duration_since(*self.last_activity.get_or_insert(now));
        if elapsed >= delay {
            self.hidden = true;
        }
        if self.hidden {
            self.settled = true;
        } else {
            // egui subtracts its predicted frame time from delayed repaint requests.
            ctx.request_repaint_after(delay - elapsed + Duration::from_millis(50));
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        id: &'static str,
        policy: Policy,
        reveal_label: &str,
        p: Palette,
        contents: impl FnOnce(&mut egui::Ui) -> Option<egui::Id>,
    ) -> Option<egui::LayerId> {
        let region = ui.available_rect_before_wrap();
        self.advance(ui.ctx(), region, &policy);
        if policy.manual_hidden {
            return None;
        }
        let width = if self.hidden {
            40.0
        } else {
            (region.width() - 16.0).clamp(40.0, 360.0)
        };
        let position = egui::pos2(region.center().x - width * 0.5, region.top() + 8.0);
        let style = ui.style().clone();
        let output = egui::Area::new(egui::Id::new(id))
            .order(egui::Order::Foreground)
            .movable(false)
            .enabled(ui.is_enabled())
            .fixed_pos(position)
            .constrain_to(region)
            .show(ui.ctx(), |ui| {
                ui.set_style(style);
                ui.set_width(width);
                ui.set_max_width(width);
                ui.set_clip_rect(ui.clip_rect().intersect(region));
                egui::Frame::new()
                    .fill(p.surface)
                    .corner_radius(8)
                    .stroke(egui::Stroke::new(1.0, p.line))
                    .show(ui, |ui| {
                        if self.hidden {
                            let response =
                                theme::icon_button(ui, reveal_label, theme::Icon::Menu, p);
                            if response.clicked() {
                                self.reveal();
                                self.focus_first = response.has_focus();
                                ui.ctx().request_repaint();
                            }
                        } else if let Some(first) = contents(ui)
                            && self.focus_first
                        {
                            ui.memory_mut(|memory| memory.request_focus(first));
                            self.focus_first = false;
                        }
                    });
            });
        self.layer = Some(output.response.layer_id);
        self.rect = Some(output.response.rect);
        self.layer
    }

    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"autoHidden":self.hidden,"settled":self.settled,
            "rect":self.rect.map(|r|[r.left(),r.top(),r.right(),r.bottom()]),"geometry":self.geometry})
    }

    pub(super) fn record_frame(
        &mut self,
        ctx: &egui::Context,
        frame: &yamantaka_domain::reader_layout::ReaderFrame,
        generation: u64,
    ) {
        if ctx.data(|data| {
            data.get_temp::<bool>(egui::Id::new("capture-control-bounds"))
                .unwrap_or(false)
        }) {
            self.geometry = Some(
                serde_json::json!({"viewport":frame.viewport,"scale":frame.zoom,"scroll":frame.scroll,"generation":generation,
                "pages":frame.pages.iter().map(|page|serde_json::json!([page.page,page.rect.left,page.rect.top,page.rect.width,page.rect.height])).collect::<Vec<_>>()}),
            );
        }
    }
}

impl YamantakaApp {
    pub(super) fn reader_overlay_controls(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        p: Palette,
    ) {
        if self.reader_chrome_hidden {
            self.reader_focus_exit(ui, p);
            return;
        }
        let count = self
            .reader
            .navigation
            .as_ref()
            .map_or(1, ReaderNavigation::page_count);
        let policy = Policy {
            auto_hide: self
                .reader
                .reader_preferences
                .as_ref()
                .is_some_and(|prefs| prefs.controls_auto_hide),
            blocked: self.reader_chrome_blocked
                || self.reader.panel
                || self.reader.loading
                || self.opening.busy(),
            manual_hidden: false,
        };
        let label = self
            .tr("Mostrar controles de lectura", "Show reading controls")
            .to_owned();
        let mut state = std::mem::take(&mut self.reader.chrome);
        state.bind(&document.id);
        state.show(ui, "primary-reading-controls", policy, &label, p, |ui| {
            self.compact_reader_toolbar(ui, count, p)
        });
        self.reader.chrome = state;
    }
}
