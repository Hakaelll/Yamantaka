//! Opt-in measurements of the native reader using an isolated demo library.
use super::*;

const PAUSE: Duration = Duration::from_millis(350);
const ZOOM_INTERVAL: Duration = Duration::from_millis(16);
const STEPS: &[(&str, u32)] = &[
    ("open", 0),
    ("next", 1),
    ("next", 2),
    ("next", 3),
    ("previous", 2),
    ("zoom", 2),
    ("resume", 2),
];

struct Pending {
    started: Instant,
    empty_paints: u64,
    visible_ms: Option<f64>,
    zoom_steps: u8,
}

#[derive(Default)]
pub(super) struct Probe {
    samples: Vec<serde_json::Value>,
    pending: Option<Pending>,
    next: Option<Instant>,
    zoom_tick_ms: Vec<f64>,
}

impl Probe {
    pub(super) fn ready(&self) -> bool {
        self.samples.len() == STEPS.len()
    }

    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"samples": self.samples, "zoomTickMs": self.zoom_tick_ms})
    }

    fn advance(&mut self, app: &mut YamantakaApp, ctx: &egui::Context) {
        let Some(&(action, page)) = STEPS.get(self.samples.len()) else {
            return;
        };
        assert!(app.error.is_none(), "Reader probe failed: {:?}", app.error);
        let now = Instant::now();
        if let Some(next) = self.next.filter(|&next| next > now) {
            ctx.request_repaint_after(next - now);
            return;
        }
        if let Some(pending) = &mut self.pending {
            let elapsed = now - pending.started;
            assert!(
                elapsed < Duration::from_secs(12),
                "Reader probe stalled at {action} on page {page}"
            );
            if action == "zoom" && pending.zoom_steps < 8 {
                self.zoom_tick_ms.push(elapsed.as_secs_f64() * 1000.0);
                pending.zoom_steps += 1;
                pending.started = Instant::now();
                app.set_reader_zoom(app.reader.zoom * 1.13);
                self.next = Some(now + ZOOM_INTERVAL);
                ctx.request_repaint_after(ZOOM_INTERVAL);
                return;
            }
            if let Some(slot) = app.reader.viewport.pages.get(&page)
                && matches!(app.surface, Surface::Reader)
                && app.reader.page == page
                && slot
                    .image_key
                    .as_ref()
                    .is_some_and(|key| key.document_id == "demo-0")
            {
                let ms = elapsed.as_secs_f64() * 1000.0;
                let visible_ms = *pending.visible_ms.get_or_insert(ms);
                if !app.reader.loading
                    && slot.plan.ready(slot.image_key.as_ref())
                    && slot.refinement.settled()
                    && !slot.refinement.blocked()
                {
                    self.samples.push(serde_json::json!({
                        "action": action,
                        "page": page,
                        "visible_ms": visible_ms,
                        "sharp_ms": ms,
                        "empty_paints": app
                            .reader
                            .viewport
                            .empty_paints
                            .saturating_sub(pending.empty_paints),
                    }));
                    self.pending = None;
                    self.next = Some(now + PAUSE);
                    if STEPS
                        .get(self.samples.len())
                        .is_some_and(|step| step.0 == "resume")
                    {
                        app.show_surface(Surface::Library);
                    }
                    ctx.request_repaint_after(PAUSE);
                    return;
                }
            }
            ctx.request_repaint_after(Duration::from_secs(12).saturating_sub(elapsed));
            return;
        }
        if !app.library_loaded {
            return;
        }
        let started = Instant::now();
        self.pending = Some(Pending {
            started,
            empty_paints: app.reader.viewport.empty_paints,
            visible_ms: None,
            zoom_steps: 1,
        });
        match action {
            "open" | "resume" => {
                let document = app
                    .documents
                    .iter()
                    .find(|doc| doc.id == "demo-0")
                    .unwrap()
                    .clone();
                assert!(app.open(document));
            }
            "next" => app.step_reader(true),
            "previous" => app.step_reader(false),
            "zoom" => {
                app.set_reader_zoom(app.reader.viewport.scale * 1.13);
                self.next = Some(now + ZOOM_INTERVAL);
                ctx.request_repaint_after(ZOOM_INTERVAL);
                return;
            }
            _ => unreachable!(),
        }
        ctx.request_repaint();
    }
}

impl YamantakaApp {
    pub fn measure_reader_performance(&mut self) {
        if self.demo && self.capture.is_some() {
            self.reader_performance = Some(Probe::default());
        }
    }

    pub(super) fn advance_reader_performance(&mut self, ctx: &egui::Context) {
        if let Some(mut probe) = self.reader_performance.take() {
            probe.advance(self, ctx);
            self.reader_performance = Some(probe);
        }
    }
}
