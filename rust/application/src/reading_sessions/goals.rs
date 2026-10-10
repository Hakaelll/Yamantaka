use yamantaka_domain::{
    Cancellation, Result,
    reading_sessions::{ReadingSessionRepository, goals::ReadingGoal},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingGoalChange {
    Read,
    Save(Option<ReadingGoal>),
}
pub struct ReadingGoalRequest {
    pub document: String,
    pub change: ReadingGoalChange,
    pub cancel: Cancellation,
}
pub struct ReadingGoalReply {
    pub document: String,
    pub change: ReadingGoalChange,
    pub cancel: Cancellation,
    pub result: Result<Option<ReadingGoal>>,
}
impl ReadingGoalRequest {
    pub(crate) fn execute(
        self,
        repository: &mut impl ReadingSessionRepository,
    ) -> ReadingGoalReply {
        let result = self.cancel.check().and_then(|()| match self.change {
            ReadingGoalChange::Read => repository.reading_goal(&self.document, &self.cancel),
            ReadingGoalChange::Save(goal) => {
                repository.save_reading_goal(&self.document, goal, &self.cancel)
            }
        });
        ReadingGoalReply {
            document: self.document,
            change: self.change,
            cancel: self.cancel,
            result,
        }
    }
}
