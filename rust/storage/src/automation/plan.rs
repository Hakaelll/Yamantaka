use super::*;
use rusqlite::types::ValueRef;
use yamantaka_domain::{
    automation::{plan::*, simulation::*},
    memory::ByteLease,
};

pub(super) fn signature(row: &rusqlite::Row<'_>) -> Result<[u8; 32]> {
    let mut digest = Sha256::new();
    for index in 0..row.as_ref().column_count() {
        match row.get_ref(index).map_err(storage)? {
            ValueRef::Null => digest.update([0]),
            ValueRef::Integer(value) => {
                digest.update([1]);
                digest.update(value.to_le_bytes());
            }
            ValueRef::Real(value) => {
                digest.update([2]);
                digest.update(value.to_bits().to_le_bytes());
            }
            ValueRef::Text(value) => {
                digest.update([3]);
                digest.update((value.len() as u64).to_le_bytes());
                digest.update(value);
            }
            ValueRef::Blob(value) => {
                digest.update([4]);
                digest.update((value.len() as u64).to_le_bytes());
                digest.update(value);
            }
        }
    }
    Ok(digest.finalize().into())
}
pub(super) fn reserve(governor: &MemoryGovernor, bytes: usize) -> Result<ByteLease> {
    governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Other,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)
}
fn invalid(message: &str) -> Failure {
    Failure::Invalid(message.into())
}
pub(super) fn record(db: &Connection, table: &str, id: &str) -> Result<[u8; 32]> {
    let mut query = db
        .prepare(&format!("SELECT * FROM {table} WHERE id=?"))
        .map_err(storage)?;
    let mut rows = query.query([id]).map_err(storage)?;
    signature(
        rows.next()
            .map_err(storage)?
            .ok_or_else(|| invalid("A reviewed record no longer exists"))?,
    )
}
fn target(
    db: &Connection,
    action: &RuleAction,
    governor: &MemoryGovernor,
    leases: &mut Vec<ByteLease>,
    previous: &[PlannedAction],
    cancel: &Cancellation,
) -> Result<PlannedTarget> {
    let kind = action.kind;
    let value = action.value.trim();
    let table = match kind {
        ActionKind::AddTag => "tags",
        ActionKind::AddCollection => "collections",
        ActionKind::AddWorkspace | ActionKind::AddReadingQueue => "workspaces",
        _ => unreachable!(),
    };
    let value = value.trim();
    let normalized = value.to_lowercase();
    let mut selected: Option<PlannedTarget> = None;
    let condition = if table == "workspaces" {
        "?2 IS NOT NULL"
    } else {
        "id=?1 OR normalized_name=?2"
    };
    let mut query = db
        .prepare(&format!(
            "SELECT * FROM {table} WHERE {condition} ORDER BY CASE WHEN id=?1 THEN 0 ELSE 1 END,id"
        ))
        .map_err(storage)?;
    let mut rows = query.query(params![value, normalized]).map_err(storage)?;
    while let Some(row) = rows.next().map_err(storage)? {
        cancel.check()?;
        let id = field(row, 0)?;
        let name = field(row, 1)?;
        if id != value && name.to_lowercase() != normalized {
            continue;
        }
        if selected.is_some() {
            return Err(invalid("More than one workspace has this name; use its ID"));
        }
        let bytes = id
            .len()
            .checked_add(name.len())
            .and_then(|size| size.checked_mul(2))
            .and_then(|size| size.checked_add(256))
            .ok_or(Failure::MemoryBudget)?;
        leases.push(reserve(governor, bytes)?);
        selected = Some(PlannedTarget {
            id: id.into(),
            name: name.into(),
            revision: Some(signature(row)?),
        });
        if id == value || table != "workspaces" {
            break;
        }
    }
    if let Some(selected) = selected {
        return Ok(selected);
    }
    if table == "workspaces" {
        return Err(invalid("The target workspace does not exist"));
    }
    if value.is_empty() || value.encode_utf16().count() > 60 {
        return Err(invalid(
            "Use a tag or collection name between 1 and 60 UTF-16 units",
        ));
    }
    leases.push(reserve(
        governor,
        value
            .len()
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(320))
            .ok_or(Failure::MemoryBudget)?,
    )?);
    for action in previous {
        let prior = match (kind, action) {
            (ActionKind::AddTag, PlannedAction::AddTag(target))
            | (ActionKind::AddCollection, PlannedAction::AddCollection(target)) => Some(target),
            _ => None,
        };
        if let Some(prior) = prior
            && prior.will_create()
            && prior.name.to_lowercase() == normalized
        {
            return Ok(prior.clone());
        }
    }
    Ok(PlannedTarget {
        id: uuid::Uuid::new_v4().to_string(),
        name: value.into(),
        revision: None,
    })
}
fn build(
    db: &Connection,
    report: Arc<SimulationReport>,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<RulePlan> {
    verify(db, &report.rule)?;
    let definition = report
        .rule
        .definition()
        .ok_or_else(|| invalid("This saved rule format needs review"))?;
    definition.validate()?;
    if report.matches.is_empty()
        || report.matches.len() as u64 != report.progress.matched
        || report.progress.matched > report.progress.scanned
        || report.progress.scanned > MAX_SIMULATION_DOCUMENTS
        || report
            .matches
            .windows(2)
            .any(|pair| pair[0].document >= pair[1].document)
    {
        return Err(invalid(
            "Run a complete, nonempty simulation before reviewing actions",
        ));
    }
    if definition.trigger.kind == TriggerKind::ExactDuplicate && report.progress.unverified != 0 {
        return Err(invalid(
            "Verify the unavailable or changed sources before reviewing duplicate actions",
        ));
    }
    let bytes = report
        .matches
        .len()
        .checked_mul(std::mem::size_of::<[u8; 32]>())
        .and_then(|bytes| {
            bytes.checked_add(
                std::mem::size_of::<RulePlan>() + 8 * std::mem::size_of::<PlannedAction>() + 512,
            )
        })
        .ok_or(Failure::MemoryBudget)?;
    let mut leases = vec![reserve(governor, bytes)?];
    let mut revisions = Vec::new();
    revisions
        .try_reserve_exact(report.matches.len())
        .map_err(|_| Failure::MemoryBudget)?;
    let trigger = &definition.trigger;
    let value = trigger.value.as_deref().unwrap_or("").trim();
    let normalized = value.to_lowercase();
    let mut query = db
        .prepare(&super::simulation::select(
            trigger,
            "d.id=?1 AND ?2 IS NULL AND ?3 IS NULL",
        ))
        .map_err(storage)?;
    for matched in &report.matches {
        cancel.check()?;
        let mut rows = query
            .query(params![
                matched.document,
                None::<String>,
                None::<String>,
                value,
                normalized
            ])
            .map_err(storage)?;
        let row = rows
            .next()
            .map_err(storage)?
            .ok_or_else(|| invalid("A matched document was removed; simulate again"))?;
        if signature(row)? != matched.revision {
            return Err(invalid(
                "A matched document or condition changed; simulate again",
            ));
        }
        revisions.push(record(db, "documents", &matched.document)?);
    }
    let mut actions = Vec::with_capacity(definition.actions.len());
    for action in &definition.actions {
        cancel.check()?;
        let value = action.value.trim();
        actions.push(match action.kind {
            ActionKind::AddTag => {
                PlannedAction::AddTag(target(db, action, governor, &mut leases, &actions, cancel)?)
            }
            ActionKind::AddCollection => PlannedAction::AddCollection(target(
                db,
                action,
                governor,
                &mut leases,
                &actions,
                cancel,
            )?),
            ActionKind::AddWorkspace => PlannedAction::AddWorkspace(target(
                db,
                action,
                governor,
                &mut leases,
                &actions,
                cancel,
            )?),
            ActionKind::AddReadingQueue => PlannedAction::AddReadingQueue(target(
                db,
                action,
                governor,
                &mut leases,
                &actions,
                cancel,
            )?),
            ActionKind::SetPriority => PlannedAction::SetFavorite(
                !["0", "false", "normal"].contains(&value.to_lowercase().as_str()),
            ),
            ActionKind::SetReadingState => PlannedAction::SetReadingState {
                finished: match value.to_lowercase().as_str() {
                    "finished" | "terminado" => true,
                    "pending" | "pendiente" | "notstarted" => false,
                    _ => return Err(invalid("Choose finished or pending as the reading state")),
                },
            },
        });
    }
    cancel.check()?;
    Ok(RulePlan::new(report, actions, revisions, leases))
}
pub(super) fn prepare(
    db: &Connection,
    report: Arc<SimulationReport>,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<RulePlan> {
    cancel.check()?;
    db.execute_batch("BEGIN DEFERRED").map_err(storage)?;
    let result = build(db, report, governor, cancel);
    db.execute_batch("ROLLBACK").map_err(storage)?;
    result
}
