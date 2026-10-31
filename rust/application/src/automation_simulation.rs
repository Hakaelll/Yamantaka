use crate::file_digest::HashedFile;
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    automation::simulation::*,
    automation::*,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation, TrackedBytesBuilder},
};
pub struct SimulationRequest {
    pub rule: Arc<SavedRule>,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
    pub watch: Arc<SimulationWatch>,
}
impl SimulationRequest {
    pub fn new(rule: Arc<SavedRule>, governor: MemoryGovernor) -> Self {
        Self {
            rule,
            governor,
            cancel: Cancellation::default(),
            watch: Arc::new(SimulationWatch::default()),
        }
    }
}
pub struct SimulationReply {
    pub rule: String,
    pub cancel: Cancellation,
    pub outcome: SimulationOutcome,
}
fn path_key(value: &str) -> String {
    let value = value.replace('\\', "/").to_lowercase();
    if let Some(unc) = value.strip_prefix("//?/unc/") {
        format!("//{unc}")
    } else {
        value.strip_prefix("//?/").unwrap_or(&value).to_owned()
    }
}
fn matches(trigger: &RuleTrigger, value: Option<&str>, entry: &ScanEntry) -> bool {
    match trigger.kind {
        TriggerKind::DocumentImported => false,
        TriggerKind::SourceFolder => value.is_some_and(|folder| {
            let path = path_key(&entry.normalized_path);
            let folder = folder.trim_end_matches('/');
            path == folder
                || path
                    .strip_prefix(folder)
                    .is_some_and(|tail| tail.starts_with('/'))
        }),
        TriggerKind::Extension => {
            value.is_some_and(|value| entry.file_name.to_lowercase().ends_with(value))
        }
        TriggerKind::Author => value.is_some_and(|value| {
            entry
                .author
                .as_ref()
                .is_some_and(|author| author.to_lowercase() == value)
        }),
        TriggerKind::Tag | TriggerKind::Collection => entry.membership,
        TriggerKind::Inaccessible => entry.missing,
        TriggerKind::ExactDuplicate => false,
    }
}
fn candidate(
    entry: &ScanEntry,
    proof: Option<ExactProof>,
    governor: &MemoryGovernor,
) -> Result<Arc<SimulationMatch>> {
    let path_bytes = proof.as_ref().map_or(0, |proof| {
        proof
            .source
            .canonical_path
            .as_os_str()
            .as_encoded_bytes()
            .len()
    });
    let bytes = entry
        .id
        .len()
        .checked_add(entry.title.len())
        .and_then(|bytes| bytes.checked_add(path_bytes.checked_mul(2)?))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<SimulationMatch>() + 256))
        .ok_or(Failure::MemoryBudget)?;
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Other,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    Ok(Arc::new(SimulationMatch::new(
        entry.id.clone(),
        entry.title.clone(),
        proof,
        lease,
    )))
}
fn observed(entry: &ScanEntry, file: &HashedFile) -> bool {
    use chrono::TimeZone;
    let expected = chrono::DateTime::parse_from_rfc3339(&entry.modified)
        .ok()
        .map(|date| date.with_timezone(&chrono::Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(&entry.modified, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .and_then(|date| chrono::Local.from_local_datetime(&date).single())
                .map(|date| date.with_timezone(&chrono::Utc))
        });
    expected == Some(chrono::DateTime::<chrono::Utc>::from(file.modified()))
        && file.size() == entry.size
        && file.fingerprint() == entry.fingerprint
}
fn execute(request: &SimulationRequest, reader: &dyn RuleScanReader) -> SimulationOutcome {
    let start = Instant::now();
    let mut progress = SimulationProgress::default();
    let check = || {
        request.cancel.check()?;
        if start.elapsed() >= Duration::from_secs(MAX_SIMULATION_SECONDS) {
            Err(Failure::Cancelled)
        } else {
            Ok(())
        }
    };
    let mut run = || -> Result<Option<Vec<Arc<SimulationMatch>>>> {
        check()?;
        let definition = request
            .rule
            .definition()
            .ok_or_else(|| Failure::Invalid("This saved rule format needs review".into()))?;
        definition.validate()?;
        let bound = reader.begin(&request.rule, &request.governor, &request.cancel)?;
        let Some(upper) = bound.value.as_deref() else {
            return Ok(Some(Vec::new()));
        };
        let value = definition.trigger.value.as_deref().map(|value| {
            if definition.trigger.kind == TriggerKind::SourceFolder {
                path_key(value.trim())
            } else {
                value.trim().to_lowercase()
            }
        });
        let exact = definition.trigger.kind == TriggerKind::ExactDuplicate;
        let mut buffer = if exact {
            Some(
                TrackedBytesBuilder::try_zeroed(&request.governor, MemoryCategory::Other, 65536)
                    .map_err(|_| Failure::MemoryBudget)?,
            )
        } else {
            None
        };
        let mut seen: HashMap<(u64, [u8; 32]), (Arc<SimulationMatch>, bool)> = HashMap::new();
        let mut output = Vec::new();
        let mut after: Option<Arc<ScanEntry>> = None;
        loop {
            check()?;
            let page = reader.page(
                after.as_ref().map(|entry| entry.id.as_str()),
                upper,
                &definition.trigger,
                &request.governor,
                &request.cancel,
            )?;
            if page.len() > 80
                || page.windows(2).any(|pair| pair[0].id >= pair[1].id)
                || page
                    .first()
                    .is_some_and(|entry| after.as_ref().is_some_and(|after| entry.id <= after.id))
                || page.last().is_some_and(|entry| entry.id.as_str() > upper)
            {
                return Err(Failure::Invalid(
                    "Simulation page is outside its ordered bounds".into(),
                ));
            }
            if page.is_empty() {
                break;
            }
            for entry in page {
                check()?;
                if progress.scanned >= MAX_SIMULATION_DOCUMENTS {
                    return Ok(None);
                }
                progress.scanned += 1;
                if exact {
                    let hashed = if entry.missing {
                        None
                    } else {
                        match HashedFile::read_checked(
                            &entry.path,
                            buffer.as_mut().unwrap().as_mut_slice(),
                            &check,
                        ) {
                            Ok(file) if observed(&entry, &file) => Some(file),
                            Ok(_) => None,
                            Err(Failure::Cancelled) => return Err(Failure::Cancelled),
                            Err(_) => None,
                        }
                    };
                    if let Some(file) = hashed {
                        let key = (file.size(), file.digest());
                        let proof = ExactProof {
                            source: file.source_revision(),
                            digest: file.digest(),
                        };
                        let matched = candidate(&entry, Some(proof), &request.governor)?;
                        if let Some((first, published)) = seen.get_mut(&key) {
                            if !*published {
                                output.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
                                output.push(first.clone());
                                *published = true;
                                progress.matched += 1;
                            }
                            output.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
                            output.push(matched);
                            progress.matched += 1;
                        } else {
                            seen.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
                            seen.insert(key, (matched, false));
                        }
                    } else {
                        progress.unverified += 1;
                    }
                } else if matches(&definition.trigger, value.as_deref(), &entry) {
                    let matched = candidate(&entry, None, &request.governor)?;
                    output.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
                    output.push(matched);
                    progress.matched += 1;
                }
                after = Some(entry);
                request.watch.publish(progress);
            }
        }
        if exact {
            output.sort_unstable_by(|left, right| left.document.cmp(&right.document));
        }
        check()?;
        Ok(Some(output))
    };
    match run() {
        Ok(Some(matches)) => SimulationOutcome::Complete(SimulationReport {
            rule: request.rule.clone(),
            progress,
            matches,
        }),
        Ok(None) => SimulationOutcome::LimitReached(progress),
        Err(Failure::Cancelled)
            if !request.cancel.is_cancelled()
                && start.elapsed() >= Duration::from_secs(MAX_SIMULATION_SECONDS) =>
        {
            SimulationOutcome::LimitReached(progress)
        }
        Err(Failure::Cancelled) => SimulationOutcome::Cancelled(progress),
        Err(error) => SimulationOutcome::Failed { progress, error },
    }
}
pub(crate) struct SimulationJob {
    rule: String,
    cancel: Cancellation,
    replies: Receiver<SimulationReply>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl SimulationJob {
    pub(crate) fn start(request: SimulationRequest, reader: RuleScanFactory) -> Result<Self> {
        request.cancel.check()?;
        let rule = request.rule.revision.id.clone();
        let cancel = request.cancel.clone();
        let (send, replies) = bounded(1);
        let handle = thread::Builder::new()
            .name("yamantaka-rule-simulation".into())
            .spawn(move || {
                let outcome = match reader() {
                    Ok(reader) => execute(&request, &*reader),
                    Err(error) => SimulationOutcome::Failed {
                        progress: Default::default(),
                        error,
                    },
                };
                let _ = send.send(SimulationReply {
                    rule: request.rule.revision.id.clone(),
                    cancel: request.cancel,
                    outcome,
                });
            })
            .map_err(|_| Failure::Storage("Simulation worker could not start".into()))?;
        Ok(Self {
            rule,
            cancel,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn poll(&mut self) -> Option<SimulationReply> {
        let reply = match self.replies.try_recv() {
            Ok(reply) => reply,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => SimulationReply {
                rule: self.rule.clone(),
                cancel: self.cancel.clone(),
                outcome: SimulationOutcome::Failed {
                    progress: Default::default(),
                    error: Failure::Storage("Simulation worker stopped".into()),
                },
            },
        };
        self.completed = true;
        Some(reply)
    }
}
impl Drop for SimulationJob {
    fn drop(&mut self) {
        if !self.completed {
            self.cancel.cancel();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
#[cfg(test)]
#[path = "automation_simulation/verification_test.rs"]
pub(crate) mod verification_test;
