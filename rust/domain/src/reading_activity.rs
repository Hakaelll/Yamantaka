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

#[cfg(test)]
mod tests {
    use super::*;
    fn seconds(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn idle_time_stops_at_the_activity_window_and_late_input_does_not_backfill() {
        let mut activity = ReadingActivity::new(Duration::ZERO, true);
        assert_eq!(activity.observe(seconds(10), true, false), 10);
        assert_eq!(activity.observe(seconds(300), true, true), 30);
        assert_eq!(activity.observe(seconds(305), true, false), 35);
        assert_eq!(activity.observe(seconds(600), true, false), 60);
    }
    #[test]
    fn hidden_time_is_excluded_and_returning_to_the_reader_starts_a_new_window() {
        let mut activity = ReadingActivity::new(Duration::ZERO, true);
        assert_eq!(activity.observe(seconds(8), false, false), 8);
        assert_eq!(activity.observe(seconds(100), false, true), 8);
        assert_eq!(activity.observe(seconds(200), true, false), 8);
        assert_eq!(activity.observe(seconds(209), true, false), 17);
    }
    #[test]
    fn repeated_and_regressing_observations_cannot_add_time_twice() {
        let mut activity = ReadingActivity::new(Duration::ZERO, true);
        assert_eq!(
            activity.observe(Duration::from_millis(1500), true, false),
            1
        );
        assert_eq!(activity.observe(seconds(1), true, false), 1);
        assert_eq!(
            activity.observe(Duration::from_millis(1500), true, false),
            1
        );
        assert_eq!(activity.observe(seconds(2), true, false), 2);
    }
    #[test]
    fn long_active_sessions_keep_the_legacy_twenty_four_hour_limit() {
        let mut activity = ReadingActivity::new(Duration::ZERO, true);
        for n in 1..=4000 {
            activity.observe(seconds(n * 25), true, true);
        }
        assert_eq!(activity.observe(seconds(100025), false, false), 86400);
    }
}
