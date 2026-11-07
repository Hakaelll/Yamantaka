use super::*;
mod capture;
mod probe;
pub(super) use capture::Capture;
pub(crate) use probe::verify;

#[derive(Default)]
pub(super) struct State {
    document: String,
    last_activity: Option<Instant>,
    pointer_reveal: Option<f64>,
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
            self.pointer_reveal = None;
            self.reveal();
        }
    }

    pub(super) fn reveal(&mut self) {
        self.hidden = false;
        self.last_activity = Some(Instant::now());
    }
    pub(super) fn focus_controls(&mut self) {
        self.reveal();
        self.pointer_reveal = None;
        self.focus_first = true;
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
                .is_some_and(|response| {
                    Some(response.layer_id) == self.layer
                        && self
                            .rect
                            .is_some_and(|rect| rect.contains(response.rect.center()))
                });
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
            if self.hidden && pointer && policy.auto_hide && !theme::reduced_motion(ctx) {
                self.pointer_reveal = Some(ctx.input(|input| input.time));
            }
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

    pub(super) fn show_inline(
        &mut self,
        ui: &mut egui::Ui,
        policy: Policy,
        reveal_label: &str,
        p: Palette,
        contents: impl FnOnce(&mut egui::Ui) -> Option<egui::Id>,
    ) -> bool {
        let rect = reader_controls::toolbar_slot(ui);
        self.advance(ui.ctx(), rect, &policy);
        self.layer = Some(ui.layer_id());
        self.rect = Some(rect);
        ui.painter().rect_filled(rect, 0, p.surface);
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            egui::Stroke::new(1.0, p.line),
        );
        let mut bar = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("primary-reading-controls")
                .max_rect(rect.shrink2(egui::vec2(8.0, 4.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        bar.set_clip_rect(bar.clip_rect().intersect(rect));
        bar.spacing_mut().item_spacing.x = 4.0;
        if policy.manual_hidden {
            return !policy.blocked && self.reveal_button(&mut bar, reveal_label, p);
        }
        self.controls(&mut bar, reveal_label, p, contents);
        false
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
                        self.controls(ui, reveal_label, p, contents);
                    });
            });
        self.layer = Some(output.response.layer_id);
        self.rect = Some(output.response.rect);
        self.layer
    }

    fn controls(
        &mut self,
        ui: &mut egui::Ui,
        reveal_label: &str,
        p: Palette,
        contents: impl FnOnce(&mut egui::Ui) -> Option<egui::Id>,
    ) {
        if self.hidden {
            self.reveal_button(ui, reveal_label, p);
        } else {
            if let Some(started) = self.pointer_reveal {
                let progress = ((ui.input(|input| input.time) - started) / 0.16) as f32;
                if self.focus_first || progress >= 1.0 || theme::reduced_motion(ui.ctx()) {
                    self.pointer_reveal = None;
                } else {
                    ui.set_opacity(1.0 - 0.15 * (1.0 - progress.clamp(0.0, 1.0)).powi(3));
                    ui.ctx().request_repaint();
                }
            }
            if let Some(first) = contents(ui)
                && self.focus_first
            {
                ui.memory_mut(|memory| memory.request_focus(first));
                self.focus_first = false;
            }
        }
    }

    fn reveal_button(&mut self, ui: &mut egui::Ui, label: &str, p: Palette) -> bool {
        let response = theme::icon_button(ui, label, theme::Icon::Menu, p);
        if response.clicked() {
            self.reveal();
            self.focus_first = response.has_focus();
            ui.ctx().request_repaint();
            true
        } else {
            false
        }
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
    pub(super) fn reader_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        count: u32,
        p: Palette,
    ) {
        let manual_hidden = self.reader_chrome_hidden || self.reader_focus;
        let keep_open =
            !manual_hidden && (self.reader.panel || self.reader.loading || self.opening.busy());
        let policy = Policy {
            auto_hide: self
                .reader
                .reader_preferences
                .as_ref()
                .is_some_and(|prefs| prefs.controls_auto_hide),
            blocked: self.reader_chrome_blocked || keep_open,
            manual_hidden,
        };
        let label = if self.reader_focus {
            self.tr("Salir de concentración", "Exit focus mode")
        } else {
            self.tr("Mostrar controles de lectura", "Show reading controls")
        };
        let label = if manual_hidden {
            let shortcut = if self.reader_focus {
                shortcuts::Shortcut::Focus
            } else {
                shortcuts::Shortcut::ToggleChrome
            };
            format!(
                "{label} · {}",
                shortcuts::bindings::caption(self.shortcut_bindings.token(shortcut))
            )
        } else {
            label.to_owned()
        };
        let mut state = std::mem::take(&mut self.reader.chrome);
        state.bind(&document.id);
        if keep_open {
            state.reveal();
        }
        let revealed = state.show_inline(ui, policy, &label, p, |ui| {
            self.reader_toolbar_contents(ui, count, p)
        });
        self.reader.chrome = state;
        if revealed {
            self.reader_focus = false;
            self.reader_chrome_hidden = false;
            if let Some(reference) = &mut self.reference {
                reference.pane.chrome.reveal();
            }
        }
    }
}
