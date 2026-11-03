use super::*;
use yamantaka_domain::Failure;

enum Mode {
    Hidden,
    Revealed,
    Held,
    Pinned,
}
pub(in crate::app) struct Capture {
    mode: Mode,
    start: Option<Instant>,
    before: Option<serde_json::Value>,
    hidden: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
}
impl Capture {
    fn new(mode: &str) -> yamantaka_domain::Result<Self> {
        let mode = match mode {
            "hidden" => Mode::Hidden,
            "revealed" => Mode::Revealed,
            "held" => Mode::Held,
            "pinned" => Mode::Pinned,
            _ => return Err(Failure::Invalid("Unknown synthetic chrome mode".into())),
        };
        Ok(Self {
            mode,
            start: None,
            before: None,
            hidden: None,
            result: None,
        })
    }

    pub(in crate::app) fn advance(
        &mut self,
        ctx: &egui::Context,
        ready: bool,
        primary: &mut State,
        mut reference: Option<&mut State>,
    ) -> bool {
        if self.result.is_some() {
            return true;
        }
        if !ready
            || primary.geometry.is_none()
            || reference
                .as_ref()
                .is_some_and(|state| state.geometry.is_none())
        {
            return false;
        }
        let geometry = serde_json::json!({"primary":primary.geometry,"reference":reference.as_ref().and_then(|state|state.geometry.as_ref())});
        self.before.get_or_insert_with(|| geometry.clone());
        let start = *self.start.get_or_insert_with(Instant::now);
        let hidden = primary.hidden && reference.as_ref().is_none_or(|state| state.hidden);
        let visible = !primary.hidden && reference.as_ref().is_none_or(|state| !state.hidden);
        let done = match self.mode {
            Mode::Hidden => hidden,
            Mode::Revealed if self.hidden.is_none() && hidden => {
                self.hidden = Some(geometry.clone());
                primary.reveal();
                if let Some(reference) = &mut reference {
                    reference.reveal();
                }
                ctx.request_repaint();
                false
            }
            Mode::Revealed => self.hidden.is_some() && visible,
            Mode::Held | Mode::Pinned => start.elapsed() >= Duration::from_millis(3300),
        };
        if !done {
            return false;
        }
        let unchanged = self.before.as_ref() == Some(&geometry)
            && self
                .hidden
                .as_ref()
                .is_none_or(|snapshot| snapshot == &geometry);
        let expected = match self.mode {
            Mode::Hidden => hidden,
            _ => visible,
        };
        self.result = Some(
            serde_json::json!({"synthetic":true,"geometryUnchanged":unchanged,"expectedVisibilityMet":expected,
            "elapsedMs":start.elapsed().as_secs_f64()*1000.0,"before":self.before,"hidden":self.hidden,"after":geometry}),
        );
        true
    }

    pub(in crate::app) fn summary(&self) -> serde_json::Value {
        self.result.clone().unwrap_or_default()
    }
}

impl YamantakaApp {
    pub fn preview_reader_chrome(&mut self, mode: &str) -> yamantaka_domain::Result<()> {
        if !self.demo || self.capture.is_none() {
            return Err(Failure::Invalid(
                "Chrome verification requires demo capture".into(),
            ));
        }
        self.chrome_capture = Some(Capture::new(mode)?);
        Ok(())
    }
}
