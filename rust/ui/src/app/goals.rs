use super::*;
use yamantaka_application::reading_sessions::goals::{
    ReadingGoalChange, ReadingGoalReply, ReadingGoalRequest,
};
use yamantaka_domain::reading_sessions::goals::ReadingGoal;

#[derive(Default)]
pub(super) struct GoalState {
    document: Option<String>,
    reading: Option<Cancellation>,
    writing: Option<(String, ReadingGoalChange, Cancellation)>,
    loaded: bool,
    goal: Option<ReadingGoal>,
    error: Option<String>,
    editing: bool,
    pages: String,
    minutes: String,
    preview: u8,
}
impl GoalState {
    pub(super) fn ready(&self) -> bool {
        self.loaded && self.reading.is_none() && self.writing.is_none() && self.preview == 0
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"ready":self.ready(),"document":self.document,"pages":self.goal.and_then(ReadingGoal::pages),"minutes":self.goal.and_then(ReadingGoal::minutes),"editing":self.editing,"pendingRead":self.reading.is_some(),"pendingWrite":self.writing.is_some(),"error":self.error})
    }

    fn select(&mut self, document: &str) -> Option<ReadingGoalRequest> {
        if self.document.as_deref() == Some(document) {
            return None;
        }
        if let Some(cancel) = self.reading.take() {
            cancel.cancel();
        }
        self.document = Some(document.to_owned());
        self.loaded = false;
        self.goal = None;
        self.error = None;
        self.editing = false;
        self.pages.clear();
        self.minutes.clear();
        self.read()
    }
    fn read(&mut self) -> Option<ReadingGoalRequest> {
        let document = self.document.clone()?;
        if let Some(cancel) = self.reading.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        self.reading = Some(cancel.clone());
        self.error = None;
        Some(ReadingGoalRequest {
            document,
            change: ReadingGoalChange::Read,
            cancel,
        })
    }
    fn edit(&mut self) {
        self.pages = self
            .goal
            .and_then(ReadingGoal::pages)
            .map_or_else(String::new, |n| n.to_string());
        self.minutes = self
            .goal
            .and_then(ReadingGoal::minutes)
            .map_or_else(String::new, |n| n.to_string());
        self.editing = true;
        self.error = None;
    }
    fn draft(&self) -> yamantaka_domain::Result<Option<ReadingGoal>> {
        fn number(text: &str) -> yamantaka_domain::Result<Option<u32>> {
            let text = text.trim();
            if text.is_empty() {
                return Ok(None);
            }
            if !text.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(yamantaka_domain::Failure::Invalid(
                    "Use a whole positive number".into(),
                ));
            }
            text.parse()
                .map(Some)
                .map_err(|_| yamantaka_domain::Failure::Invalid("Reading goal is too large".into()))
        }
        let goal = ReadingGoal::new(number(&self.pages)?, number(&self.minutes)?)?;
        Ok((!goal.is_empty()).then_some(goal))
    }
    fn save(&mut self, goal: Option<ReadingGoal>) -> Option<ReadingGoalRequest> {
        if !self.loaded || self.writing.is_some() {
            return None;
        }
        let document = self.document.clone()?;
        if let Some(cancel) = self.reading.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        let change = ReadingGoalChange::Save(goal);
        self.writing = Some((document.clone(), change, cancel.clone()));
        self.error = None;
        Some(ReadingGoalRequest {
            document,
            change,
            cancel,
        })
    }
    fn accept(&mut self, reply: ReadingGoalReply) -> Option<String> {
        if reply.cancel.is_cancelled() {
            return None;
        }
        let selected = self.document.as_deref() == Some(reply.document.as_str());
        match reply.change {
            ReadingGoalChange::Read => {
                if !selected
                    || !self
                        .reading
                        .as_ref()
                        .is_some_and(|token| token.same_request(&reply.cancel))
                {
                    return None;
                }
                self.reading = None;
            }
            ReadingGoalChange::Save(_) => {
                if !self
                    .writing
                    .as_ref()
                    .is_some_and(|(document, change, token)| {
                        *document == reply.document
                            && *change == reply.change
                            && token.same_request(&reply.cancel)
                    })
                {
                    return None;
                }
                self.writing = None;
                if !selected {
                    return reply.result.err().map(|error| error.to_string());
                }
                if let Some(cancel) = self.reading.take() {
                    cancel.cancel();
                }
            }
        }
        match reply.result {
            Ok(goal) => {
                self.loaded = true;
                self.goal = goal;
                self.error = None;
                self.editing = false;
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
        None
    }
}
#[cfg(test)]
mod tests;
mod views;
