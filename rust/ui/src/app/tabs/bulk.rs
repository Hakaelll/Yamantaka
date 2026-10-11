use super::*;

impl YamantakaApp {
    pub(in crate::app) fn close_other_reader_tabs(&mut self, id: &str, to_right: bool) {
        if self.tabs.busy() || self.opening.busy() {
            return;
        }
        self.sync_tab_context();
        let mut plan = self.tabs.model.clone();
        let valid = if to_right {
            plan.close_to_right(id)
        } else {
            plan.close_others(id)
        };
        if valid {
            self.request_tab_plan(plan);
        }
    }
}
