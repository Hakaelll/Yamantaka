use std::time::Duration;

const ACTIVITY_WINDOW: Duration = Duration::from_secs(30);
const SESSION_LIMIT: Duration = Duration::from_secs(24 * 60 * 60);

/// Measures visible reading activity from monotonic elapsed time without periodic UI ticks.
pub struct ReadingActivity {
    observed: Duration,
    last_activity: Duration,
    active: Duration,
    visible: bool,
}
impl ReadingActivity {
    pub fn new(now: Duration, visible: bool) -> Self {
        Self {
            observed: now,
            last_activity: now,
            active: Duration::ZERO,
            visible,
        }
    }
    /// The caller supplies time elapsed from one session origin and the new visibility state.
    pub fn observe(&mut self, now: Duration, visible: bool, interaction: bool) -> u64 {
        let now = now.max(self.observed);
        if self.visible {
            let end = now.min(self.last_activity.saturating_add(ACTIVITY_WINDOW));
            self.active = self
                .active
                .saturating_add(end.saturating_sub(self.observed))
                .min(SESSION_LIMIT);
        }
        if interaction || visible && !self.visible {
            self.last_activity = now;
        }
        self.observed = now;
        self.visible = visible;
        self.active.as_secs()
    }
}
