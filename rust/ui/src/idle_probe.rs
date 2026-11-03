//! Synthetic native-loop observation. This does not measure GPU presentation time.
use eframe::egui;
use std::time::{Duration, Instant};

const WARMUP: Duration = Duration::from_secs(2);
const OBSERVATION: Duration = Duration::from_secs(2);

pub(crate) struct Probe {
    control: bool,
    ready_since: Option<Instant>,
    started: Option<(Instant, u64)>,
    input_events: usize,
    lost_readiness: bool,
    result: Option<serde_json::Value>,
}

impl Probe {
    pub(crate) fn new(control: bool) -> Self {
        Self {
            control,
            ready_since: None,
            started: None,
            input_events: 0,
            lost_readiness: false,
            result: None,
        }
    }

    pub(crate) fn observing(&self) -> bool {
        self.started.is_some() && self.result.is_none()
    }

    pub(crate) fn summary(&self) -> serde_json::Value {
        self.result.clone().unwrap_or_default()
    }

    pub(crate) fn advance(&mut self, ctx: &egui::Context, ready: bool) -> bool {
        if self.result.is_some() {
            return true;
        }
        let now = Instant::now();
        let frame = ctx.cumulative_frame_nr();
        let Some((start, first_frame)) = self.started else {
            if ready {
                let since = self.ready_since.get_or_insert(now);
                if now.duration_since(*since) >= WARMUP {
                    self.started = Some((now, frame));
                    let wake = ctx.clone();
                    if std::thread::Builder::new()
                        .name("synthetic-idle-deadline".into())
                        .spawn(move || {
                            std::thread::sleep(OBSERVATION);
                            wake.request_repaint();
                        })
                        .is_err()
                    {
                        self.result = Some(
                            serde_json::json!({"valid":false,"expectationMet":false,"error":"Could not start observation deadline"}),
                        );
                        return true;
                    }
                    if self.control {
                        ctx.request_repaint_after(Duration::from_millis(33));
                    }
                }
            } else {
                self.ready_since = None;
            }
            return false;
        };
        self.input_events += ctx.input(|input| input.events.len());
        self.lost_readiness |= !ready;
        let elapsed = now.duration_since(start);
        if elapsed < OBSERVATION {
            if self.control {
                ctx.request_repaint_after(Duration::from_millis(33));
            }
            return false;
        }
        // Exclude both boundary updates. Only updates inside the interval count.
        let intermediate = frame.saturating_sub(first_frame).saturating_sub(1);
        let valid = self.input_events == 0 && !self.lost_readiness;
        self.result = Some(serde_json::json!({
            "synthetic": true,
            "positiveControl": self.control,
            "warmupMs": WARMUP.as_millis() as u64,
            "requestedObservationMs": OBSERVATION.as_millis() as u64,
            "actualObservationMs": elapsed.as_secs_f64() * 1000.0,
            "firstFrame": first_frame,
            "lastFrame": frame,
            "intermediateUpdates": intermediate,
            "inputEvents": self.input_events,
            "lostReadiness": self.lost_readiness,
            "valid": valid,
            "expectationMet": valid && if self.control { intermediate >= 3 } else { intermediate == 0 },
            "scope": "egui native update loop only; no GPU/CPU/energy or interaction-time acceptance"
        }));
        true
    }
}
