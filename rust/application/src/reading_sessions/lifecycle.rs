use super::{
    ReadingSessionChange, ReadingSessionOperation, ReadingSessionReply, ReadingSessionRequest,
};
use std::time::Duration;
use yamantaka_domain::reading_sessions::{
    ReaderSessionContext, ReadingSessionStart, validate_session_identity,
};
use yamantaka_domain::{Cancellation, Failure, Result, reading_activity::ReadingActivity};

pub const MAX_UNCOMPLETED_SESSIONS: usize = 32;

enum WriteState {
    Ready,
    Pending(ReadingSessionOperation, Cancellation),
    Failed,
}
struct Completion {
    ended_at: String,
    page: u32,
    active_seconds: u64,
    context: Option<ReaderSessionContext>,
}
enum Phase {
    Reading(ReadingActivity),
    Closed(Completion),
}
struct Session {
    id: String,
    document: String,
    started_at: String,
    start_page: u32,
    started: bool,
    phase: Phase,
    write: WriteState,
}
impl Session {
    fn request(&self, change: ReadingSessionChange) -> ReadingSessionRequest {
        ReadingSessionRequest {
            id: self.id.clone(),
            document: self.document.clone(),
            change,
            cancel: Cancellation::default(),
        }
    }
    fn start_request(&self) -> ReadingSessionRequest {
        self.request(ReadingSessionChange::Start {
            started_at: self.started_at.clone(),
            page: self.start_page,
        })
    }
    fn finish_request(&self, completion: &Completion) -> ReadingSessionRequest {
        self.request(ReadingSessionChange::Finish {
            ended_at: completion.ended_at.clone(),
            page: completion.page,
            active_seconds: completion.active_seconds,
            context: completion.context.clone(),
        })
    }
}

/// Owns pending writes independently of the visible reader and its PDF cancellation token.
#[derive(Default)]
pub struct ReadingSessionLifecycle {
    sessions: Vec<Session>,
}
impl ReadingSessionLifecycle {
    pub fn current_document(&self) -> Option<&str> {
        self.sessions.last().and_then(|session| {
            matches!(session.phase, Phase::Reading(_)).then_some(session.document.as_str())
        })
    }
    pub fn has_capacity(&self) -> bool {
        self.sessions.len() < MAX_UNCOMPLETED_SESSIONS
    }

    pub fn begin(
        &mut self,
        start: &ReadingSessionStart<'_>,
        now: Duration,
        visible: bool,
    ) -> Result<()> {
        if !self.has_capacity() {
            return Err(Failure::QueueFull);
        }
        if self
            .sessions
            .iter()
            .any(|s| matches!(s.phase, Phase::Reading(_)) || s.id == start.id)
        {
            return Err(Failure::Invalid(
                "Finish the current reading session before starting another".into(),
            ));
        }
        validate_session_identity(start.id, start.document)?;
        chrono::DateTime::parse_from_rfc3339(start.started_at)
            .map_err(|_| Failure::Invalid("Reading session start time is invalid".into()))?;
        self.sessions.push(Session {
            id: start.id.into(),
            document: start.document.into(),
            started_at: start.started_at.into(),
            start_page: start.page,
            started: false,
            phase: Phase::Reading(ReadingActivity::new(now, visible)),
            write: WriteState::Ready,
        });
        Ok(())
    }

    pub fn observe(&mut self, now: Duration, visible: bool, interaction: bool) {
        if let Some(session) = self.sessions.last_mut()
            && let Phase::Reading(activity) = &mut session.phase
        {
            activity.observe(now, visible, interaction);
        }
    }

    pub fn finish(
        &mut self,
        ended_at: String,
        page: u32,
        context: Option<ReaderSessionContext>,
        now: Duration,
    ) -> Result<()> {
        let Some(session) = self.sessions.last_mut() else {
            return Ok(());
        };
        let Phase::Reading(activity) = &mut session.phase else {
            return Ok(());
        };
        let ended = chrono::DateTime::parse_from_rfc3339(&ended_at)
            .map_err(|_| Failure::Invalid("Reading session end time is invalid".into()))?;
        let started = chrono::DateTime::parse_from_rfc3339(&session.started_at)
            .map_err(|_| Failure::Invalid("Reading session start time is invalid".into()))?;
        if ended < started {
            return Err(Failure::Invalid(
                "Reading session ends before its start".into(),
            ));
        }
        if let Some(context) = &context {
            context.validate()?;
        }
        let active_seconds = activity.observe(now, false, false);
        session.phase = Phase::Closed(Completion {
            ended_at,
            page,
            active_seconds,
            context,
        });
        Ok(())
    }

    pub fn next_request(&mut self) -> Option<ReadingSessionRequest> {
        for session in &mut self.sessions {
            if !matches!(session.write, WriteState::Ready) {
                continue;
            }
            let request = if !session.started {
                session.start_request()
            } else if let Phase::Closed(completion) = &session.phase {
                session.finish_request(completion)
            } else {
                continue;
            };
            session.write = WriteState::Pending(request.operation(), request.cancel.clone());
            return Some(request);
        }
        None
    }

    pub fn accept(&mut self, reply: ReadingSessionReply) -> Option<Result<()>> {
        let index = self.sessions.iter().position(|s| {
            s.id == reply.id
                && s.document == reply.document
                && matches!(&s.write, WriteState::Pending(operation, cancel)
                if *operation == reply.operation && cancel.same_request(&reply.cancel))
        })?;
        match &reply.result {
            Ok(()) if reply.operation == ReadingSessionOperation::Finish => {
                self.sessions.remove(index);
            }
            Ok(()) => {
                self.sessions[index].started = true;
                self.sessions[index].write = WriteState::Ready;
            }
            Err(_) => self.sessions[index].write = WriteState::Failed,
        }
        Some(reply.result)
    }

    pub fn submission_failed(&mut self, request: &ReadingSessionRequest, error: Failure) -> bool {
        self.accept(ReadingSessionReply {
            id: request.id.clone(),
            document: request.document.clone(),
            operation: request.operation(),
            result: Err(error),
            cancel: request.cancel.clone(),
        })
        .is_some()
    }

    pub fn retry(&mut self, id: &str) -> bool {
        if let Some(session) = self
            .sessions
            .iter_mut()
            .find(|s| s.id == id && matches!(s.write, WriteState::Failed))
        {
            session.write = WriteState::Ready;
            true
        } else {
            false
        }
    }

    /// Replays immutable start/finish pairs, including writes whose acknowledgements were lost.
    pub fn recovery_commands(&self) -> Result<Vec<ReadingSessionRequest>> {
        let mut commands = Vec::with_capacity(self.sessions.len() * 2);
        for session in &self.sessions {
            let Phase::Closed(completion) = &session.phase else {
                return Err(Failure::Invalid(
                    "Finalize active reading before collecting recovery writes".into(),
                ));
            };
            commands.push(session.start_request());
            commands.push(session.finish_request(completion));
        }
        Ok(commands)
    }
}
