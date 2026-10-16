//! Incremental indexing state machine for the existing actor. Every step is
//! nonblocking with respect to PDFium. Reserve one native document slot before
//! stepping; use `set_slot_available(false)` to yield its native handle before
//! another reader or importer takes that slot.
use crossbeam_channel::{Select, TryRecvError};
use std::collections::VecDeque;
use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, PdfInfo, Result,
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
    search::{IndexAccess, IndexSession, IndexStats, SearchRepository},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexPhase {
    Disabled,
    Idle,
    Running,
    Paused,
    Cancelling,
    Cancelled,
    Completed,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexProgress {
    pub enabled: bool,
    pub phase: IndexPhase,
    pub generation: u64,
    pub completed_documents: u64,
    pub failed_documents: u64,
    pub current_document: Option<String>,
    pub current_page: u32,
    pub total_pages: u32,
    pub last_error_code: Option<&'static str>,
}
impl Default for IndexProgress {
    fn default() -> Self {
        Self {
            enabled: false,
            phase: IndexPhase::Disabled,
            generation: 0,
            completed_documents: 0,
            failed_documents: 0,
            current_document: None,
            current_page: 0,
            total_pages: 0,
            last_error_code: None,
        }
    }
}
pub enum IndexCommand {
    Enable(bool),
    Start { rebuild: bool },
    Pause(bool),
    BackgroundPaused(bool),
    InteractionPaused(bool),
    Cancel,
    Exclude { document_id: String, excluded: bool },
    Clear,
}
struct Active {
    session: IndexSession,
    native_id: String,
    cancel: Cancellation,
}
enum Pending {
    Open(PdfTicket<PdfInfo>),
    Text(u32, PdfTicket<String>),
    Close(PdfTicket<()>),
}
struct Run {
    cancel: Cancellation,
    candidates: VecDeque<String>,
    after_id: Option<String>,
    active: Option<Active>,
    pending: Option<Pending>,
}
/// At most one native ticket, one active document and 80 candidate IDs are
/// retained. Progress is a coalesced snapshot; it never creates an event queue.
#[derive(Default)]
pub struct SearchIndexController {
    progress: IndexProgress,
    dirty: bool,
    manual_paused: bool,
    background_paused: bool,
    interaction_paused: bool,
    slot_paused: bool,
    run: Option<Run>,
}
impl SearchIndexController {
    pub fn progress(&self) -> &IndexProgress {
        &self.progress
    }
    pub fn take_progress(&mut self) -> Option<IndexProgress> {
        if std::mem::take(&mut self.dirty) {
            Some(self.progress.clone())
        } else {
            None
        }
    }
    /// Stop native admission and release the active handle at the next ready
    /// boundary. Durable progress and the run's candidate order are retained.
    /// Regranting a slot cannot revive an already cancelled native ticket.
    pub fn set_slot_available(&mut self, available: bool) {
        if self.slot_paused == !available {
            return;
        }
        self.slot_paused = !available;
        if !available && let Some(active) = self.run.as_ref().and_then(|run| run.active.as_ref()) {
            active.cancel.cancel();
        }
        if self.run.is_some() {
            self.phase();
        }
    }
    /// Includes a pending close: the caller must not reuse this native slot
    /// until the close reply has been consumed.
    pub fn native_document_id(&self) -> Option<&str> {
        self.run
            .as_ref()
            .and_then(|run| run.active.as_ref())
            .map(|active| active.native_id.as_str())
    }
    pub fn statistics<R: SearchRepository>(&self, repository: &R) -> Result<IndexStats> {
        repository.index_stats()
    }
    fn paused(&self) -> bool {
        self.manual_paused || self.background_paused || self.interaction_paused || self.slot_paused
    }
    fn phase(&mut self) {
        self.progress.phase = if let Some(run) = &self.run {
            if run.cancel.is_cancelled() {
                IndexPhase::Cancelling
            } else if self.paused() {
                IndexPhase::Paused
            } else {
                IndexPhase::Running
            }
        } else if !self.progress.enabled {
            IndexPhase::Disabled
        } else {
            IndexPhase::Idle
        };
        self.dirty = true;
    }
    fn cancel(&mut self) {
        if let Some(run) = &self.run {
            run.cancel.cancel();
            if let Some(active) = &run.active {
                active.cancel.cancel();
            }
        }
        self.phase();
    }
    pub fn command<R: SearchRepository>(
        &mut self,
        command: IndexCommand,
        repository: &mut R,
    ) -> Result<()> {
        match command {
            IndexCommand::Enable(enabled) => {
                self.progress.enabled = enabled;
                if !enabled {
                    self.cancel();
                }
                self.phase();
            }
            IndexCommand::Pause(value) => {
                self.manual_paused = value;
                self.phase();
            }
            IndexCommand::BackgroundPaused(value) => {
                self.background_paused = value;
                self.phase();
            }
            IndexCommand::InteractionPaused(value) => {
                self.interaction_paused = value;
                self.phase();
            }
            IndexCommand::Cancel => self.cancel(),
            IndexCommand::Start { rebuild } => {
                if !self.progress.enabled {
                    return Err(Failure::Invalid("Global indexing is disabled".into()));
                }
                if self.run.is_some() {
                    return Err(Failure::Invalid(
                        "Indexing is running or closing its document".into(),
                    ));
                }
                let cancel = Cancellation::default();
                let generation = self
                    .progress
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| Failure::Invalid("Index generation overflow".into()))?;
                if rebuild {
                    repository.clear_index(&cancel)?;
                }
                self.progress.generation = generation;
                self.progress.completed_documents = 0;
                self.progress.failed_documents = 0;
                self.progress.current_page = 0;
                self.progress.total_pages = 0;
                self.progress.current_document = None;
                self.progress.last_error_code = None;
                self.run = Some(Run {
                    cancel,
                    candidates: VecDeque::new(),
                    after_id: None,
                    active: None,
                    pending: None,
                });
                self.phase();
            }
            IndexCommand::Exclude {
                document_id,
                excluded,
            } => {
                // Cancelling first invalidates all late native replies. The
                // repository exclusion also invalidates the stored revision.
                self.cancel();
                repository.exclude_from_index(&document_id, excluded, &Cancellation::default())?;
            }
            IndexCommand::Clear => {
                self.cancel();
                repository.clear_index(&Cancellation::default())?;
            }
        }
        Ok(())
    }
    pub fn register_readiness<'a>(&'a self, select: &mut Select<'a>) {
        if let Some(pending) = self.run.as_ref().and_then(|r| r.pending.as_ref()) {
            match pending {
                Pending::Open(t) => {
                    select.recv(t.readiness());
                }
                Pending::Text(_, t) => {
                    select.recv(t.readiness());
                }
                Pending::Close(t) => {
                    select.recv(t.readiness());
                }
            }
        }
    }
    pub fn needs_step(&self) -> bool {
        self.run.as_ref().is_some_and(|run| match &run.pending {
            Some(Pending::Open(t)) => t.is_ready(),
            Some(Pending::Text(_, t)) => t.is_ready(),
            Some(Pending::Close(t)) => t.is_ready(),
            None => {
                run.cancel.is_cancelled()
                    || run
                        .active
                        .as_ref()
                        .is_some_and(|active| active.cancel.is_cancelled())
                    || !self.paused()
            }
        })
    }
    fn record_failure<R: SearchRepository>(
        &mut self,
        repository: &mut R,
        error: &Failure,
    ) -> Result<()> {
        let code = match error {
            Failure::PasswordRequired => "passwordRequired",
            Failure::Cancelled => "cancelled",
            Failure::Storage(_) => "storage",
            Failure::Invalid(_) => "sourceChanged",
            _ => "pdf",
        };
        self.progress.failed_documents += 1;
        self.progress.last_error_code = Some(code);
        self.dirty = true;
        if let Some(run) = &mut self.run
            && !run.cancel.is_cancelled()
            && let Some(active) = &mut run.active
        {
            repository.fail_index(&mut active.session, code, &run.cancel)?;
        }
        Ok(())
    }
    fn close<E: AsyncPdfEngine>(&mut self, engine: &E) -> Result<()> {
        if let Some(run) = &mut self.run
            && let Some(active) = &run.active
        {
            run.pending = Some(Pending::Close(engine.try_close(active.native_id.clone())?));
        }
        Ok(())
    }
    fn finish_run<R: SearchRepository>(
        &mut self,
        repository: &mut R,
        cancelled: bool,
    ) -> Result<()> {
        if !cancelled {
            repository.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())?;
        }
        self.run = None;
        self.progress.current_document = None;
        self.progress.current_page = 0;
        self.progress.total_pages = 0;
        self.progress.phase = if !self.progress.enabled {
            IndexPhase::Disabled
        } else if cancelled {
            IndexPhase::Cancelled
        } else {
            IndexPhase::Completed
        };
        self.dirty = true;
        Ok(())
    }
    /// Execute at most one boundary per actor turn. SQL writes commit together;
    /// native work is submitted as a ticket and completed on a later turn.
    pub fn step<R: SearchRepository + LibraryRepository, E: AsyncPdfEngine>(
        &mut self,
        repository: &mut R,
        engine: &E,
    ) -> Result<bool> {
        if !self.needs_step() {
            return Ok(false);
        }
        let pending = self.run.as_mut().and_then(|r| r.pending.take());
        if let Some(pending) = pending {
            match pending {
                Pending::Open(ticket) => {
                    let result = ticket_result(&ticket);
                    if self.native_cancelled() {
                        self.close(engine)?;
                        return Ok(true);
                    }
                    match result {
                        Ok(info) => {
                            let active = self.run.as_ref().unwrap().active.as_ref().unwrap();
                            if info.pages.len() != active.session.page_count as usize {
                                self.record_failure(
                                    repository,
                                    &Failure::Invalid(
                                        "PDF page count changed; revalidate the document".into(),
                                    ),
                                )?;
                                self.close(engine)?;
                            }
                        }
                        Err(error) => {
                            self.record_failure(repository, &error)?;
                            self.close(engine)?;
                        }
                    }
                }
                Pending::Text(page, ticket) => {
                    let result = ticket_result(&ticket);
                    if self.native_cancelled() {
                        self.close(engine)?;
                        return Ok(true);
                    }
                    match result {
                        Ok(text) => {
                            let run = self.run.as_mut().unwrap();
                            let active = run.active.as_mut().unwrap();
                            if let Err(error) = repository.index_page(
                                &mut active.session,
                                page,
                                &text,
                                IndexAccess::Enabled,
                                &run.cancel,
                            ) {
                                self.record_failure(repository, &error)?;
                                self.close(engine)?;
                            } else {
                                self.progress.current_page = active.session.cursor;
                                self.dirty = true;
                            }
                        }
                        Err(error) => {
                            self.record_failure(repository, &error)?;
                            self.close(engine)?;
                        }
                    }
                }
                Pending::Close(ticket) => {
                    let result = ticket_result(&ticket);
                    let run = self.run.as_mut().unwrap();
                    if let Some(active) = run.active.take()
                        && active.cancel.is_cancelled()
                        && !run.cancel.is_cancelled()
                    {
                        run.candidates.push_front(active.session.document_id);
                    }
                    self.progress.current_document = None;
                    self.dirty = true;
                    if let Err(error) = result {
                        self.progress.last_error_code = Some("close");
                        self.cancel();
                        return Err(error);
                    }
                }
            }
            return Ok(true);
        }
        if self.run.as_ref().unwrap().cancel.is_cancelled() {
            if self.run.as_ref().unwrap().active.is_some() {
                self.close(engine)?;
            } else {
                self.finish_run(repository, true)?;
            }
            return Ok(true);
        }
        if self.native_cancelled() {
            self.close(engine)?;
            return Ok(true);
        }
        if self.paused() {
            return Ok(false);
        }
        if self.run.as_ref().unwrap().active.is_some() {
            let run = self.run.as_mut().unwrap();
            let active = run.active.as_mut().unwrap();
            if active.session.cursor == active.session.page_count {
                repository.finish_index(&mut active.session, IndexAccess::Enabled, &run.cancel)?;
                self.progress.completed_documents += 1;
                self.dirty = true;
                self.close(engine)?;
            } else {
                let page = active.session.cursor;
                match engine.try_text(active.native_id.clone(), page, active.cancel.clone()) {
                    Ok(ticket) => run.pending = Some(Pending::Text(page, ticket)),
                    Err(error) => {
                        self.record_failure(repository, &error)?;
                        self.close(engine)?;
                    }
                }
            }
            return Ok(true);
        }
        let run = self.run.as_mut().unwrap();
        if run.candidates.is_empty() {
            run.candidates = repository
                .index_candidates(
                    run.after_id.as_deref(),
                    80,
                    IndexAccess::Enabled,
                    &run.cancel,
                )?
                .into();
        }
        let Some(id) = run.candidates.pop_front() else {
            self.finish_run(repository, false)?;
            return Ok(true);
        };
        run.after_id = Some(id.clone());
        let Some(document) = repository.get(&id)? else {
            return Ok(true);
        };
        if document.missing || !repository.needs_index(&id)? {
            return Ok(true);
        }
        let native_id = crate::temporary_document::reserve(repository, "index")?;
        let session = repository.begin_index(&id, true, IndexAccess::Enabled, &run.cancel)?;
        self.progress.current_document = Some(id);
        self.progress.current_page = session.cursor;
        self.progress.total_pages = session.page_count;
        self.dirty = true;
        let native_cancel = Cancellation::default();
        run.active = Some(Active {
            session,
            native_id: native_id.clone(),
            cancel: native_cancel.clone(),
        });
        match engine.try_open(
            native_id,
            document.path,
            None,
            yamantaka_domain::pdf_ticket::PdfOpenPurpose::Background,
            native_cancel,
        ) {
            Ok(ticket) => run.pending = Some(Pending::Open(ticket)),
            Err(error) => {
                self.record_failure(repository, &error)?;
                self.close(engine)?;
            }
        }
        Ok(true)
    }
    /// Shutdown cancels pending work and returns a close ticket to join with
    /// the actor's shutdown tickets. Never drop the engine before that join.
    pub fn shutdown<E: AsyncPdfEngine>(&mut self, engine: &E) -> Result<Option<PdfTicket<()>>> {
        self.cancel();
        let run = self.run.take();
        self.progress.phase = if self.progress.enabled {
            IndexPhase::Cancelled
        } else {
            IndexPhase::Disabled
        };
        self.progress.current_document = None;
        self.progress.current_page = 0;
        self.progress.total_pages = 0;
        self.dirty = true;
        let Some(mut run) = run else {
            return Ok(None);
        };
        if let Some(Pending::Close(ticket)) = run.pending.take() {
            return Ok(Some(ticket));
        }
        run.pending.take();
        run.active
            .map(|active| engine.try_close(active.native_id))
            .transpose()
    }
    /// Stop a failed run without retrying a failing repository or admission in
    /// a tight loop. The caller retains and consumes the returned close ticket.
    pub fn abort<E: AsyncPdfEngine>(
        &mut self,
        error: &Failure,
        engine: &E,
    ) -> Result<Option<PdfTicket<()>>> {
        self.progress.failed_documents += 1;
        self.progress.last_error_code = Some(match error {
            Failure::Storage(_) => "storage",
            Failure::Invalid(_) => "sourceChanged",
            Failure::PasswordRequired => "passwordRequired",
            Failure::Cancelled => "cancelled",
            _ => "pdf",
        });
        self.shutdown(engine)
    }
    fn native_cancelled(&self) -> bool {
        self.run.as_ref().is_some_and(|run| {
            run.cancel.is_cancelled()
                || run
                    .active
                    .as_ref()
                    .is_some_and(|active| active.cancel.is_cancelled())
        })
    }
}
impl Drop for SearchIndexController {
    fn drop(&mut self) {
        if let Some(run) = &self.run {
            run.cancel.cancel();
            if let Some(active) = &run.active {
                active.cancel.cancel();
            }
        }
    }
}
fn ticket_result<T>(ticket: &PdfTicket<T>) -> Result<T> {
    match ticket.try_recv() {
        Ok(result) => result,
        Err(TryRecvError::Disconnected) => Err(Failure::Pdf("PDF worker stopped".into())),
        Err(TryRecvError::Empty) => Err(Failure::Invalid(
            "Index ticket was consumed before readiness".into(),
        )),
    }
}
